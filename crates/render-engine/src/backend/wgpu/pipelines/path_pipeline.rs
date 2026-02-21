//! Tessellated path utilities and Phase 3 path pipeline foundation.

use crate::backend::wgpu::image_store;
use crate::backend::wgpu::pipelines::gradient_atlas::{GradientAtlas, GradientParams};
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, LineCap, LineJoin,
    StrokeOptions, StrokeTessellator, StrokeVertex, VertexBuffers,
};
use std::collections::HashMap;
use std::sync::Arc;
use style_engine::{
    Paint, PathCommand, StrokeCap, StrokeJoin, StrokeStyle, VectorPath, WindingRule,
};
use thiserror::Error;
use wgpu::util::DeviceExt;

/// Vertex payload for tessellated path rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PathVertex {
    /// Tessellated local-space position.
    pub position: [f32; 2],
    /// Optional normal. Fill tessellation uses `[0, 0]`.
    pub normal: [f32; 2],
}

/// CPU-side tessellation output.
#[derive(Debug, Clone, Default)]
pub struct PathMesh {
    pub vertices: Vec<PathVertex>,
    pub indices: Vec<u32>,
}

/// GPU vertex payload for path rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PathGpuVertex {
    pub position: [f32; 2],
    pub normal: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 2],
    pub fill_type: u32,
    pub gradient_index: u32,
}

/// Prepared path draw batch for upload.
#[derive(Debug, Clone)]
pub struct PathBatch {
    pub mesh: Arc<PathMesh>,
    pub paint: Paint,
    pub opacity: f32,
    pub size: [f32; 2],
    pub offset: [f32; 2],
}

/// Simple bounded tessellation cache with LRU eviction.
#[derive(Debug)]
pub struct TessellationCache {
    cache: HashMap<u64, Arc<PathMesh>>,
    lyon_paths: HashMap<u64, Arc<Path>>,
    interned_path_keys: HashMap<usize, InternedPathKeyEntry>,
    access_epoch: HashMap<u64, u64>,
    epoch: u64,
    max_entries: usize,
    stats: TessellationCacheStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PathFingerprint {
    command_len: usize,
    winding: WindingRule,
    first: u64,
    last: u64,
}

#[derive(Debug, Clone, Copy)]
struct InternedPathKeyEntry {
    path_hash: u64,
    fingerprint: PathFingerprint,
}

/// Runtime counters for tessellation cache behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TessellationCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

/// Tessellation failures during vector path conversion or lyon tessellation.
#[derive(Debug, Error)]
pub enum TessellationError {
    #[error("lyon tessellation failed: {0}")]
    Lyon(String),
}

/// Phase 3 render pipeline placeholder for tessellated paths.
///
/// This skeleton exists so Phase 3 can wire tessellation and GPU upload in
/// incremental steps without changing the public API shape again.
pub struct PathPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    index_count: u32,
    gradient_atlas: GradientAtlas,
    gradient_params_buffer: wgpu::Buffer,
    gradient_params_capacity: usize,
    gradient_params: Vec<GradientParams>,
    gradient_bind_group: Option<wgpu::BindGroup>,
    gradient_bind_group_layout: wgpu::BindGroupLayout,
}

const INITIAL_VERTEX_CAPACITY: usize = 4096;
const INITIAL_INDEX_CAPACITY: usize = 8192;
const INITIAL_GRADIENT_CAPACITY: usize = 256;
const IMAGE_SUBDIVISION_TARGET_PIXELS: f32 = 4.0;
const IMAGE_SUBDIVISION_MAX: u32 = 128;
const IMAGE_SUBDIVISION_TRIANGLE_BUDGET: u32 = 16_384;

#[inline]
fn mix_u64(mut state: u64, value: u64) -> u64 {
    let mixed = value
        .wrapping_add(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(state << 6)
        .wrapping_add(state >> 2);
    state ^= mixed;
    state
}

#[inline]
fn hash_vec2_fast(mut state: u64, value: glam::Vec2) -> u64 {
    state = mix_u64(state, value.x.to_bits() as u64);
    mix_u64(state, value.y.to_bits() as u64)
}

#[inline]
fn hash_vec4_fast(mut state: u64, value: glam::Vec4) -> u64 {
    state = mix_u64(state, value.x.to_bits() as u64);
    state = mix_u64(state, value.y.to_bits() as u64);
    state = mix_u64(state, value.z.to_bits() as u64);
    mix_u64(state, value.w.to_bits() as u64)
}

#[inline]
fn hash_paint_fast(mut state: u64, paint: &Paint) -> u64 {
    match paint {
        Paint::Solid(color) => {
            state = mix_u64(state, 0);
            hash_vec4_fast(state, *color)
        }
        Paint::Linear(gradient) => {
            state = mix_u64(state, 1);
            state = hash_vec2_fast(state, gradient.start);
            state = hash_vec2_fast(state, gradient.end);
            state = mix_u64(state, gradient.stops.len() as u64);
            for stop in &gradient.stops {
                state = mix_u64(state, stop.position.to_bits() as u64);
                state = hash_vec4_fast(state, stop.color);
            }
            state
        }
        Paint::Radial(gradient) => {
            state = mix_u64(state, 2);
            state = hash_vec2_fast(state, gradient.center);
            state = mix_u64(state, gradient.radius.to_bits() as u64);
            state = mix_u64(state, gradient.stops.len() as u64);
            for stop in &gradient.stops {
                state = mix_u64(state, stop.position.to_bits() as u64);
                state = hash_vec4_fast(state, stop.color);
            }
            state
        }
        Paint::Angular(gradient) => {
            state = mix_u64(state, 3);
            state = hash_vec2_fast(state, gradient.center);
            state = mix_u64(state, gradient.angle.to_bits() as u64);
            state = mix_u64(state, gradient.stops.len() as u64);
            for stop in &gradient.stops {
                state = mix_u64(state, stop.position.to_bits() as u64);
                state = hash_vec4_fast(state, stop.color);
            }
            state
        }
        Paint::Diamond(gradient) => {
            state = mix_u64(state, 4);
            state = hash_vec2_fast(state, gradient.center);
            state = mix_u64(state, gradient.scale.to_bits() as u64);
            state = mix_u64(state, gradient.stops.len() as u64);
            for stop in &gradient.stops {
                state = mix_u64(state, stop.position.to_bits() as u64);
                state = hash_vec4_fast(state, stop.color);
            }
            state
        }
        Paint::Image(image) => {
            state = mix_u64(state, 5);
            state = mix_u64(state, image.image_id.0);
            let scale_mode = match image.scale_mode {
                style_engine::ImageScaleMode::Fill => 0u64,
                style_engine::ImageScaleMode::Fit => 1u64,
                style_engine::ImageScaleMode::Crop => 2u64,
                style_engine::ImageScaleMode::Tile => 3u64,
                style_engine::ImageScaleMode::Stretch => 4u64,
            };
            state = mix_u64(state, scale_mode);
            if let Some(transform) = image.transform {
                for v in transform {
                    state = mix_u64(state, v.to_bits() as u64);
                }
            }
            state
        }
    }
}

fn sample_paint_at_uv(paint: &Paint, uv: glam::Vec2, target_size: glam::Vec2) -> glam::Vec4 {
    match paint {
        Paint::Solid(color) => *color,
        Paint::Linear(gradient) => {
            let dir = gradient.end - gradient.start;
            let dir_len_sq = dir.dot(dir);
            let t = if dir_len_sq < 0.0001 {
                0.0
            } else {
                ((uv - gradient.start).dot(dir) / dir_len_sq).clamp(0.0, 1.0)
            };
            Paint::interpolate_stops(t, &gradient.stops)
        }
        Paint::Radial(gradient) => {
            let radius = gradient.radius.abs();
            let t = if radius < 0.0001 {
                0.0
            } else {
                (uv.distance(gradient.center) / radius).clamp(0.0, 1.0)
            };
            Paint::interpolate_stops(t, &gradient.stops)
        }
        Paint::Angular(gradient) => {
            let d = uv - gradient.center;
            let angle = d.y.atan2(d.x);
            let mut t = (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
            if t < 0.0 {
                t += 1.0;
            }
            Paint::interpolate_stops(t.fract(), &gradient.stops)
        }
        Paint::Diamond(gradient) => {
            let d = (uv - gradient.center).abs();
            let scale = gradient.scale.abs().max(0.0001);
            let t = ((d.x / scale) + (d.y / scale)).clamp(0.0, 1.0);
            Paint::interpolate_stops(t, &gradient.stops)
        }
        Paint::Image(image) => image_store::sample_image_fill(image, uv, target_size)
            .unwrap_or_else(|| glam::Vec4::new(1.0, 0.0, 1.0, 1.0)),
    }
}

fn image_subdivision_steps(size: glam::Vec2, base_triangle_count: usize) -> u32 {
    let desired = ((size.x.max(size.y).max(1.0) / IMAGE_SUBDIVISION_TARGET_PIXELS).ceil() as u32)
        .clamp(2, IMAGE_SUBDIVISION_MAX);
    let max_by_budget = ((IMAGE_SUBDIVISION_TRIANGLE_BUDGET as f32
        / base_triangle_count.max(1) as f32)
        .sqrt()
        .floor() as u32)
        .max(1);
    desired.min(max_by_budget).max(1)
}

fn sample_batch_color(batch: &PathBatch, local_pos: glam::Vec2, size: glam::Vec2) -> [f32; 4] {
    match &batch.paint {
        Paint::Solid(color) => {
            let mut c = *color;
            c.w *= batch.opacity;
            c.to_array()
        }
        Paint::Image(_) => {
            let mut c = sample_paint_at_uv(
                &batch.paint,
                (local_pos / size).clamp(glam::Vec2::ZERO, glam::Vec2::ONE),
                size,
            );
            c.w *= batch.opacity;
            c.to_array()
        }
        Paint::Linear(_) | Paint::Radial(_) | Paint::Angular(_) | Paint::Diamond(_) => {
            [1.0, 1.0, 1.0, batch.opacity]
        }
    }
}

fn push_batch_vertex(
    batch: &PathBatch,
    vertices: &mut Vec<PathGpuVertex>,
    local_pos: glam::Vec2,
    normal: glam::Vec2,
    size: glam::Vec2,
    runtime: PaintRuntime,
) -> u32 {
    let idx = vertices.len() as u32;
    let uv = (local_pos / size).clamp(glam::Vec2::ZERO, glam::Vec2::ONE);
    vertices.push(PathGpuVertex {
        position: [local_pos.x + batch.offset[0], local_pos.y + batch.offset[1]],
        normal: normal.to_array(),
        color: sample_batch_color(batch, local_pos, size),
        uv: uv.to_array(),
        fill_type: runtime.fill_type,
        gradient_index: runtime.gradient_index,
    });
    idx
}

struct ImageSampleTriangle {
    positions: [glam::Vec2; 3],
    normals: [glam::Vec2; 3],
}

fn append_subdivided_image_triangle(
    batch: &PathBatch,
    vertices: &mut Vec<PathGpuVertex>,
    indices: &mut Vec<u32>,
    size: glam::Vec2,
    subdivision_steps: u32,
    triangle: &ImageSampleTriangle,
    runtime: PaintRuntime,
) {
    let steps = subdivision_steps.max(1);
    let mut row_indices: Vec<Vec<u32>> = Vec::with_capacity((steps + 1) as usize);
    let [p0, p1, p2] = triangle.positions;
    let [n0, n1, n2] = triangle.normals;

    for row in 0..=steps {
        let row_t = row as f32 / steps as f32;
        let start_pos = p0.lerp(p2, row_t);
        let end_pos = p1.lerp(p2, row_t);
        let start_normal = n0.lerp(n2, row_t);
        let end_normal = n1.lerp(n2, row_t);

        let cols = steps - row;
        let mut row_ids = Vec::with_capacity((cols + 1) as usize);
        for col in 0..=cols {
            let col_t = if cols == 0 {
                0.0
            } else {
                col as f32 / cols as f32
            };
            let local_pos = start_pos.lerp(end_pos, col_t);
            let normal = start_normal.lerp(end_normal, col_t);
            row_ids.push(push_batch_vertex(
                batch, vertices, local_pos, normal, size, runtime,
            ));
        }
        row_indices.push(row_ids);
    }

    for row in 0..steps as usize {
        let top = &row_indices[row];
        let bottom = &row_indices[row + 1];
        let cols = bottom.len();
        for col in 0..cols {
            indices.push(top[col]);
            indices.push(top[col + 1]);
            indices.push(bottom[col]);
            if col + 1 < cols {
                indices.push(top[col + 1]);
                indices.push(bottom[col + 1]);
                indices.push(bottom[col]);
            }
        }
    }
}

fn append_batch_geometry(
    batch: &PathBatch,
    vertices: &mut Vec<PathGpuVertex>,
    indices: &mut Vec<u32>,
    runtime: PaintRuntime,
) {
    if batch.mesh.vertices.is_empty() || batch.mesh.indices.is_empty() {
        return;
    }

    let size = glam::Vec2::new(batch.size[0].max(1.0), batch.size[1].max(1.0));
    if matches!(batch.paint, Paint::Image(_)) {
        let subdivision_steps = image_subdivision_steps(size, batch.mesh.indices.len() / 3);
        for tri in batch.mesh.indices.chunks_exact(3) {
            let ia = tri[0] as usize;
            let ib = tri[1] as usize;
            let ic = tri[2] as usize;
            let va = batch.mesh.vertices[ia];
            let vb = batch.mesh.vertices[ib];
            let vc = batch.mesh.vertices[ic];
            let triangle = ImageSampleTriangle {
                positions: [
                    glam::Vec2::from(va.position),
                    glam::Vec2::from(vb.position),
                    glam::Vec2::from(vc.position),
                ],
                normals: [
                    glam::Vec2::from(va.normal),
                    glam::Vec2::from(vb.normal),
                    glam::Vec2::from(vc.normal),
                ],
            };
            append_subdivided_image_triangle(
                batch,
                vertices,
                indices,
                size,
                subdivision_steps,
                &triangle,
                runtime,
            );
        }
        return;
    }

    let base_vertex = vertices.len() as u32;
    for v in &batch.mesh.vertices {
        let local_pos = glam::Vec2::new(v.position[0], v.position[1]);
        let normal = glam::Vec2::new(v.normal[0], v.normal[1]);
        let _ = push_batch_vertex(batch, vertices, local_pos, normal, size, runtime);
    }
    indices.extend(batch.mesh.indices.iter().map(|i| i + base_vertex));
}

fn hash_vector_path(path: &VectorPath) -> u64 {
    let mut hash = match path.winding_rule {
        WindingRule::NonZero => 0x243F_6A88_85A3_08D3,
        WindingRule::EvenOdd => 0x1319_8A2E_0370_7344,
    };
    hash = mix_u64(hash, path.commands.len() as u64);
    for command in &path.commands {
        match *command {
            PathCommand::MoveTo(p) => {
                hash = mix_u64(hash, 0);
                hash = hash_vec2_fast(hash, p);
            }
            PathCommand::LineTo(p) => {
                hash = mix_u64(hash, 1);
                hash = hash_vec2_fast(hash, p);
            }
            PathCommand::QuadraticTo { control, to } => {
                hash = mix_u64(hash, 2);
                hash = hash_vec2_fast(hash, control);
                hash = hash_vec2_fast(hash, to);
            }
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => {
                hash = mix_u64(hash, 3);
                hash = hash_vec2_fast(hash, control1);
                hash = hash_vec2_fast(hash, control2);
                hash = hash_vec2_fast(hash, to);
            }
            PathCommand::Close => {
                hash = mix_u64(hash, 4);
            }
        }
    }
    hash ^ (hash >> 33)
}

fn hash_stroke_style(stroke: &StrokeStyle) -> u64 {
    let mut hash = 0x9E37_79B9_7F4A_7C15u64;
    hash = mix_u64(hash, stroke.weight.to_bits() as u64);
    hash = mix_u64(
        hash,
        match stroke.align {
            style_engine::StrokeAlign::Inside => 0,
            style_engine::StrokeAlign::Center => 1,
            style_engine::StrokeAlign::Outside => 2,
        },
    );
    hash = mix_u64(
        hash,
        match stroke.cap {
            StrokeCap::Butt => 0,
            StrokeCap::Round => 1,
            StrokeCap::Square => 2,
        },
    );
    hash = mix_u64(
        hash,
        match stroke.join {
            StrokeJoin::Miter => 0,
            StrokeJoin::Bevel => 1,
            StrokeJoin::Round => 2,
        },
    );
    hash = mix_u64(hash, stroke.miter_limit.to_bits() as u64);
    hash = mix_u64(hash, stroke.dash_pattern.len() as u64);
    for dash in &stroke.dash_pattern {
        hash = mix_u64(hash, dash.to_bits() as u64);
    }
    hash = mix_u64(hash, stroke.dash_offset.to_bits() as u64);
    if let Some(side) = stroke.side_weights {
        hash = mix_u64(hash, side.top.to_bits() as u64);
        hash = mix_u64(hash, side.right.to_bits() as u64);
        hash = mix_u64(hash, side.bottom.to_bits() as u64);
        hash = mix_u64(hash, side.left.to_bits() as u64);
    }
    hash = mix_u64(hash, stroke.paints.len() as u64);
    for paint in &stroke.paints {
        hash = hash_paint_fast(hash, paint);
    }
    hash ^ (hash >> 33)
}

#[inline]
fn stroke_key_from_parts(path_hash: u64, stroke_hash: u64) -> u64 {
    path_hash ^ stroke_hash.rotate_left(1)
}

impl TessellationCache {
    fn command_fingerprint(command: PathCommand) -> u64 {
        match command {
            PathCommand::MoveTo(p) => {
                0x01u64 ^ ((p.x.to_bits() as u64) << 1) ^ ((p.y.to_bits() as u64).rotate_left(17))
            }
            PathCommand::LineTo(p) => {
                0x02u64 ^ ((p.x.to_bits() as u64) << 1) ^ ((p.y.to_bits() as u64).rotate_left(17))
            }
            PathCommand::QuadraticTo { control, to } => {
                0x03u64
                    ^ ((control.x.to_bits() as u64) << 1)
                    ^ ((control.y.to_bits() as u64).rotate_left(9))
                    ^ ((to.x.to_bits() as u64).rotate_left(17))
                    ^ ((to.y.to_bits() as u64).rotate_left(29))
            }
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => {
                0x04u64
                    ^ ((control1.x.to_bits() as u64) << 1)
                    ^ ((control1.y.to_bits() as u64).rotate_left(7))
                    ^ ((control2.x.to_bits() as u64).rotate_left(13))
                    ^ ((control2.y.to_bits() as u64).rotate_left(19))
                    ^ ((to.x.to_bits() as u64).rotate_left(23))
                    ^ ((to.y.to_bits() as u64).rotate_left(31))
            }
            PathCommand::Close => 0x05u64,
        }
    }

    fn path_fingerprint(path: &VectorPath) -> PathFingerprint {
        let first = path
            .commands
            .first()
            .copied()
            .map(Self::command_fingerprint)
            .unwrap_or(0);
        let last = path
            .commands
            .last()
            .copied()
            .map(Self::command_fingerprint)
            .unwrap_or(0);
        PathFingerprint {
            command_len: path.commands.len(),
            winding: path.winding_rule,
            first,
            last,
        }
    }

    fn fill_key_interned(&mut self, path: &VectorPath) -> u64 {
        let ptr = path as *const VectorPath as usize;
        let fingerprint = Self::path_fingerprint(path);
        if let Some(entry) = self.interned_path_keys.get(&ptr)
            && entry.fingerprint == fingerprint
        {
            return entry.path_hash;
        }

        let path_hash = hash_vector_path(path);
        self.interned_path_keys.insert(
            ptr,
            InternedPathKeyEntry {
                path_hash,
                fingerprint,
            },
        );
        path_hash
    }

    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            lyon_paths: HashMap::new(),
            interned_path_keys: HashMap::new(),
            access_epoch: HashMap::new(),
            epoch: 0,
            max_entries: max_entries.max(1),
            stats: TessellationCacheStats::default(),
        }
    }

    fn touch(&mut self, key: u64) {
        self.epoch = self.epoch.wrapping_add(1);
        self.access_epoch.insert(key, self.epoch);
    }

    fn evict_if_needed(&mut self) {
        while self.cache.len() > self.max_entries {
            if let Some((&oldest_key, _)) =
                self.access_epoch.iter().min_by_key(|(_, epoch)| **epoch)
            {
                self.cache.remove(&oldest_key);
                self.lyon_paths.remove(&oldest_key);
                self.access_epoch.remove(&oldest_key);
                self.stats.evictions = self.stats.evictions.saturating_add(1);
            } else {
                break;
            }
        }
    }

    fn get_cached_mesh(&mut self, key: u64) -> Option<Arc<PathMesh>> {
        if let Some(mesh) = self.cache.get(&key).cloned() {
            self.stats.hits = self.stats.hits.saturating_add(1);
            self.touch(key);
            return Some(mesh);
        }
        None
    }

    fn get_or_build_lyon_path(&mut self, path_hash: u64, path: &VectorPath) -> Arc<Path> {
        if let Some(lyon_path) = self.lyon_paths.get(&path_hash).cloned() {
            return lyon_path;
        }
        let lyon_path = Arc::new(vector_path_to_lyon(path));
        self.lyon_paths.insert(path_hash, Arc::clone(&lyon_path));
        lyon_path
    }

    fn insert_mesh(&mut self, key: u64, mesh: Arc<PathMesh>) -> Arc<PathMesh> {
        self.cache.insert(key, Arc::clone(&mesh));
        self.touch(key);
        self.evict_if_needed();
        mesh
    }

    fn record_miss(&mut self) {
        self.stats.misses = self.stats.misses.saturating_add(1);
    }

    pub fn fill_key(path: &VectorPath) -> u64 {
        hash_vector_path(path)
    }

    pub fn stroke_key(path: &VectorPath, stroke: &StrokeStyle) -> u64 {
        stroke_key_from_parts(hash_vector_path(path), hash_stroke_style(stroke))
    }

    pub fn stroke_key_from_path_hash(path_hash: u64, stroke: &StrokeStyle) -> u64 {
        stroke_key_from_parts(path_hash, hash_stroke_style(stroke))
    }

    pub fn get_or_tessellate_fill_with_key(
        &mut self,
        key: u64,
        path: &VectorPath,
    ) -> Result<Arc<PathMesh>, TessellationError> {
        if let Some(mesh) = self.get_cached_mesh(key) {
            return Ok(mesh);
        }
        self.record_miss();
        let lyon_path = self.get_or_build_lyon_path(key, path);
        let mesh = Arc::new(tessellate_fill_from_lyon_path(
            &lyon_path,
            path.winding_rule,
        )?);
        Ok(self.insert_mesh(key, mesh))
    }

    pub fn get_or_tessellate_fill(
        &mut self,
        path: &VectorPath,
    ) -> Result<Arc<PathMesh>, TessellationError> {
        let key = self.fill_key_interned(path);
        self.get_or_tessellate_fill_with_key(key, path)
    }

    pub fn get_or_tessellate_stroke_with_key(
        &mut self,
        key: u64,
        path: &VectorPath,
        stroke: &StrokeStyle,
    ) -> Result<Arc<PathMesh>, TessellationError> {
        if let Some(mesh) = self.get_cached_mesh(key) {
            return Ok(mesh);
        }
        self.record_miss();
        let path_hash = Self::fill_key(path);
        let lyon_path = self.get_or_build_lyon_path(path_hash, path);
        let mesh = Arc::new(tessellate_stroke_from_lyon_path(&lyon_path, stroke)?);
        Ok(self.insert_mesh(key, mesh))
    }

    pub fn get_or_tessellate_stroke(
        &mut self,
        path: &VectorPath,
        stroke: &StrokeStyle,
    ) -> Result<Arc<PathMesh>, TessellationError> {
        let path_hash = self.fill_key_interned(path);
        let key = stroke_key_from_parts(path_hash, hash_stroke_style(stroke));
        self.get_or_tessellate_stroke_with_key(key, path, stroke)
    }

    pub fn stats(&self) -> TessellationCacheStats {
        self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = TessellationCacheStats::default();
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.cache.len()
    }
}

fn winding_to_fill_rule(winding: WindingRule) -> FillRule {
    match winding {
        WindingRule::NonZero => FillRule::NonZero,
        WindingRule::EvenOdd => FillRule::EvenOdd,
    }
}

fn stroke_cap_to_line_cap(cap: StrokeCap) -> LineCap {
    match cap {
        StrokeCap::Butt => LineCap::Butt,
        StrokeCap::Round => LineCap::Round,
        StrokeCap::Square => LineCap::Square,
    }
}

fn stroke_join_to_line_join(join: StrokeJoin) -> LineJoin {
    match join {
        StrokeJoin::Miter => LineJoin::Miter,
        StrokeJoin::Bevel => LineJoin::Bevel,
        StrokeJoin::Round => LineJoin::Round,
    }
}

/// Convert a `style_engine::VectorPath` to a lyon path.
pub fn vector_path_to_lyon(path: &VectorPath) -> Path {
    let mut builder = Path::builder();
    let mut contour_open = false;

    for command in &path.commands {
        match *command {
            PathCommand::MoveTo(p) => {
                if contour_open {
                    builder.end(false);
                }
                builder.begin(point(p.x, p.y));
                contour_open = true;
            }
            PathCommand::LineTo(p) => {
                builder.line_to(point(p.x, p.y));
            }
            PathCommand::QuadraticTo { control, to } => {
                builder.quadratic_bezier_to(point(control.x, control.y), point(to.x, to.y));
            }
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => {
                builder.cubic_bezier_to(
                    point(control1.x, control1.y),
                    point(control2.x, control2.y),
                    point(to.x, to.y),
                );
            }
            PathCommand::Close => {
                builder.close();
                contour_open = false;
            }
        }
    }

    if contour_open {
        builder.end(false);
    }

    builder.build()
}

/// Tessellate path fill geometry using lyon.
pub fn tessellate_fill(path: &VectorPath) -> Result<PathMesh, TessellationError> {
    if path.commands.is_empty() {
        return Ok(PathMesh::default());
    }

    let lyon_path = vector_path_to_lyon(path);
    tessellate_fill_from_lyon_path(&lyon_path, path.winding_rule)
}

fn tessellate_fill_from_lyon_path(
    lyon_path: &Path,
    winding_rule: WindingRule,
) -> Result<PathMesh, TessellationError> {
    let options = FillOptions::tolerance(0.1).with_fill_rule(winding_to_fill_rule(winding_rule));

    let mut buffers: VertexBuffers<PathVertex, u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    tessellator
        .tessellate_path(
            lyon_path,
            &options,
            &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex<'_>| PathVertex {
                position: vertex.position().to_array(),
                normal: [0.0, 0.0],
            }),
        )
        .map_err(|e| TessellationError::Lyon(format!("{e:?}")))?;

    Ok(PathMesh {
        vertices: buffers.vertices,
        indices: buffers.indices,
    })
}

/// Tessellate path stroke geometry using lyon.
pub fn tessellate_stroke(
    path: &VectorPath,
    stroke: &StrokeStyle,
) -> Result<PathMesh, TessellationError> {
    if path.commands.is_empty() || stroke.weight <= 0.0 {
        return Ok(PathMesh::default());
    }

    let lyon_path = vector_path_to_lyon(path);
    tessellate_stroke_from_lyon_path(&lyon_path, stroke)
}

fn tessellate_stroke_from_lyon_path(
    lyon_path: &Path,
    stroke: &StrokeStyle,
) -> Result<PathMesh, TessellationError> {
    let options = StrokeOptions::tolerance(0.1)
        .with_line_width(stroke.weight)
        .with_line_cap(stroke_cap_to_line_cap(stroke.cap))
        .with_line_join(stroke_join_to_line_join(stroke.join));

    let mut buffers: VertexBuffers<PathVertex, u32> = VertexBuffers::new();
    let mut tessellator = StrokeTessellator::new();
    tessellator
        .tessellate_path(
            lyon_path,
            &options,
            &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex<'_, '_>| {
                let normal = vertex.normal().to_array();
                PathVertex {
                    position: vertex.position().to_array(),
                    normal,
                }
            }),
        )
        .map_err(|e| TessellationError::Lyon(format!("{e:?}")))?;

    Ok(PathMesh {
        vertices: buffers.vertices,
        indices: buffers.indices,
    })
}

fn gradient_params_for_paint(paint: &Paint, atlas_row: u32) -> Option<GradientParams> {
    match paint {
        Paint::Linear(gradient) => Some(GradientParams::linear(
            gradient.start.to_array(),
            gradient.end.to_array(),
            atlas_row,
        )),
        Paint::Radial(gradient) => Some(GradientParams::radial(
            gradient.center.to_array(),
            gradient.radius,
            atlas_row,
        )),
        Paint::Angular(gradient) => Some(GradientParams::angular(
            gradient.center.to_array(),
            atlas_row,
        )),
        Paint::Diamond(gradient) => Some(GradientParams::diamond(
            gradient.center.to_array(),
            [gradient.scale, gradient.scale],
            atlas_row,
        )),
        Paint::Solid(_) | Paint::Image(_) => None,
    }
}

impl PathPipeline {
    pub fn new(
        device: &wgpu::Device,
        globals_bind_group_layout: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Path Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/path.wgsl").into()),
        });

        let gradient_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Path Gradient Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let mut gradient_atlas = GradientAtlas::default();
        gradient_atlas.init_gpu(device);
        let gradient_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Path Gradient Params Storage Buffer"),
            size: (INITIAL_GRADIENT_CAPACITY * std::mem::size_of::<GradientParams>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let gradient_bind_group = if let (Some(view), Some(sampler)) =
            (gradient_atlas.texture_view(), gradient_atlas.sampler())
        {
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Path Gradient Bind Group"),
                layout: &gradient_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: gradient_params_buffer.as_entire_binding(),
                    },
                ],
            }))
        } else {
            None
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Path Pipeline Layout"),
            bind_group_layouts: &[globals_bind_group_layout, &gradient_bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Path Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<PathGpuVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                        2 => Float32x4,
                        3 => Float32x2,
                        4 => Uint32,
                        5 => Uint32
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Path Vertex Buffer"),
            size: (INITIAL_VERTEX_CAPACITY * std::mem::size_of::<PathGpuVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Path Index Buffer"),
            size: (INITIAL_INDEX_CAPACITY * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            vertex_capacity: INITIAL_VERTEX_CAPACITY,
            index_capacity: INITIAL_INDEX_CAPACITY,
            index_count: 0,
            gradient_atlas,
            gradient_params_buffer,
            gradient_params_capacity: INITIAL_GRADIENT_CAPACITY,
            gradient_params: Vec::new(),
            gradient_bind_group,
            gradient_bind_group_layout,
        }
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, batches: &[PathBatch]) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        self.gradient_params.clear();

        for batch in batches {
            let runtime = if let Some(mut params) = gradient_params_for_paint(&batch.paint, 0) {
                let atlas_row = match &batch.paint {
                    Paint::Linear(gradient) => self.gradient_atlas.add_gradient(&gradient.stops),
                    Paint::Radial(gradient) => self.gradient_atlas.add_gradient(&gradient.stops),
                    Paint::Angular(gradient) => self.gradient_atlas.add_gradient(&gradient.stops),
                    Paint::Diamond(gradient) => self.gradient_atlas.add_gradient(&gradient.stops),
                    Paint::Solid(_) | Paint::Image(_) => 0,
                };
                params.atlas_row = (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32;
                let gradient_index = self.gradient_params.len() as u32;
                self.gradient_params.push(params);
                PaintRuntime {
                    fill_type: params.gradient_type + 1,
                    gradient_index,
                }
            } else {
                PaintRuntime::default()
            };
            append_batch_geometry(batch, &mut vertices, &mut indices, runtime);
        }

        self.gradient_atlas.upload_to_gpu(queue);
        if !self.gradient_params.is_empty() {
            if self.gradient_params.len() > self.gradient_params_capacity {
                let new_capacity = self.gradient_params.len().next_power_of_two();
                self.gradient_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Path Gradient Params Storage Buffer"),
                    size: (new_capacity * std::mem::size_of::<GradientParams>()) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.gradient_params_capacity = new_capacity;
                if let (Some(view), Some(sampler)) = (
                    self.gradient_atlas.texture_view(),
                    self.gradient_atlas.sampler(),
                ) {
                    self.gradient_bind_group =
                        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Path Gradient Bind Group"),
                            layout: &self.gradient_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(sampler),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: self.gradient_params_buffer.as_entire_binding(),
                                },
                            ],
                        }));
                }
            }
            let params_bytes = bytemuck::cast_slice(&self.gradient_params);
            queue.write_buffer(&self.gradient_params_buffer, 0, params_bytes);
        }

        self.index_count = indices.len() as u32;
        if self.index_count == 0 {
            return;
        }

        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Path Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Path Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.vertex_capacity = vertices.len();
        self.index_capacity = indices.len();
    }

    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        globals_bind_group: &wgpu::BindGroup,
    ) {
        if self.index_count == 0 {
            return;
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, globals_bind_group, &[]);
        if let Some(ref gradient_bg) = self.gradient_bind_group {
            render_pass.set_bind_group(1, gradient_bg, &[]);
        }
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    fn triangle_path() -> VectorPath {
        let mut path = VectorPath::new();
        path.move_to(Vec2::new(0.0, 0.0));
        path.line_to(Vec2::new(100.0, 0.0));
        path.line_to(Vec2::new(50.0, 100.0));
        path.close();
        path
    }

    #[test]
    fn test_tessellate_fill_triangle_generates_mesh() {
        let mesh = tessellate_fill(&triangle_path()).expect("fill tessellation should succeed");
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0, "indices should form triangles");
    }

    #[test]
    fn test_tessellate_stroke_line_generates_mesh() {
        let mut path = VectorPath::new();
        path.move_to(Vec2::new(0.0, 0.0));
        path.line_to(Vec2::new(120.0, 0.0));

        let stroke = StrokeStyle::solid(
            style_engine::Paint::solid(glam::Vec4::new(1.0, 1.0, 1.0, 1.0)),
            4.0,
            style_engine::StrokeAlign::Center,
        );

        let mesh = tessellate_stroke(&path, &stroke).expect("stroke tessellation should succeed");
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn test_tessellate_fill_empty_path_is_empty_mesh() {
        let mesh = tessellate_fill(&VectorPath::new()).expect("empty path should not error");
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn test_tessellate_stroke_zero_weight_is_empty_mesh() {
        let path = triangle_path();
        let stroke = StrokeStyle::solid(
            style_engine::Paint::solid(glam::Vec4::new(1.0, 1.0, 1.0, 1.0)),
            0.0,
            style_engine::StrokeAlign::Center,
        );

        let mesh = tessellate_stroke(&path, &stroke).expect("zero-width stroke should not error");
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn test_tessellation_cache_reuses_fill_entry() {
        let mut cache = TessellationCache::new(8);
        let path = triangle_path();

        let first = cache
            .get_or_tessellate_fill(&path)
            .expect("first tessellation should succeed");
        let second = cache
            .get_or_tessellate_fill(&path)
            .expect("cached tessellation should succeed");

        assert_eq!(cache.len(), 1, "same path should produce one cache entry");
        assert!(
            Arc::ptr_eq(&first, &second),
            "cache hit should return same mesh allocation"
        );
        assert_eq!(first.indices.len(), second.indices.len());
        let stats = cache.stats();
        assert_eq!(stats.misses, 1, "first lookup should be a miss");
        assert_eq!(stats.hits, 1, "second lookup should be a hit");
    }

    #[test]
    fn test_tessellation_cache_stats_reset() {
        let mut cache = TessellationCache::new(8);
        let path = triangle_path();
        let _ = cache.get_or_tessellate_fill(&path);
        let _ = cache.get_or_tessellate_fill(&path);

        let before = cache.stats();
        assert!(before.hits > 0 || before.misses > 0);

        cache.reset_stats();
        assert_eq!(cache.stats(), TessellationCacheStats::default());
    }

    #[test]
    fn test_sample_paint_at_uv_linear_gradient_varies_across_uv() {
        let paint = Paint::Linear(style_engine::LinearGradient {
            start: glam::Vec2::new(0.0, 0.5),
            end: glam::Vec2::new(1.0, 0.5),
            stops: vec![
                style_engine::ColorStop::new(0.0, glam::Vec4::new(1.0, 0.0, 0.0, 1.0)),
                style_engine::ColorStop::new(1.0, glam::Vec4::new(0.0, 0.0, 1.0, 1.0)),
            ],
        });

        let left = sample_paint_at_uv(&paint, glam::Vec2::new(0.0, 0.5), glam::Vec2::ONE);
        let right = sample_paint_at_uv(&paint, glam::Vec2::new(1.0, 0.5), glam::Vec2::ONE);

        assert!(left.x > right.x, "left should be redder than right");
        assert!(right.z > left.z, "right should be bluer than left");
    }

    #[test]
    fn test_sample_paint_at_uv_image_fill_reads_registered_image() {
        use style_engine::{ImageFill, ImageId, ImageScaleMode};

        crate::backend::wgpu::image_store::register_image_rgba8(
            ImageId(99_001),
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, // row 0
                0, 0, 255, 255, 255, 255, 255, 255, // row 1
            ],
        )
        .expect("register image");

        let paint = Paint::Image(ImageFill {
            image_id: ImageId(99_001),
            scale_mode: ImageScaleMode::Fill,
            transform: None,
        });

        let top_left = sample_paint_at_uv(&paint, glam::Vec2::new(0.0, 0.0), glam::Vec2::ONE);
        assert!(top_left.x > 0.9);
        assert!(top_left.y < 0.1);
        assert!(top_left.z < 0.1);

        crate::backend::wgpu::image_store::unregister_image(ImageId(99_001));
    }

    #[test]
    fn test_append_batch_geometry_subdivides_image_batches_and_samples_interior() {
        use style_engine::{ImageFill, ImageId, ImageScaleMode};

        let image_id = ImageId(99_002);
        let width = 8u32;
        let height = 8u32;
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                let mut c = [0u8, 0u8, 0u8, 255u8];
                if x == 0 || y == 0 || x + 1 == width || y + 1 == height {
                    c = [255, 255, 255, 255];
                }
                if (3..=4).contains(&x) && (3..=4).contains(&y) {
                    c = [255, 0, 0, 255];
                }
                rgba[idx] = c[0];
                rgba[idx + 1] = c[1];
                rgba[idx + 2] = c[2];
                rgba[idx + 3] = c[3];
            }
        }

        crate::backend::wgpu::image_store::register_image_rgba8(image_id, width, height, rgba)
            .expect("register image");

        let mesh = Arc::new(PathMesh {
            vertices: vec![
                PathVertex {
                    position: [0.0, 0.0],
                    normal: [0.0, 0.0],
                },
                PathVertex {
                    position: [120.0, 0.0],
                    normal: [0.0, 0.0],
                },
                PathVertex {
                    position: [120.0, 120.0],
                    normal: [0.0, 0.0],
                },
                PathVertex {
                    position: [0.0, 120.0],
                    normal: [0.0, 0.0],
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        });

        let batch = PathBatch {
            mesh,
            paint: Paint::Image(ImageFill {
                image_id,
                scale_mode: ImageScaleMode::Fill,
                transform: None,
            }),
            opacity: 1.0,
            size: [120.0, 120.0],
            offset: [0.0, 0.0],
        };

        let mut gpu_vertices = Vec::new();
        let mut gpu_indices = Vec::new();
        append_batch_geometry(
            &batch,
            &mut gpu_vertices,
            &mut gpu_indices,
            PaintRuntime::default(),
        );

        assert!(
            gpu_vertices.len() > batch.mesh.vertices.len(),
            "image batches should be subdivided to increase sampling density"
        );
        assert!(
            gpu_indices.len() > batch.mesh.indices.len(),
            "image batches should emit denser triangle geometry"
        );

        let has_black = gpu_vertices
            .iter()
            .any(|v| v.color[0] < 0.15 && v.color[1] < 0.15 && v.color[2] < 0.15);
        let has_red = gpu_vertices
            .iter()
            .any(|v| v.color[0] > 0.85 && v.color[1] < 0.2 && v.color[2] < 0.2);
        assert!(has_black, "expected interior black texels to be sampled");
        assert!(has_red, "expected center red texels to be sampled");

        crate::backend::wgpu::image_store::unregister_image(image_id);
    }

    #[test]
    fn test_image_subdivision_steps_dense_for_scaled_images() {
        let steps = image_subdivision_steps(glam::Vec2::new(300.0, 170.0), 2);
        assert!(
            steps >= 64,
            "image batches should use dense subdivision for large scaled content, got {steps}"
        );
    }

    #[test]
    fn test_append_batch_geometry_defers_linear_gradient_to_shader_path() {
        let mesh = Arc::new(PathMesh {
            vertices: vec![
                PathVertex {
                    position: [0.0, 0.0],
                    normal: [0.0, 0.0],
                },
                PathVertex {
                    position: [100.0, 0.0],
                    normal: [0.0, 0.0],
                },
                PathVertex {
                    position: [0.0, 100.0],
                    normal: [0.0, 0.0],
                },
            ],
            indices: vec![0, 1, 2],
        });
        let batch = PathBatch {
            mesh,
            paint: Paint::Linear(style_engine::LinearGradient {
                start: glam::Vec2::new(0.0, 0.0),
                end: glam::Vec2::new(1.0, 0.0),
                stops: vec![
                    style_engine::ColorStop::new(0.0, glam::Vec4::new(1.0, 0.0, 0.0, 1.0)),
                    style_engine::ColorStop::new(1.0, glam::Vec4::new(0.0, 0.0, 1.0, 1.0)),
                ],
            }),
            opacity: 0.5,
            size: [100.0, 100.0],
            offset: [0.0, 0.0],
        };

        let mut gpu_vertices = Vec::new();
        let mut gpu_indices = Vec::new();
        append_batch_geometry(
            &batch,
            &mut gpu_vertices,
            &mut gpu_indices,
            PaintRuntime::default(),
        );

        assert_eq!(gpu_indices.len(), 3);
        assert!(!gpu_vertices.is_empty());
        for vertex in gpu_vertices {
            assert!(
                (vertex.color[0] - 1.0).abs() < 1e-6
                    && (vertex.color[1] - 1.0).abs() < 1e-6
                    && (vertex.color[2] - 1.0).abs() < 1e-6
                    && (vertex.color[3] - 0.5).abs() < 1e-6,
                "gradient vertices should keep neutral color for shader evaluation"
            );
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct PaintRuntime {
    fill_type: u32,
    gradient_index: u32,
}
