//! WGPU backend implementation.

pub mod context;
pub mod pipelines;

use crate::backend::text::TextRenderer;
use crate::{Color, RendererError, Scene, SceneNode};
use bevy_ecs::prelude::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use text_engine::{ShapedText, shape_text_parallel};
use tracing::{Level, instrument, span};

use context::WgpuContext;
pub use pipelines::primitive_pipeline::PrimitiveInstance;
use pipelines::primitive_pipeline::{
    FLAG_FILL_TYPE_MASK, GradientParams, PrimitivePipeline, create_primitive_instances,
    create_primitive_instances_with_pipeline,
};
use pipelines::path_pipeline::{
    PathBatch, PathPipeline, TessellationCache, tessellate_fill, tessellate_stroke,
};
pub use pipelines::path_pipeline::TessellationCacheStats;

/// Threshold for parallelizing text shaping
/// Below this count, sequential shaping is faster due to thread overhead
const TEXT_PARALLEL_THRESHOLD: usize = 8;

/// Text node data for shaping: (node, text, font_size, style)
type TextNodeData<'a> = (&'a SceneNode, &'a str, f32, &'a style_engine::VisualStyle);

const GLYPH_ATLAS_SIZE: u32 = 1024;
const GRADIENT_ATLAS_SIZE: f32 = 1024.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PathFingerprint {
    command_len: usize,
    winding: style_engine::WindingRule,
    first: u64,
    last: u64,
}

#[derive(Debug, Clone, Copy)]
struct InternedPathEntry {
    path_hash: u64,
    fingerprint: PathFingerprint,
}

#[derive(Debug, Default)]
struct PathInterner {
    by_ptr: HashMap<usize, InternedPathEntry>,
}

impl PathInterner {
    fn fingerprint(path: &style_engine::VectorPath) -> PathFingerprint {
        fn command_fp(command: style_engine::PathCommand) -> u64 {
            match command {
                style_engine::PathCommand::MoveTo(p) => {
                    0x01u64
                        ^ ((p.x.to_bits() as u64) << 1)
                        ^ ((p.y.to_bits() as u64).rotate_left(17))
                }
                style_engine::PathCommand::LineTo(p) => {
                    0x02u64
                        ^ ((p.x.to_bits() as u64) << 1)
                        ^ ((p.y.to_bits() as u64).rotate_left(17))
                }
                style_engine::PathCommand::QuadraticTo { control, to } => {
                    0x03u64
                        ^ ((control.x.to_bits() as u64) << 1)
                        ^ ((control.y.to_bits() as u64).rotate_left(9))
                        ^ ((to.x.to_bits() as u64).rotate_left(17))
                        ^ ((to.y.to_bits() as u64).rotate_left(29))
                }
                style_engine::PathCommand::CubicTo {
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
                style_engine::PathCommand::Close => 0x05u64,
            }
        }

        let first = path.commands.first().copied().map(command_fp).unwrap_or(0);
        let last = path.commands.last().copied().map(command_fp).unwrap_or(0);
        PathFingerprint {
            command_len: path.commands.len(),
            winding: path.winding_rule,
            first,
            last,
        }
    }

    fn hash_for(&mut self, path: &style_engine::VectorPath) -> u64 {
        let ptr = path as *const style_engine::VectorPath as usize;
        let fingerprint = Self::fingerprint(path);
        if let Some(entry) = self.by_ptr.get(&ptr) {
            if entry.fingerprint == fingerprint {
                return entry.path_hash;
            }
        }

        let path_hash = TessellationCache::fill_key(path);
        self.by_ptr.insert(
            ptr,
            InternedPathEntry {
                path_hash,
                fingerprint,
            },
        );
        path_hash
    }
}

#[derive(Clone, Copy)]
enum TextFill {
    Solid(glam::Vec4),
    Gradient {
        param_index: u32,
        fill_type: u32,
        opacity: f32,
        text_bounds: [f32; 4], // x, y, width, height in scene space
    },
}

fn apply_text_fill_to_glyph(instance: &mut PrimitiveInstance, fill: TextFill) {
    match fill {
        TextFill::Solid(color) => {
            instance.color = color.to_array();
            instance.flags &= !0xF;
        }
        TextFill::Gradient {
            param_index,
            fill_type,
            opacity,
            text_bounds,
        } => {
            instance.color = [1.0, 1.0, 1.0, opacity];
            instance.gradient_params = [param_index as f32, text_bounds[0], text_bounds[1], text_bounds[2]];
            instance.stroke_params = [text_bounds[3], 0.0];
            instance.flags = (instance.flags & !FLAG_FILL_TYPE_MASK) | (fill_type & FLAG_FILL_TYPE_MASK);
        }
    }
}

/// Helper to create PrimitiveInstances from a SceneNode (ECS compatibility).
///
/// Uses the no-pipeline fallback path:
/// - solid fills are emitted directly
/// - gradients are approximated to a representative color
/// - strokes and drop shadows are emitted
/// - text is skipped (text shaping requires backend-owned text/glyph resources)
///
/// Returns empty vec if the node is invisible or has no styled content.
pub fn create_node_instances(node: &SceneNode) -> Vec<PrimitiveInstance> {
    use crate::NodeContent;
    use pipelines::primitive_pipeline::create_primitive_instances;

    if !node.visible || node.opacity <= 0.0 {
        return Vec::new();
    }

    match &node.content {
        NodeContent::Styled { style } => {
            let pos = glam::Vec2::new(node.bounds.x, node.bounds.y);
            let size = glam::Vec2::new(node.bounds.width, node.bounds.height);
            create_primitive_instances(style, pos, size, node.opacity)
        }
        NodeContent::Empty => Vec::new(),
    }
}

/// wgpu-based rendering backend.
#[derive(Resource)]
pub struct WgpuBackend {
    pub(crate) context: WgpuContext,
    primitive_pipeline: PrimitivePipeline,
    path_pipeline: PathPipeline,
    tessellation_cache: TessellationCache,
    path_interner: PathInterner,
    text_renderer: TextRenderer,
    glyph_texture: wgpu::Texture,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PathCacheWarmupReport {
    pub fill_paths: u64,
    pub stroke_paths: u64,
    pub warmed_meshes: u64,
    pub failed: u64,
}

impl WgpuBackend {
    /// Create a new wgpu backend from a window.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the created `WgpuBackend` is dropped *before* the window
    /// it was created from. This is required because `wgpu::Surface` holds a reference to the
    /// window handle, but carries a `'static` lifetime. Accessing the surface after the window
    /// is destroyed results in undefined behavior.
    #[instrument(skip(window), fields(width, height, composition_mode))]
    pub unsafe fn new<W>(
        window: &W,
        width: u32,
        height: u32,
        composition_mode: bool,
    ) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle + Sync,
    {
        // SAFETY: Propagating the safety requirement to the caller.
        let context = unsafe { WgpuContext::new(window, width, height, composition_mode)? };

        let text_renderer = TextRenderer::new();

        // Create glyph atlas texture for PrimitivePipeline
        const ATLAS_SIZE: u32 = 1024;
        let glyph_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas Texture"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let glyph_texture_view = glyph_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let glyph_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Glyph Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let primitive_pipeline = PrimitivePipeline::new(
            &context.device,
            &context.globals_bind_group_layout,
            &glyph_texture_view,
            &glyph_sampler,
            context.config.format,
        );
        let path_pipeline = PathPipeline::new(
            &context.device,
            &context.globals_bind_group_layout,
            context.config.format,
        );

        Ok(Self {
            context,
            primitive_pipeline,
            path_pipeline,
            tessellation_cache: TessellationCache::new(2048),
            path_interner: PathInterner::default(),
            text_renderer,
            glyph_texture,
        })
    }

    /// Pre-tessellate visible path geometry for the current scene into the cache.
    ///
    /// Call this after large scene/style loads to bias toward cache-hit rendering.
    pub fn warm_path_cache(&mut self, scene: &Scene) -> PathCacheWarmupReport {
        use crate::NodeContent;
        let mut report = PathCacheWarmupReport::default();

        for (_node_id, node) in scene.iter_visuals() {
            if !node.visible || node.opacity <= 0.0 {
                continue;
            }

            let NodeContent::Styled { style } = &node.content else {
                continue;
            };

            if let Some(paths) = &style.fill_geometry {
                for path in paths {
                    report.fill_paths = report.fill_paths.saturating_add(1);
                    let key = self.path_interner.hash_for(path);
                    match self
                        .tessellation_cache
                        .get_or_tessellate_fill_with_key(key, path)
                    {
                        Ok(mesh) => {
                            if !mesh.indices.is_empty() {
                                report.warmed_meshes = report.warmed_meshes.saturating_add(1);
                            }
                        }
                        Err(_) => {
                            report.failed = report.failed.saturating_add(1);
                        }
                    }
                }
            }

            if let Some(stroke) = &style.stroke {
                if let Some(stroke_paths) = style.stroke_geometry.as_ref().or(style.fill_geometry.as_ref()) {
                    for path in stroke_paths {
                        report.stroke_paths = report.stroke_paths.saturating_add(1);
                        let path_hash = self.path_interner.hash_for(path);
                        let key = TessellationCache::stroke_key_from_path_hash(path_hash, stroke);
                        match self
                            .tessellation_cache
                            .get_or_tessellate_stroke_with_key(key, path, stroke)
                        {
                            Ok(mesh) => {
                                if !mesh.indices.is_empty() {
                                    report.warmed_meshes = report.warmed_meshes.saturating_add(1);
                                }
                            }
                            Err(_) => {
                                report.failed = report.failed.saturating_add(1);
                            }
                        }
                    }
                }
            }
        }

        report
    }

    pub fn tessellation_cache_stats(&self) -> TessellationCacheStats {
        self.tessellation_cache.stats()
    }

    pub fn reset_tessellation_cache_stats(&mut self) {
        self.tessellation_cache.reset_stats();
    }

    /// Render a collection of primitive instances directly (ECS-friendly API)
    #[instrument(skip(self, instances))]
    pub fn render_instances(
        &mut self,
        instances: &[PrimitiveInstance],
    ) -> Result<(), RendererError> {
        let _span = span!(Level::TRACE, "render_instances").entered();

        self.primitive_pipeline
            .prepare(&self.context.device, &self.context.queue, instances);
        self.path_pipeline
            .prepare(&self.context.device, &self.context.queue, &[]);

        let WgpuBackend {
            context,
            primitive_pipeline,
            path_pipeline,
            ..
        } = self;

        context.with_render_pass(|render_pass, globals_bind_group| {
            primitive_pipeline.render(render_pass, globals_bind_group, instances.len() as u32);
            path_pipeline.render(render_pass, globals_bind_group);
        })
    }

    /// Collect instances from the scene.
    ///
    /// Returns a tuple of (primitive_instances, text_nodes_for_shaping, path_batches).
    /// Text nodes are extracted from VisualStyle and returned for shaping.
    fn collect_instances<'a>(
        pipeline: &mut PrimitivePipeline,
        tessellation_cache: &mut TessellationCache,
        path_interner: &mut PathInterner,
        scene: &'a Scene,
    ) -> (Vec<PrimitiveInstance>, Vec<TextNodeData<'a>>, Vec<PathBatch>) {
        Self::collect_instances_impl(Some(pipeline), Some(tessellation_cache), Some(path_interner), scene)
    }

    #[cfg(test)]
    fn collect_instances_for_tests<'a>(
        scene: &'a Scene,
    ) -> (Vec<PrimitiveInstance>, Vec<TextNodeData<'a>>, Vec<PathBatch>) {
        Self::collect_instances_impl(None, None, None, scene)
    }

    fn collect_instances_impl<'a>(
        mut pipeline: Option<&mut PrimitivePipeline>,
        mut tessellation_cache: Option<&mut TessellationCache>,
        mut path_interner: Option<&mut PathInterner>,
        scene: &'a Scene,
    ) -> (Vec<PrimitiveInstance>, Vec<TextNodeData<'a>>, Vec<PathBatch>) {
        use crate::NodeContent;
        let mut instances = Vec::new();
        let mut text_nodes_for_shaping = Vec::new();
        let mut path_batches = Vec::new();

        for (_node_id, node) in scene.iter_visuals() {
            if !node.visible || node.opacity <= 0.0 {
                continue;
            }

            if let NodeContent::Styled { style } = &node.content {
                if let Some(paths) = &style.fill_geometry {
                    let fill_paint = resolve_path_fill_paint(style);

                    for path in paths {
                        let path_hash = if let Some(interner) = path_interner.as_deref_mut() {
                            interner.hash_for(path)
                        } else {
                            TessellationCache::fill_key(path)
                        };
                        let mesh_result = if let Some(cache) = tessellation_cache.as_deref_mut() {
                            cache.get_or_tessellate_fill_with_key(path_hash, path)
                        } else {
                            tessellate_fill(path).map(Arc::new)
                        };

                        if let Ok(mesh) = mesh_result {
                            if !mesh.indices.is_empty() {
                                path_batches.push(PathBatch {
                                    mesh,
                                    paint: fill_paint.clone(),
                                    opacity: node.opacity,
                                    size: [node.bounds.width, node.bounds.height],
                                    offset: [node.bounds.x, node.bounds.y],
                                });
                            }
                        }
                    }

                    if let Some(stroke) = &style.stroke {
                        let stroke_paint = resolve_path_stroke_paint(stroke);
                        if let Some(stroke_paths) = style.stroke_geometry.as_ref().or(style.fill_geometry.as_ref()) {
                            for path in stroke_paths {
                                let path_hash = if let Some(interner) = path_interner.as_deref_mut() {
                                    interner.hash_for(path)
                                } else {
                                    TessellationCache::fill_key(path)
                                };
                                let stroke_key = TessellationCache::stroke_key_from_path_hash(path_hash, stroke);
                                let mesh_result = if let Some(cache) = tessellation_cache.as_deref_mut() {
                                    cache.get_or_tessellate_stroke_with_key(stroke_key, path, stroke)
                                } else {
                                    tessellate_stroke(path, stroke).map(Arc::new)
                                };
                                if let Ok(mesh) = mesh_result {
                                    if !mesh.indices.is_empty() {
                                        path_batches.push(PathBatch {
                                            mesh,
                                            paint: stroke_paint.clone(),
                                            opacity: node.opacity,
                                            size: [node.bounds.width, node.bounds.height],
                                            offset: [node.bounds.x, node.bounds.y],
                                        });
                                    }
                                }
                            }
                        }
                    }
                    continue;
                }

                // Check if this style has text that needs shaping
                if let Some(text_content) = &style.text {
                    text_nodes_for_shaping.push((
                        node,
                        text_content.text.as_str(),
                        text_content.font_size,
                        style.as_ref(),
                    ));
                }

                // Create primitive instances for this style
                let pos = glam::Vec2::new(node.bounds.x, node.bounds.y);
                let size = glam::Vec2::new(node.bounds.width, node.bounds.height);
                let node_instances = if let Some(pipeline) = pipeline.as_deref_mut() {
                    create_primitive_instances_with_pipeline(pipeline, style, pos, size, node.opacity)
                } else {
                    create_primitive_instances(style, pos, size, node.opacity)
                };
                instances.extend(node_instances);
            }
        }
        (instances, text_nodes_for_shaping, path_batches)
    }

    fn resolve_text_fill(
        pipeline: &mut PrimitivePipeline,
        style: &style_engine::VisualStyle,
        opacity: f32,
        text_bounds: [f32; 4],
    ) -> TextFill {
        let Some(fill) = style.fills.first() else {
            return TextFill::Solid(glam::Vec4::new(0.0, 0.0, 0.0, opacity));
        };

        match fill {
            style_engine::Paint::Solid(color) => {
                let mut final_color = *color;
                final_color.w *= opacity;
                TextFill::Solid(final_color)
            }
            style_engine::Paint::Linear(gradient) => {
                let atlas_row = pipeline.add_gradient(&gradient.stops);
                let param_index = pipeline.add_gradient_params(GradientParams {
                    start: gradient.start.to_array(),
                    end: gradient.end.to_array(),
                    atlas_row: (atlas_row as f32 + 0.5) / GRADIENT_ATLAS_SIZE,
                    gradient_type: 0,
                    _padding: [0.0; 2],
                });
                TextFill::Gradient {
                    param_index,
                    fill_type: 1,
                    opacity,
                    text_bounds,
                }
            }
            style_engine::Paint::Radial(gradient) => {
                let atlas_row = pipeline.add_gradient(&gradient.stops);
                let param_index = pipeline.add_gradient_params(GradientParams {
                    start: gradient.center.to_array(),
                    end: (gradient.center + glam::Vec2::new(gradient.radius, 0.0)).to_array(),
                    atlas_row: (atlas_row as f32 + 0.5) / GRADIENT_ATLAS_SIZE,
                    gradient_type: 1,
                    _padding: [0.0; 2],
                });
                TextFill::Gradient {
                    param_index,
                    fill_type: 2,
                    opacity,
                    text_bounds,
                }
            }
            style_engine::Paint::Angular(gradient) => {
                let atlas_row = pipeline.add_gradient(&gradient.stops);
                let param_index = pipeline.add_gradient_params(GradientParams {
                    start: gradient.center.to_array(),
                    end: gradient.center.to_array(),
                    atlas_row: (atlas_row as f32 + 0.5) / GRADIENT_ATLAS_SIZE,
                    gradient_type: 2,
                    _padding: [0.0; 2],
                });
                TextFill::Gradient {
                    param_index,
                    fill_type: 3,
                    opacity,
                    text_bounds,
                }
            }
            style_engine::Paint::Diamond(gradient) => {
                let atlas_row = pipeline.add_gradient(&gradient.stops);
                let param_index = pipeline.add_gradient_params(GradientParams {
                    start: gradient.center.to_array(),
                    end: (gradient.center + glam::Vec2::new(gradient.scale, gradient.scale))
                        .to_array(),
                    atlas_row: (atlas_row as f32 + 0.5) / GRADIENT_ATLAS_SIZE,
                    gradient_type: 3,
                    _padding: [0.0; 2],
                });
                TextFill::Gradient {
                    param_index,
                    fill_type: 4,
                    opacity,
                    text_bounds,
                }
            }
            style_engine::Paint::Image(_) => TextFill::Solid(glam::Vec4::new(0.0, 0.0, 0.0, opacity)),
        }
    }
}

fn resolve_path_fill_paint(style: &style_engine::VisualStyle) -> style_engine::Paint {
    style
        .fills
        .first()
        .cloned()
        .unwrap_or_else(|| style_engine::Paint::solid(glam::Vec4::new(1.0, 0.0, 1.0, 1.0)))
}

fn resolve_path_stroke_paint(stroke: &style_engine::StrokeStyle) -> style_engine::Paint {
    stroke
        .top_paint()
        .cloned()
        .unwrap_or_else(|| style_engine::Paint::solid(glam::Vec4::new(1.0, 0.0, 1.0, 1.0)))
}

impl super::RenderBackend for WgpuBackend {
    #[instrument(skip(self, scene))]
    fn render(&mut self, scene: &Scene) -> Result<(), RendererError> {
        let _span = span!(Level::TRACE, "render_frame").entered();

        // Clear per-frame gradient data
        self.primitive_pipeline.clear_gradient_params();

        let (mut instances, raw_text_nodes, path_batches) = Self::collect_instances(
            &mut self.primitive_pipeline,
            &mut self.tessellation_cache,
            &mut self.path_interner,
            scene,
        );

        // Process text nodes
        if !raw_text_nodes.is_empty() {
            // Step 1: Shape text in parallel if above threshold
            // Shaping is CPU-intensive and read-only (uses thread-local FontSystem)
            let shaped_results: Vec<(
                glam::Vec2,
                f32,
                [f32; 4],
                &style_engine::VisualStyle,
                ShapedText,
            )> = if raw_text_nodes.len()
                >= TEXT_PARALLEL_THRESHOLD
            {
                // Parallel shaping
                raw_text_nodes
                    .par_iter()
                    .filter(|(_, text, _, _)| !text.is_empty())
                    .map(|(node, text, font_size, style)| {
                        let shaped = shape_text_parallel(text, *font_size);
                        let position = glam::Vec2::new(node.bounds.x, node.bounds.y + font_size);
                        let text_bounds = [node.bounds.x, node.bounds.y, node.bounds.width, node.bounds.height];
                        (position, node.opacity, text_bounds, *style, shaped)
                    })
                    .collect()
            } else {
                // Sequential shaping for small counts
                raw_text_nodes
                    .iter()
                    .filter(|(_, text, _, _)| !text.is_empty())
                    .map(|(node, text, font_size, style)| {
                        let shaped = self
                            .text_renderer
                            .text_engine_mut()
                            .shape_text(text, *font_size);
                        let position = glam::Vec2::new(node.bounds.x, node.bounds.y + font_size);
                        let text_bounds = [node.bounds.x, node.bounds.y, node.bounds.width, node.bounds.height];
                        (position, node.opacity, text_bounds, *style, shaped)
                    })
                    .collect()
            };

            // Step 2: Generate glyph instances and add to primitives
            for (position, opacity, text_bounds, style, shaped) in shaped_results {
                let fill = Self::resolve_text_fill(
                    &mut self.primitive_pipeline,
                    style,
                    opacity,
                    text_bounds,
                );
                let glyph_instances = self
                    .text_renderer
                    .generate_instances(&shaped, position, glam::Vec4::ONE);

                // Apply text fill metadata to generated glyph primitive instances
                for mut instance in glyph_instances {
                    apply_text_fill_to_glyph(&mut instance, fill);
                    instances.push(instance);
                }
            }

            // Update glyph atlas texture (atlas is always 1024x1024)
            self.context.queue.write_texture(
                self.glyph_texture.as_image_copy(),
                self.text_renderer.atlas().texture_data(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(GLYPH_ATLAS_SIZE),
                    rows_per_image: Some(GLYPH_ATLAS_SIZE),
                },
                wgpu::Extent3d {
                    width: GLYPH_ATLAS_SIZE,
                    height: GLYPH_ATLAS_SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.primitive_pipeline
            .prepare(&self.context.device, &self.context.queue, &instances);
        self.path_pipeline
            .prepare(&self.context.device, &self.context.queue, &path_batches);

        let WgpuBackend {
            context,
            primitive_pipeline,
            path_pipeline,
            ..
        } = self;

        context.with_render_pass(|render_pass, globals_bind_group| {
            primitive_pipeline.render(render_pass, globals_bind_group, instances.len() as u32);
            path_pipeline.render(render_pass, globals_bind_group);
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.context.resize(width, height);
    }

    fn set_clear_color(&mut self, color: Color) {
        self.context.clear_color = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeContent, SceneNode, Transform2D};

    #[test]
    fn test_instance_collection_from_scene() {
        // Create a test scene with rectangles
        let mut scene = Scene::new();
        let root = scene.root();

        // Add visible rectangle
        let red_rect = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    style_engine::VisualStyle::new()
                        .solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4()),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 200.0,
            },
            children: vec![],
            parent: None, // Set by add_node
            visible: true,
            opacity: 1.0,
        };
        scene.add_node(root, red_rect);

        // Add invisible rectangle (should be skipped)
        let invisible_rect = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    style_engine::VisualStyle::new()
                        .solid_fill(Color::rgba(0.0, 1.0, 0.0, 1.0).as_vec4()),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            },
            children: vec![],
            parent: None, // Set by add_node
            visible: false,
            opacity: 1.0,
        };
        scene.add_node(root, invisible_rect);

        // Add rectangle with opacity
        let blue_rect = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    style_engine::VisualStyle::new()
                        .solid_fill(Color::rgba(0.0, 0.0, 1.0, 1.0).as_vec4()),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 200.0,
                y: 100.0,
                width: 150.0,
                height: 150.0,
            },
            children: vec![],
            parent: None, // Set by add_node
            visible: true,
            opacity: 0.5,
        };
        scene.add_node(root, blue_rect);

        // Collect instances
        let (instances, _, _) = WgpuBackend::collect_instances_for_tests(&scene);

        // Should have 2 instances (red and blue, not invisible)
        assert_eq!(instances.len(), 2);

        // Find red rectangle instance
        let red_instance = instances
            .iter()
            .find(|i| i.color[0] > 0.9 && i.color[1] < 0.1)
            .expect("Red instance not found");
        assert_eq!(red_instance.pos, [10.0, 20.0]);
        assert_eq!(red_instance.size, [100.0, 200.0]);
        assert_eq!(red_instance.color, [1.0, 0.0, 0.0, 1.0]);

        // Find blue rectangle instance (with opacity applied)
        let blue_instance = instances
            .iter()
            .find(|i| i.color[2] > 0.9 && i.color[0] < 0.1)
            .expect("Blue instance not found");
        assert_eq!(blue_instance.pos, [200.0, 100.0]);
        assert_eq!(blue_instance.size, [150.0, 150.0]);
        assert_eq!(blue_instance.color, [0.0, 0.0, 1.0, 0.5]); // opacity applied

        // Both flat rects should have corner_radii = 0.0
        assert_eq!(red_instance.corner_radii, [0.0; 4]);
        assert_eq!(blue_instance.corner_radii, [0.0; 4]);
    }

    #[test]
    fn test_rounded_rect_preserves_corner_radius() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Add a RoundedRect with corner_radius
        let rounded = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    style_engine::VisualStyle::new()
                        .solid_fill(Color::rgba(0.5, 0.5, 0.5, 1.0).as_vec4())
                        .corner_radius(12.0),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 30.0,
                y: 30.0,
                width: 200.0,
                height: 100.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        scene.add_node(root, rounded);

        // Add a flat Rect for comparison
        let flat = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    style_engine::VisualStyle::new()
                        .solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4()),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        scene.add_node(root, flat);

        let (instances, _, _) = WgpuBackend::collect_instances_for_tests(&scene);
        assert_eq!(instances.len(), 2);

        // Rounded rect should preserve corner_radius (uniform radii)
        let rounded_inst = instances
            .iter()
            .find(|i| i.corner_radii[0] > 0.0)
            .expect("Rounded instance not found");
        assert_eq!(rounded_inst.corner_radii, [12.0; 4]); // uniform radii
        assert_eq!(rounded_inst.pos, [30.0, 30.0]);

        // Flat rect should have 0.0
        let flat_inst = instances
            .iter()
            .find(|i| i.color[0] > 0.9)
            .expect("Flat instance not found");
        assert_eq!(flat_inst.corner_radii, [0.0; 4]);
    }

    #[test]
    fn test_text_parallel_shaping_below_threshold() {
        // Create scene with text nodes below parallel threshold
        let mut scene = Scene::new();
        let root = scene.root();

        // Add 4 text nodes (below TEXT_PARALLEL_THRESHOLD of 8)
        for i in 0..4 {
            let text_node = SceneNode {
                content: NodeContent::Styled {
                    style: Box::new(
                        crate::VisualStyle::new()
                            .solid_fill(Color::rgba(1.0, 1.0, 1.0, 1.0).as_vec4())
                            .text(crate::TextContent::new(format!("Text {}", i), 16.0)),
                    ),
                },
                transform: Transform2D::identity(),
                bounds: plat_core::Rect {
                    x: 0.0,
                    y: (i * 20) as f32,
                    width: 100.0,
                    height: 20.0,
                },
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            };
            scene.add_node(root, text_node);
        }

        let (_, text_nodes, _) = WgpuBackend::collect_instances_for_tests(&scene);
        assert_eq!(text_nodes.len(), 4);
        // Sequential path should be taken
    }

    #[test]
    fn test_text_parallel_shaping_above_threshold() {
        // Create scene with text nodes above parallel threshold
        let mut scene = Scene::new();
        let root = scene.root();

        // Add 10 text nodes (above TEXT_PARALLEL_THRESHOLD of 8)
        for i in 0..10 {
            let text_node = SceneNode {
                content: NodeContent::Styled {
                    style: Box::new(
                        crate::VisualStyle::new()
                            .solid_fill(Color::rgba(1.0, 1.0, 1.0, 1.0).as_vec4())
                            .text(crate::TextContent::new(format!("Text {}", i), 16.0)),
                    ),
                },
                transform: Transform2D::identity(),
                bounds: plat_core::Rect {
                    x: 0.0,
                    y: (i * 20) as f32,
                    width: 100.0,
                    height: 20.0,
                },
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            };
            scene.add_node(root, text_node);
        }

        let (_, text_nodes, _) = WgpuBackend::collect_instances_for_tests(&scene);
        assert_eq!(text_nodes.len(), 10);
        // Parallel path should be taken
    }

    #[test]
    fn test_text_shaping_with_empty_strings() {
        // Create scene with mix of empty and non-empty text
        let mut scene = Scene::new();
        let root = scene.root();

        for i in 0..5 {
            let text = if i % 2 == 0 {
                String::new()
            } else {
                format!("Text {}", i)
            };

            let text_node = SceneNode {
                content: NodeContent::Styled {
                    style: Box::new(
                        crate::VisualStyle::new()
                            .solid_fill(Color::rgba(1.0, 1.0, 1.0, 1.0).as_vec4())
                            .text(crate::TextContent::new(text, 16.0)),
                    ),
                },
                transform: Transform2D::identity(),
                bounds: plat_core::Rect {
                    x: 0.0,
                    y: (i * 20) as f32,
                    width: 100.0,
                    height: 20.0,
                },
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            };
            scene.add_node(root, text_node);
        }

        let (_, text_nodes, _) = WgpuBackend::collect_instances_for_tests(&scene);
        assert_eq!(text_nodes.len(), 5);
        // Empty strings should be filtered out during shaping
    }

    #[test]
    fn test_apply_text_fill_solid_sets_color_and_solid_flag() {
        let mut instance =
            PrimitiveInstance::glyph([0.0, 0.0], [10.0, 12.0], [1.0, 1.0, 1.0, 1.0], [0.0; 4]);
        let color = glam::Vec4::new(0.2, 0.4, 0.6, 0.8);

        apply_text_fill_to_glyph(&mut instance, TextFill::Solid(color));

        assert_eq!(instance.color, color.to_array());
        assert_eq!(instance.flags & FLAG_FILL_TYPE_MASK, 0, "solid fill type bits should be 0");
    }

    #[test]
    fn test_apply_text_fill_gradient_sets_flag_and_gradient_index() {
        let mut instance =
            PrimitiveInstance::glyph([0.0, 0.0], [10.0, 12.0], [1.0, 1.0, 1.0, 1.0], [0.0; 4]);

        apply_text_fill_to_glyph(
            &mut instance,
            TextFill::Gradient {
                param_index: 42,
                fill_type: 3,
                opacity: 0.65,
                text_bounds: [10.0, 20.0, 100.0, 40.0],
            },
        );

        assert_eq!(instance.flags & FLAG_FILL_TYPE_MASK, 3, "gradient fill type bits should be set");
        assert_eq!(instance.gradient_params[0], 42.0);
        assert_eq!(instance.gradient_params[1], 10.0);
        assert_eq!(instance.gradient_params[2], 20.0);
        assert_eq!(instance.gradient_params[3], 100.0);
        assert_eq!(instance.stroke_params[0], 40.0);
        assert!((instance.color[3] - 0.65).abs() < 1e-6);
        assert_eq!(instance.corner_radii, [0.0; 4]);
    }

    #[test]
    fn test_create_node_instances_emits_fill_stroke_and_shadow_for_non_text_style() {
        let node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    crate::VisualStyle::new()
                        .fill(style_engine::Paint::Linear(style_engine::LinearGradient {
                            start: glam::Vec2::new(0.0, 0.5),
                            end: glam::Vec2::new(1.0, 0.5),
                            stops: vec![
                                style_engine::ColorStop::new(
                                    0.0,
                                    glam::Vec4::new(1.0, 0.0, 0.0, 1.0),
                                ),
                                style_engine::ColorStop::new(
                                    1.0,
                                    glam::Vec4::new(0.0, 0.0, 1.0, 1.0),
                                ),
                            ],
                        }))
                        .stroke(style_engine::StrokeStyle::solid(
                            style_engine::Paint::solid(glam::Vec4::new(1.0, 1.0, 1.0, 1.0)),
                            2.0,
                            style_engine::StrokeAlign::Inside,
                        ))
                        .drop_shadow(
                            glam::Vec2::new(2.0, 2.0),
                            6.0,
                            glam::Vec4::new(0.0, 0.0, 0.0, 0.3),
                        ),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };

        let instances = create_node_instances(&node);

        assert_eq!(instances.len(), 3, "expected shadow + fill + stroke");
        assert!(
            instances
                .iter()
                .any(|i| (i.flags & pipelines::primitive_pipeline::FLAG_IS_SHADOW) != 0),
            "expected one shadow instance"
        );
        assert!(
            instances.iter().any(|i| (i.flags & (1 << 4)) != 0),
            "expected one stroke instance"
        );
    }

    #[test]
    fn test_collect_instances_routes_fill_geometry_to_path_batches() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut path = style_engine::VectorPath::new();
        path.move_to(glam::Vec2::new(0.0, 0.0));
        path.line_to(glam::Vec2::new(80.0, 0.0));
        path.line_to(glam::Vec2::new(40.0, 60.0));
        path.close();

        let node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    crate::VisualStyle::new()
                        .solid_fill(glam::Vec4::new(0.2, 0.8, 0.4, 1.0))
                        .fill_geometry(vec![path]),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 100.0,
                y: 120.0,
                width: 80.0,
                height: 60.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 0.75,
        };
        scene.add_node(root, node);

        let (instances, _text_nodes, path_batches) = WgpuBackend::collect_instances_for_tests(&scene);
        assert!(instances.is_empty(), "path-only node should not emit primitive rect instances");
        assert_eq!(path_batches.len(), 1, "expected one tessellated path batch");
        assert!(!path_batches[0].mesh.indices.is_empty());
    }

    #[test]
    fn test_collect_instances_adds_stroke_geometry_batches() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut path = style_engine::VectorPath::new();
        path.move_to(glam::Vec2::new(0.0, 0.0));
        path.line_to(glam::Vec2::new(60.0, 0.0));
        path.line_to(glam::Vec2::new(30.0, 40.0));
        path.close();

        let node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    crate::VisualStyle::new()
                        .solid_fill(glam::Vec4::new(0.2, 0.8, 0.4, 1.0))
                        .fill_geometry(vec![path])
                        .stroke(style_engine::StrokeStyle::solid(
                            style_engine::Paint::solid(glam::Vec4::new(1.0, 1.0, 1.0, 1.0)),
                            3.0,
                            style_engine::StrokeAlign::Center,
                        )),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 50.0,
                y: 50.0,
                width: 60.0,
                height: 40.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        scene.add_node(root, node);

        let (instances, _text_nodes, path_batches) = WgpuBackend::collect_instances_for_tests(&scene);
        assert!(instances.is_empty(), "geometry node should route through path batches");
        assert_eq!(path_batches.len(), 2, "expected fill + stroke path batches");
    }
}
