//! WGPU backend implementation.

pub mod context;
pub mod effects;
pub mod image_store;
pub mod pipelines;
pub mod render_target_pool;

use crate::backend::text::TextRenderer;
use crate::{Color, RendererError, Scene, SceneNode};
#[cfg(not(target_arch = "wasm32"))]
use bevy_ecs::prelude::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use text_engine::{ShapedText, shape_text_parallel};
use tracing::{Level, instrument, span};

use context::WgpuContext;
use effects::{
    EffectPassKind, RenderTargetHandle, RenderTargetKey, RenderTargetPool, backdrop_capture_bounds,
    classify_effect_passes,
};
use pipelines::blend_pipeline::{BlendParams, BlendPipeline};
use pipelines::blur_pipeline::{BlurDirection, BlurParams, BlurPipeline, select_blur_tier};
pub use pipelines::path_pipeline::TessellationCacheStats;
use pipelines::path_pipeline::{
    PathBatch, PathPipeline, TessellationCache, tessellate_fill, tessellate_stroke,
};
pub use pipelines::primitive_pipeline::PrimitiveInstance;
use pipelines::primitive_pipeline::{
    FLAG_FILL_TYPE_MASK, GradientParams, PrimitivePipeline, create_primitive_instances,
    create_primitive_instances_with_pipeline,
};
use pipelines::stencil_pipeline::{ClipStack, plan_clip_sequence_for_nested_clips};

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
        if let Some(entry) = self.by_ptr.get(&ptr)
            && entry.fingerprint == fingerprint
        {
            return entry.path_hash;
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
            instance.gradient_params = [
                param_index as f32,
                text_bounds[0],
                text_bounds[1],
                text_bounds[2],
            ];
            instance.stroke_params = [text_bounds[3], 0.0];
            instance.flags =
                (instance.flags & !FLAG_FILL_TYPE_MASK) | (fill_type & FLAG_FILL_TYPE_MASK);
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
            create_primitive_instances(style, pos, size, node.opacity * style.opacity)
        }
        NodeContent::Empty => Vec::new(),
    }
}

/// wgpu-based rendering backend.
#[cfg_attr(not(target_arch = "wasm32"), derive(Resource))]
pub struct WgpuBackend {
    pub(crate) context: WgpuContext,
    primitive_pipeline: PrimitivePipeline,
    path_pipeline: PathPipeline,
    #[allow(dead_code)]
    blur_pipeline: BlurPipeline,
    #[allow(dead_code)]
    blend_pipeline: BlendPipeline,
    tessellation_cache: TessellationCache,
    path_interner: PathInterner,
    text_renderer: TextRenderer,
    glyph_texture: wgpu::Texture,
    effect_target_pool: RenderTargetPool,
    clip_stack: ClipStack,
    effect_sampler: wgpu::Sampler,
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
    #[cfg(not(target_arch = "wasm32"))]
    pub unsafe fn new<W>(
        window: &W,
        width: u32,
        height: u32,
        composition_mode: bool,
    ) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle,
    {
        // SAFETY: Propagating the safety requirement to the caller.
        let context = unsafe { WgpuContext::new(window, width, height, composition_mode)? };
        Self::from_context(context)
    }

    /// Create a new wgpu backend from a window asynchronously on wasm targets.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the created `WgpuBackend` is dropped *before* the window
    /// it was created from. This is required because `wgpu::Surface` holds a reference to the
    /// window handle, but carries a `'static` lifetime. Accessing the surface after the window
    /// is destroyed results in undefined behavior.
    #[instrument(skip(window), fields(width, height, composition_mode))]
    #[cfg(target_arch = "wasm32")]
    pub async unsafe fn new_async<W>(
        window: &W,
        width: u32,
        height: u32,
        composition_mode: bool,
    ) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle,
    {
        // SAFETY: Propagating the safety requirement to the caller.
        let context =
            unsafe { WgpuContext::new_async(window, width, height, composition_mode).await? };
        Self::from_context(context)
    }

    fn from_context(context: WgpuContext) -> Result<Self, RendererError> {
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
        let blur_pipeline = BlurPipeline::new(&context.device, context.config.format);
        let blend_pipeline = BlendPipeline::new(&context.device, context.config.format);
        let effect_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Effect Pipeline Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        Ok(Self {
            context,
            primitive_pipeline,
            path_pipeline,
            blur_pipeline,
            blend_pipeline,
            tessellation_cache: TessellationCache::new(2048),
            path_interner: PathInterner::default(),
            text_renderer,
            glyph_texture,
            effect_target_pool: RenderTargetPool::new(256 * 1024 * 1024),
            clip_stack: ClipStack::default(),
            effect_sampler,
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

            if let Some(stroke) = &style.stroke
                && let Some(stroke_paths) = style
                    .stroke_geometry
                    .as_ref()
                    .or(style.fill_geometry.as_ref())
            {
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

        report
    }

    pub fn tessellation_cache_stats(&self) -> TessellationCacheStats {
        self.tessellation_cache.stats()
    }

    pub fn reset_tessellation_cache_stats(&mut self) {
        self.tessellation_cache.reset_stats();
    }

    /// Register/update a CPU-sampled image used by `Paint::Image`.
    pub fn register_image_rgba8(
        &mut self,
        image_id: style_engine::ImageId,
        width: u32,
        height: u32,
        rgba8: Vec<u8>,
    ) -> Result<(), RendererError> {
        image_store::register_image_rgba8(image_id, width, height, rgba8)
            .map_err(RendererError::InitializationFailed)
    }

    /// Unregister an image used by `Paint::Image`.
    pub fn unregister_image(&mut self, image_id: style_engine::ImageId) {
        image_store::unregister_image(image_id);
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
    ) -> (
        Vec<PrimitiveInstance>,
        Vec<TextNodeData<'a>>,
        Vec<PathBatch>,
    ) {
        Self::collect_instances_impl(
            Some(pipeline),
            Some(tessellation_cache),
            Some(path_interner),
            scene,
            false,
        )
    }

    fn collect_instances_excluding_multipass<'a>(
        pipeline: &mut PrimitivePipeline,
        tessellation_cache: &mut TessellationCache,
        path_interner: &mut PathInterner,
        scene: &'a Scene,
    ) -> (
        Vec<PrimitiveInstance>,
        Vec<TextNodeData<'a>>,
        Vec<PathBatch>,
    ) {
        Self::collect_instances_impl(
            Some(pipeline),
            Some(tessellation_cache),
            Some(path_interner),
            scene,
            true,
        )
    }

    #[cfg(test)]
    fn collect_instances_for_tests<'a>(
        scene: &'a Scene,
    ) -> (
        Vec<PrimitiveInstance>,
        Vec<TextNodeData<'a>>,
        Vec<PathBatch>,
    ) {
        Self::collect_instances_impl(None, None, None, scene, false)
    }

    #[cfg(test)]
    fn collect_instances_without_multipass_for_tests<'a>(
        scene: &'a Scene,
    ) -> (
        Vec<PrimitiveInstance>,
        Vec<TextNodeData<'a>>,
        Vec<PathBatch>,
    ) {
        Self::collect_instances_impl(None, None, None, scene, true)
    }

    fn collect_instances_impl<'a>(
        mut pipeline: Option<&mut PrimitivePipeline>,
        mut tessellation_cache: Option<&mut TessellationCache>,
        mut path_interner: Option<&mut PathInterner>,
        scene: &'a Scene,
        skip_multipass: bool,
    ) -> (
        Vec<PrimitiveInstance>,
        Vec<TextNodeData<'a>>,
        Vec<PathBatch>,
    ) {
        use crate::NodeContent;
        let mut instances = Vec::new();
        let mut text_nodes_for_shaping = Vec::new();
        let mut path_batches = Vec::new();

        for (node_id, node) in scene.iter_visuals() {
            if !node.visible || node.opacity <= 0.0 {
                continue;
            }

            if let NodeContent::Styled { style } = &node.content {
                if skip_multipass && style_requires_multipass(style) {
                    continue;
                }
                let effective_opacity = node.opacity * style.opacity;
                let Some(render_bounds) = clipped_bounds_for_node(scene, node_id, node.bounds)
                else {
                    continue;
                };

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

                        if let Ok(mesh) = mesh_result
                            && !mesh.indices.is_empty()
                        {
                            path_batches.push(PathBatch {
                                mesh,
                                paint: fill_paint.clone(),
                                opacity: effective_opacity,
                                size: [render_bounds.width, render_bounds.height],
                                offset: [render_bounds.x, render_bounds.y],
                            });
                        }
                    }

                    if let Some(stroke) = &style.stroke {
                        let stroke_paint = resolve_path_stroke_paint(stroke);
                        if let Some(stroke_paths) = style
                            .stroke_geometry
                            .as_ref()
                            .or(style.fill_geometry.as_ref())
                        {
                            for path in stroke_paths {
                                let path_hash = if let Some(interner) = path_interner.as_deref_mut()
                                {
                                    interner.hash_for(path)
                                } else {
                                    TessellationCache::fill_key(path)
                                };
                                let stroke_key =
                                    TessellationCache::stroke_key_from_path_hash(path_hash, stroke);
                                let mesh_result = if let Some(cache) =
                                    tessellation_cache.as_deref_mut()
                                {
                                    cache
                                        .get_or_tessellate_stroke_with_key(stroke_key, path, stroke)
                                } else {
                                    tessellate_stroke(path, stroke).map(Arc::new)
                                };
                                if let Ok(mesh) = mesh_result
                                    && !mesh.indices.is_empty()
                                {
                                    path_batches.push(PathBatch {
                                        mesh,
                                        paint: stroke_paint.clone(),
                                        opacity: effective_opacity,
                                        size: [render_bounds.width, render_bounds.height],
                                        offset: [render_bounds.x, render_bounds.y],
                                    });
                                }
                            }
                        }
                    }
                    continue;
                }

                // Route image-filled rectangles through path batches so Paint::Image
                // samples are resolved by the path pipeline instead of magenta fallback.
                if matches!(style.fills.first(), Some(style_engine::Paint::Image(_))) {
                    let fill_paint = resolve_path_fill_paint(style);
                    let rect_path = rect_path_for_size(render_bounds.width, render_bounds.height);
                    let path_hash = TessellationCache::fill_key(&rect_path);
                    let mesh_result = if let Some(cache) = tessellation_cache.as_deref_mut() {
                        cache.get_or_tessellate_fill_with_key(path_hash, &rect_path)
                    } else {
                        tessellate_fill(&rect_path).map(Arc::new)
                    };
                    if let Ok(mesh) = mesh_result
                        && !mesh.indices.is_empty()
                    {
                        path_batches.push(PathBatch {
                            mesh,
                            paint: fill_paint.clone(),
                            opacity: effective_opacity,
                            size: [render_bounds.width, render_bounds.height],
                            offset: [render_bounds.x, render_bounds.y],
                        });
                    }

                    if let Some(stroke) = &style.stroke {
                        let stroke_paint = resolve_path_stroke_paint(stroke);
                        let stroke_key =
                            TessellationCache::stroke_key_from_path_hash(path_hash, stroke);
                        let stroke_mesh_result = if let Some(cache) =
                            tessellation_cache.as_deref_mut()
                        {
                            cache.get_or_tessellate_stroke_with_key(stroke_key, &rect_path, stroke)
                        } else {
                            tessellate_stroke(&rect_path, stroke).map(Arc::new)
                        };
                        if let Ok(mesh) = stroke_mesh_result
                            && !mesh.indices.is_empty()
                        {
                            path_batches.push(PathBatch {
                                mesh,
                                paint: stroke_paint,
                                opacity: effective_opacity,
                                size: [render_bounds.width, render_bounds.height],
                                offset: [render_bounds.x, render_bounds.y],
                            });
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
                let pos = glam::Vec2::new(render_bounds.x, render_bounds.y);
                let size = glam::Vec2::new(render_bounds.width, render_bounds.height);
                let node_instances = if let Some(pipeline) = pipeline.as_deref_mut() {
                    create_primitive_instances_with_pipeline(
                        pipeline,
                        style,
                        pos,
                        size,
                        effective_opacity,
                    )
                } else {
                    create_primitive_instances(style, pos, size, effective_opacity)
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
            style_engine::Paint::Image(_) => {
                TextFill::Solid(glam::Vec4::new(0.0, 0.0, 0.0, opacity))
            }
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

fn rect_intersection(a: plat_core::Rect, b: plat_core::Rect) -> Option<plat_core::Rect> {
    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = (a.x + a.width).min(b.x + b.width);
    let y1 = (a.y + a.height).min(b.y + b.height);

    if x1 <= x0 || y1 <= y0 {
        return None;
    }

    Some(plat_core::Rect::new(x0, y0, x1 - x0, y1 - y0))
}

fn ancestor_clip_bounds(scene: &Scene, node_id: crate::NodeId) -> Option<plat_core::Rect> {
    use crate::NodeContent;

    let mut current = scene.parent(node_id);
    let mut clip: Option<plat_core::Rect> = None;

    while let Some(parent_id) = current {
        let Some(parent_node) = scene.get_node(parent_id) else {
            break;
        };

        if let NodeContent::Styled { style } = &parent_node.content
            && style.clips_content
        {
            clip = Some(match clip {
                Some(existing) => rect_intersection(existing, parent_node.bounds)?,
                None => parent_node.bounds,
            });
        }

        current = parent_node.parent;
    }

    clip
}

/// Compute active sibling-mask bounds for `node_id` across ancestor levels.
///
/// Figma-style behavior is approximated by selecting the last preceding mask sibling
/// in each ancestor level and intersecting those bounds.
fn ancestor_mask_bounds(scene: &Scene, node_id: crate::NodeId) -> Option<plat_core::Rect> {
    use crate::NodeContent;

    let mut current = node_id;
    let mut mask_clip: Option<plat_core::Rect> = None;

    while let Some(parent_id) = scene.parent(current) {
        let Some(parent_node) = scene.get_node(parent_id) else {
            break;
        };

        let mut level_mask: Option<plat_core::Rect> = None;
        for &sibling_id in &parent_node.children {
            if sibling_id == current {
                break;
            }
            let Some(sibling) = scene.get_node(sibling_id) else {
                continue;
            };
            if !sibling.visible || sibling.opacity <= 0.0 {
                continue;
            }
            if let NodeContent::Styled { style } = &sibling.content
                && style.is_mask
            {
                level_mask = Some(sibling.bounds);
            }
        }

        if let Some(level_mask) = level_mask {
            mask_clip = Some(match mask_clip {
                Some(existing) => rect_intersection(existing, level_mask)?,
                None => level_mask,
            });
        }

        current = parent_id;
    }

    mask_clip
}

fn clipped_bounds_for_node(
    scene: &Scene,
    node_id: crate::NodeId,
    node_bounds: plat_core::Rect,
) -> Option<plat_core::Rect> {
    let mut clipped = node_bounds;
    if let Some(clip) = ancestor_clip_bounds(scene, node_id) {
        clipped = rect_intersection(clipped, clip)?;
    }
    if let Some(mask) = ancestor_mask_bounds(scene, node_id) {
        clipped = rect_intersection(clipped, mask)?;
    }
    Some(clipped)
}

fn rect_to_scissor_bounds(
    bounds: plat_core::Rect,
    frame_width: u32,
    frame_height: u32,
) -> Option<[u32; 4]> {
    let x0 = bounds.x.max(0.0).min(frame_width as f32).floor() as u32;
    let y0 = bounds.y.max(0.0).min(frame_height as f32).floor() as u32;
    let x1 = (bounds.x + bounds.width)
        .max(0.0)
        .min(frame_width as f32)
        .ceil() as u32;
    let y1 = (bounds.y + bounds.height)
        .max(0.0)
        .min(frame_height as f32)
        .ceil() as u32;

    if x1 <= x0 || y1 <= y0 {
        return None;
    }

    Some([x0, y0, x1 - x0, y1 - y0])
}

fn rect_path_for_size(width: f32, height: f32) -> style_engine::VectorPath {
    let mut path = style_engine::VectorPath::new();
    path.move_to(glam::Vec2::new(0.0, 0.0));
    path.line_to(glam::Vec2::new(width, 0.0));
    path.line_to(glam::Vec2::new(width, height));
    path.line_to(glam::Vec2::new(0.0, height));
    path.close();
    path
}

fn style_requires_multipass(style: &style_engine::VisualStyle) -> bool {
    if !matches!(
        style.blend_mode,
        style_engine::BlendMode::Normal | style_engine::BlendMode::PassThrough
    ) {
        return true;
    }

    style.effects.iter().any(|effect| match effect {
        style_engine::Effect::LayerBlur(blur) => blur.visible && blur.radius > 0.0,
        style_engine::Effect::BackgroundBlur(blur) => blur.visible && blur.radius > 0.0,
        style_engine::Effect::InnerShadow(shadow) => shadow.visible,
        _ => false,
    })
}

fn collect_multipass_node_ids(scene: &Scene) -> Vec<crate::NodeId> {
    use crate::NodeContent;

    scene
        .iter_visuals()
        .filter_map(|(node_id, node)| {
            if !node.visible || node.opacity <= 0.0 {
                return None;
            }
            match &node.content {
                NodeContent::Styled { style } if style_requires_multipass(style) => Some(node_id),
                _ => None,
            }
        })
        .collect()
}

fn classify_scene_effect_kinds(scene: &Scene) -> Vec<EffectPassKind> {
    use crate::NodeContent;

    let mut kinds = Vec::new();
    for (_, node) in scene.iter_visuals() {
        if !node.visible || node.opacity <= 0.0 {
            continue;
        }
        let NodeContent::Styled { style } = &node.content else {
            continue;
        };
        kinds.extend(classify_effect_passes(style, !node.children.is_empty()));
    }
    kinds
}

fn max_scene_blur_radius(scene: &Scene) -> f32 {
    use crate::NodeContent;
    use style_engine::Effect;

    let mut max_radius = 0.0f32;
    for (_, node) in scene.iter_visuals() {
        if !node.visible || node.opacity <= 0.0 {
            continue;
        }
        let NodeContent::Styled { style } = &node.content else {
            continue;
        };
        for effect in &style.effects {
            match effect {
                Effect::LayerBlur(blur) if blur.visible => {
                    max_radius = max_radius.max(blur.radius);
                }
                Effect::BackgroundBlur(blur) if blur.visible => {
                    max_radius = max_radius.max(blur.radius);
                }
                _ => {}
            }
        }
    }
    max_radius
}

fn collect_background_capture_bounds(
    scene: &Scene,
    frame_width: u32,
    frame_height: u32,
) -> Vec<[u32; 4]> {
    use crate::NodeContent;
    use style_engine::Effect;

    let mut bounds = Vec::new();
    let frame = [0, 0, frame_width, frame_height];

    for (_, node) in scene.iter_visuals() {
        if !node.visible || node.opacity <= 0.0 {
            continue;
        }
        let NodeContent::Styled { style } = &node.content else {
            continue;
        };

        for effect in &style.effects {
            if let Effect::BackgroundBlur(blur) = effect
                && blur.visible
                && blur.radius > 0.0
            {
                let node_bounds = [
                    node.bounds.x.max(0.0) as u32,
                    node.bounds.y.max(0.0) as u32,
                    node.bounds.width.max(0.0) as u32,
                    node.bounds.height.max(0.0) as u32,
                ];
                bounds.push(backdrop_capture_bounds(node_bounds, blur.radius, frame));
            }
        }
    }

    bounds
}

impl WgpuBackend {
    /// Register font bytes for text shaping/rasterization.
    ///
    /// Returns number of newly visible faces in the backing text engine database.
    pub fn register_font_bytes(&mut self, bytes: Vec<u8>) -> usize {
        self.text_renderer.register_font_bytes(bytes)
    }

    fn prepare_phase4_effect_state(&mut self, scene: &Scene) {
        // Phase 4 planner: detect effects that require offscreen multipass work.
        // Current integration reserves pooled targets and keeps the direct renderer
        // path active until full per-node effect compositing is layered in.
        let effect_kinds = classify_scene_effect_kinds(scene);
        let _blur_tier = select_blur_tier(max_scene_blur_radius(scene));
        let _background_capture_bounds = collect_background_capture_bounds(
            scene,
            self.context.config.width,
            self.context.config.height,
        );
        let _clip_sequence = if effect_kinds
            .iter()
            .any(|k| matches!(k, EffectPassKind::StencilPush))
        {
            plan_clip_sequence_for_nested_clips()
        } else {
            Vec::new()
        };
        let requires_offscreen = effect_kinds.iter().any(|kind| {
            matches!(
                kind,
                EffectPassKind::OffscreenLayer
                    | EffectPassKind::BackgroundCapture
                    | EffectPassKind::BlurHorizontal
                    | EffectPassKind::BlurVertical
                    | EffectPassKind::BlendComposite
                    | EffectPassKind::InnerShadow
            )
        });
        if requires_offscreen {
            let key =
                RenderTargetKey::new(self.context.config.width, self.context.config.height, false);
            let context = &mut self.context;
            let handle = self.effect_target_pool.acquire(key, |pool_key| {
                let h = context.create_render_target(pool_key);
                let bytes = context
                    .get_render_target(h)
                    .map(|target| target.estimated_bytes())
                    .unwrap_or_else(|| pool_key.estimated_bytes());
                (h, bytes)
            });
            let _ = self.effect_target_pool.release(handle);
            self.effect_target_pool.end_frame_with(|evicted| {
                let _ = context.remove_render_target(evicted);
            });
        }
    }

    fn collect_frame_batches_internal(
        &mut self,
        scene: &Scene,
        skip_multipass: bool,
    ) -> (Vec<PrimitiveInstance>, Vec<PathBatch>) {
        // Clear per-frame gradient data
        self.primitive_pipeline.clear_gradient_params();

        let (mut instances, raw_text_nodes, path_batches) = if skip_multipass {
            Self::collect_instances_excluding_multipass(
                &mut self.primitive_pipeline,
                &mut self.tessellation_cache,
                &mut self.path_interner,
                scene,
            )
        } else {
            Self::collect_instances(
                &mut self.primitive_pipeline,
                &mut self.tessellation_cache,
                &mut self.path_interner,
                scene,
            )
        };

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
            )> = if raw_text_nodes.len() >= TEXT_PARALLEL_THRESHOLD {
                // Parallel shaping
                raw_text_nodes
                    .par_iter()
                    .filter(|(_, text, _, _)| !text.is_empty())
                    .map(|(node, text, font_size, style)| {
                        let shaped = shape_text_parallel(text, *font_size);
                        let position = glam::Vec2::new(node.bounds.x, node.bounds.y + font_size);
                        let text_bounds = [
                            node.bounds.x,
                            node.bounds.y,
                            node.bounds.width,
                            node.bounds.height,
                        ];
                        (
                            position,
                            node.opacity * style.opacity,
                            text_bounds,
                            *style,
                            shaped,
                        )
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
                        let text_bounds = [
                            node.bounds.x,
                            node.bounds.y,
                            node.bounds.width,
                            node.bounds.height,
                        ];
                        (
                            position,
                            node.opacity * style.opacity,
                            text_bounds,
                            *style,
                            shaped,
                        )
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
                let glyph_instances =
                    self.text_renderer
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

        (instances, path_batches)
    }

    fn collect_frame_batches(&mut self, scene: &Scene) -> (Vec<PrimitiveInstance>, Vec<PathBatch>) {
        self.collect_frame_batches_internal(scene, false)
    }

    fn collect_frame_batches_without_multipass(
        &mut self,
        scene: &Scene,
    ) -> (Vec<PrimitiveInstance>, Vec<PathBatch>) {
        self.collect_frame_batches_internal(scene, true)
    }

    fn collect_style_batches_for_bounds(
        &mut self,
        style: &style_engine::VisualStyle,
        effective_opacity: f32,
        render_bounds: plat_core::Rect,
    ) -> (Vec<PrimitiveInstance>, Vec<PathBatch>) {
        let mut instances = Vec::new();
        let mut path_batches = Vec::new();

        if let Some(paths) = &style.fill_geometry {
            let fill_paint = resolve_path_fill_paint(style);

            for path in paths {
                let path_hash = self.path_interner.hash_for(path);
                let mesh_result = self
                    .tessellation_cache
                    .get_or_tessellate_fill_with_key(path_hash, path);

                if let Ok(mesh) = mesh_result
                    && !mesh.indices.is_empty()
                {
                    path_batches.push(PathBatch {
                        mesh,
                        paint: fill_paint.clone(),
                        opacity: effective_opacity,
                        size: [render_bounds.width, render_bounds.height],
                        offset: [render_bounds.x, render_bounds.y],
                    });
                }
            }

            if let Some(stroke) = &style.stroke {
                let stroke_paint = resolve_path_stroke_paint(stroke);
                if let Some(stroke_paths) = style
                    .stroke_geometry
                    .as_ref()
                    .or(style.fill_geometry.as_ref())
                {
                    for path in stroke_paths {
                        let path_hash = self.path_interner.hash_for(path);
                        let stroke_key =
                            TessellationCache::stroke_key_from_path_hash(path_hash, stroke);
                        let mesh_result = self
                            .tessellation_cache
                            .get_or_tessellate_stroke_with_key(stroke_key, path, stroke);
                        if let Ok(mesh) = mesh_result
                            && !mesh.indices.is_empty()
                        {
                            path_batches.push(PathBatch {
                                mesh,
                                paint: stroke_paint.clone(),
                                opacity: effective_opacity,
                                size: [render_bounds.width, render_bounds.height],
                                offset: [render_bounds.x, render_bounds.y],
                            });
                        }
                    }
                }
            }
        } else {
            let pos = glam::Vec2::new(render_bounds.x, render_bounds.y);
            let size = glam::Vec2::new(render_bounds.width, render_bounds.height);
            let node_instances = create_primitive_instances_with_pipeline(
                &mut self.primitive_pipeline,
                style,
                pos,
                size,
                effective_opacity,
            );
            instances.extend(node_instances);
        }

        if let Some(text_content) = &style.text
            && !text_content.text.is_empty()
        {
            let shaped = self
                .text_renderer
                .text_engine_mut()
                .shape_text(&text_content.text, text_content.font_size);
            let position =
                glam::Vec2::new(render_bounds.x, render_bounds.y + text_content.font_size);
            let text_bounds = [
                render_bounds.x,
                render_bounds.y,
                render_bounds.width,
                render_bounds.height,
            ];
            let fill = Self::resolve_text_fill(
                &mut self.primitive_pipeline,
                style,
                effective_opacity,
                text_bounds,
            );
            let glyph_instances =
                self.text_renderer
                    .generate_instances(&shaped, position, glam::Vec4::ONE);
            for mut instance in glyph_instances {
                apply_text_fill_to_glyph(&mut instance, fill);
                instances.push(instance);
            }

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

        (instances, path_batches)
    }

    fn acquire_effect_target(&mut self, key: RenderTargetKey) -> RenderTargetHandle {
        let context = &mut self.context;
        self.effect_target_pool.acquire(key, |pool_key| {
            let handle = context.create_render_target(pool_key);
            let bytes = context
                .get_render_target(handle)
                .map(|target| target.estimated_bytes())
                .unwrap_or_else(|| pool_key.estimated_bytes());
            (handle, bytes)
        })
    }

    fn release_effect_target(&mut self, handle: RenderTargetHandle) {
        let _ = self.effect_target_pool.release(handle);
    }

    fn draw_batches_to_view(
        &mut self,
        target_view: &wgpu::TextureView,
        load_op: wgpu::LoadOp<wgpu::Color>,
        instances: &[PrimitiveInstance],
        path_batches: &[PathBatch],
        scissor: Option<[u32; 4]>,
    ) {
        let should_skip = instances.is_empty()
            && path_batches.is_empty()
            && !matches!(load_op, wgpu::LoadOp::Clear(_));
        if should_skip {
            return;
        }

        self.primitive_pipeline
            .prepare(&self.context.device, &self.context.queue, instances);
        self.path_pipeline
            .prepare(&self.context.device, &self.context.queue, path_batches);

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Multipass Draw Batches Encoder"),
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Multipass Draw Batches"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        if let Some([x, y, width, height]) = scissor
            && width > 0
            && height > 0
        {
            render_pass.set_scissor_rect(x, y, width, height);
        }

        self.primitive_pipeline.render(
            &mut render_pass,
            &self.context.globals_bind_group,
            instances.len() as u32,
        );
        self.path_pipeline
            .render(&mut render_pass, &self.context.globals_bind_group);

        drop(render_pass);
        self.context.queue.submit(std::iter::once(encoder.finish()));
    }

    fn run_blur_pass(
        &mut self,
        source_view: &wgpu::TextureView,
        target_view: &wgpu::TextureView,
        radius: f32,
        direction: BlurDirection,
    ) {
        let params = BlurParams::from_radius(
            radius,
            self.context.config.width,
            self.context.config.height,
            direction,
        );
        self.blur_pipeline
            .update_params(&self.context.queue, &params);
        let bind_group = self.blur_pipeline.create_bind_group(
            &self.context.device,
            source_view,
            &self.effect_sampler,
        );

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Multipass Blur Encoder"),
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Multipass Blur Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        self.blur_pipeline.render(&mut render_pass, &bind_group);

        drop(render_pass);
        self.context.queue.submit(std::iter::once(encoder.finish()));
    }

    fn run_blend_composite(
        &mut self,
        src_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        target_view: &wgpu::TextureView,
        blend_mode: style_engine::BlendMode,
        scissor: Option<[u32; 4]>,
    ) {
        self.blend_pipeline
            .update_params(&self.context.queue, BlendParams::new(blend_mode));
        let bind_group = self.blend_pipeline.create_bind_group(
            &self.context.device,
            src_view,
            dst_view,
            &self.effect_sampler,
        );

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Multipass Blend Encoder"),
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Multipass Blend Composite"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        if let Some([x, y, width, height]) = scissor
            && width > 0
            && height > 0
        {
            render_pass.set_scissor_rect(x, y, width, height);
        }

        self.blend_pipeline.render(&mut render_pass, &bind_group);

        drop(render_pass);
        self.context.queue.submit(std::iter::once(encoder.finish()));
    }

    fn copy_texture_full_frame(&self, src: &wgpu::Texture, dst: &wgpu::Texture) {
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Multipass Copy Texture Encoder"),
                });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: dst,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.context.config.width,
                height: self.context.config.height,
                depth_or_array_layers: 1,
            },
        );
        self.context.queue.submit(std::iter::once(encoder.finish()));
    }

    fn render_multipass_effect_nodes(
        &mut self,
        scene: &Scene,
        multipass_node_ids: &[crate::NodeId],
        surface_texture: &wgpu::Texture,
        surface_view: &wgpu::TextureView,
    ) {
        let frame_key =
            RenderTargetKey::new(self.context.config.width, self.context.config.height, false);

        for &node_id in multipass_node_ids {
            let Some(node) = scene.get_node(node_id) else {
                continue;
            };
            let crate::NodeContent::Styled { style } = &node.content else {
                continue;
            };
            if !node.visible || node.opacity <= 0.0 {
                continue;
            }

            let style = style.as_ref().clone();
            let effective_opacity = node.opacity * style.opacity;
            let Some(render_bounds) = clipped_bounds_for_node(scene, node_id, node.bounds) else {
                continue;
            };
            let scissor = rect_to_scissor_bounds(
                render_bounds,
                self.context.config.width,
                self.context.config.height,
            );

            let layer_blur_radius = style.effects.iter().find_map(|effect| match effect {
                style_engine::Effect::LayerBlur(blur) if blur.visible && blur.radius > 0.0 => {
                    Some(blur.radius)
                }
                _ => None,
            });
            let background_blur_radius = style.effects.iter().find_map(|effect| match effect {
                style_engine::Effect::BackgroundBlur(blur) if blur.visible && blur.radius > 0.0 => {
                    Some(blur.radius)
                }
                _ => None,
            });

            let src_handle = self.acquire_effect_target(frame_key);
            let tmp_handle = self.acquire_effect_target(frame_key);
            let dst_handle = self.acquire_effect_target(frame_key);

            let src_view = self
                .context
                .get_render_target(src_handle)
                .expect("missing src render target")
                .color_view
                .clone();
            let tmp_view = self
                .context
                .get_render_target(tmp_handle)
                .expect("missing temp render target")
                .color_view
                .clone();
            let dst_view = self
                .context
                .get_render_target(dst_handle)
                .expect("missing dst render target")
                .color_view
                .clone();
            let src_texture = self
                .context
                .get_render_target(src_handle)
                .expect("missing src render target texture")
                .color_texture
                .clone();
            let dst_texture = self
                .context
                .get_render_target(dst_handle)
                .expect("missing dst render target texture")
                .color_texture
                .clone();

            let (node_instances, node_path_batches) =
                self.collect_style_batches_for_bounds(&style, effective_opacity, render_bounds);
            if background_blur_radius.is_none() {
                self.draw_batches_to_view(
                    &src_view,
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    &node_instances,
                    &node_path_batches,
                    None,
                );
            } else {
                self.copy_texture_full_frame(surface_texture, &src_texture);
            }

            if let Some(radius) = layer_blur_radius.or(background_blur_radius) {
                let _ = select_blur_tier(radius);
                self.run_blur_pass(&src_view, &tmp_view, radius, BlurDirection::Horizontal);
                self.run_blur_pass(&tmp_view, &src_view, radius, BlurDirection::Vertical);
            }

            self.copy_texture_full_frame(surface_texture, &dst_texture);

            let blend_mode = if matches!(
                style.blend_mode,
                style_engine::BlendMode::Normal | style_engine::BlendMode::PassThrough
            ) {
                style_engine::BlendMode::Normal
            } else {
                style.blend_mode
            };

            self.run_blend_composite(&src_view, &dst_view, surface_view, blend_mode, scissor);

            if background_blur_radius.is_some() {
                self.draw_batches_to_view(
                    surface_view,
                    wgpu::LoadOp::Load,
                    &node_instances,
                    &node_path_batches,
                    scissor,
                );
            }

            self.release_effect_target(src_handle);
            self.release_effect_target(tmp_handle);
            self.release_effect_target(dst_handle);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_scene_to_rgba(
        &mut self,
        scene: &Scene,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, RendererError> {
        self.context.resize(width, height);
        self.prepare_phase4_effect_state(scene);
        let multipass_node_ids = collect_multipass_node_ids(scene);

        let frame_key = RenderTargetKey::new(width.max(1), height.max(1), false);
        let frame_handle = self.acquire_effect_target(frame_key);
        let frame_view = self
            .context
            .get_render_target(frame_handle)
            .expect("missing frame render target")
            .color_view
            .clone();
        let frame_texture = self
            .context
            .get_render_target(frame_handle)
            .expect("missing frame render target texture")
            .color_texture
            .clone();

        let (base_instances, base_path_batches) = if multipass_node_ids.is_empty() {
            self.collect_frame_batches(scene)
        } else {
            self.collect_frame_batches_without_multipass(scene)
        };
        self.draw_batches_to_view(
            &frame_view,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: self.context.clear_color.r() as f64,
                g: self.context.clear_color.g() as f64,
                b: self.context.clear_color.b() as f64,
                a: self.context.clear_color.a() as f64,
            }),
            &base_instances,
            &base_path_batches,
            None,
        );

        if !multipass_node_ids.is_empty() {
            self.render_multipass_effect_nodes(
                scene,
                &multipass_node_ids,
                &frame_texture,
                &frame_view,
            );
        }

        let rgba = self
            .context
            .read_texture_to_rgba(&frame_texture, width, height)?;

        self.release_effect_target(frame_handle);
        self.effect_target_pool.end_frame_with(|evicted| {
            let _ = self.context.remove_render_target(evicted);
        });

        Ok(rgba)
    }
}

impl super::RenderBackend for WgpuBackend {
    #[instrument(skip(self, scene))]
    fn render(&mut self, scene: &Scene) -> Result<(), RendererError> {
        let _span = span!(Level::TRACE, "render_frame").entered();

        self.prepare_phase4_effect_state(scene);
        let multipass_node_ids = collect_multipass_node_ids(scene);
        if multipass_node_ids.is_empty() {
            let (instances, path_batches) = self.collect_frame_batches(scene);

            self.primitive_pipeline
                .prepare(&self.context.device, &self.context.queue, &instances);
            self.path_pipeline
                .prepare(&self.context.device, &self.context.queue, &path_batches);

            let WgpuBackend {
                context,
                primitive_pipeline,
                path_pipeline,
                clip_stack,
                ..
            } = self;

            return context.with_render_pass(|render_pass, globals_bind_group| {
                primitive_pipeline.render(render_pass, globals_bind_group, instances.len() as u32);
                path_pipeline.render(render_pass, globals_bind_group);
                let _ = clip_stack.depth();
            });
        }

        let (base_instances, base_path_batches) =
            self.collect_frame_batches_without_multipass(scene);

        let output = self.context.surface.get_current_texture()?;
        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.draw_batches_to_view(
            &surface_view,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: self.context.clear_color.r() as f64,
                g: self.context.clear_color.g() as f64,
                b: self.context.clear_color.b() as f64,
                a: self.context.clear_color.a() as f64,
            }),
            &base_instances,
            &base_path_batches,
            None,
        );

        self.render_multipass_effect_nodes(
            scene,
            &multipass_node_ids,
            &output.texture,
            &surface_view,
        );

        self.effect_target_pool.end_frame_with(|evicted| {
            let _ = self.context.remove_render_target(evicted);
        });

        output.present();
        Ok(())
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
    fn test_style_opacity_multiplies_node_opacity_for_primitive_instances() {
        let mut scene = Scene::new();
        let root = scene.root();

        let node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    style_engine::VisualStyle::new()
                        .solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4())
                        .opacity(0.4),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 0.5,
        };
        scene.add_node(root, node);

        let (instances, _text_nodes, _path_batches) =
            WgpuBackend::collect_instances_for_tests(&scene);
        assert_eq!(instances.len(), 1);
        assert!(
            (instances[0].color[3] - 0.2).abs() < 1e-6,
            "expected fill alpha 0.2 from node.opacity(0.5) * style.opacity(0.4), got {}",
            instances[0].color[3]
        );
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
        assert_eq!(
            instance.flags & FLAG_FILL_TYPE_MASK,
            0,
            "solid fill type bits should be 0"
        );
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

        assert_eq!(
            instance.flags & FLAG_FILL_TYPE_MASK,
            3,
            "gradient fill type bits should be set"
        );
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

        let (instances, _text_nodes, path_batches) =
            WgpuBackend::collect_instances_for_tests(&scene);
        assert!(
            instances.is_empty(),
            "path-only node should not emit primitive rect instances"
        );
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

        let (instances, _text_nodes, path_batches) =
            WgpuBackend::collect_instances_for_tests(&scene);
        assert!(
            instances.is_empty(),
            "geometry node should route through path batches"
        );
        assert_eq!(path_batches.len(), 2, "expected fill + stroke path batches");
    }

    #[test]
    fn test_style_opacity_multiplies_node_opacity_for_path_batches() {
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
                        .opacity(0.5)
                        .fill_geometry(vec![path]),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 10.0,
                y: 10.0,
                width: 60.0,
                height: 40.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 0.8,
        };
        scene.add_node(root, node);

        let (_instances, _text_nodes, path_batches) =
            WgpuBackend::collect_instances_for_tests(&scene);
        assert_eq!(path_batches.len(), 1);
        assert!(
            (path_batches[0].opacity - 0.4).abs() < 1e-6,
            "expected path opacity 0.4 from node.opacity(0.8) * style.opacity(0.5), got {}",
            path_batches[0].opacity
        );
    }

    #[test]
    fn test_clips_content_partially_clips_child_primitive_bounds() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut clip_parent = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new()
                    .solid_fill(Color::rgba(0.2, 0.2, 0.2, 1.0).as_vec4())
                    .clips_content(true),
            ),
        });
        clip_parent.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
        let parent_id = scene.add_node(root, clip_parent);

        let mut child = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new().solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4()),
            ),
        });
        child.bounds = plat_core::Rect::new(80.0, 10.0, 40.0, 20.0);
        scene.add_node(parent_id, child);

        let (instances, _, _) = WgpuBackend::collect_instances_for_tests(&scene);
        let clipped_child = instances
            .iter()
            .find(|i| (i.color[0] - 1.0).abs() < 1e-6)
            .expect("expected red child instance");

        assert_eq!(clipped_child.pos[0], 80.0);
        assert_eq!(
            clipped_child.size[0], 20.0,
            "child width should be clipped to parent bounds"
        );
    }

    #[test]
    fn test_clips_content_skips_child_fully_outside_clip_bounds() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut clip_parent = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new()
                    .solid_fill(Color::rgba(0.2, 0.2, 0.2, 1.0).as_vec4())
                    .clips_content(true),
            ),
        });
        clip_parent.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
        let parent_id = scene.add_node(root, clip_parent);

        let mut child = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new().solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4()),
            ),
        });
        child.bounds = plat_core::Rect::new(120.0, 10.0, 40.0, 20.0);
        scene.add_node(parent_id, child);

        let (instances, _, _) = WgpuBackend::collect_instances_for_tests(&scene);
        let red_count = instances
            .iter()
            .filter(|i| (i.color[0] - 1.0).abs() < 1e-6)
            .count();

        assert_eq!(
            red_count, 0,
            "child outside clipping parent should not emit instances"
        );
    }

    #[test]
    fn test_image_fill_rect_is_routed_to_path_batches() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut node = SceneNode::new(NodeContent::Styled {
            style: Box::new(crate::VisualStyle::new().fill(style_engine::Paint::Image(
                style_engine::ImageFill {
                    image_id: style_engine::ImageId(7001),
                    scale_mode: style_engine::ImageScaleMode::Fill,
                    transform: None,
                },
            ))),
        });
        node.bounds = plat_core::Rect::new(20.0, 30.0, 140.0, 90.0);
        scene.add_node(root, node);

        let (instances, _, path_batches) = WgpuBackend::collect_instances_for_tests(&scene);
        assert!(
            instances.is_empty(),
            "image-fill rect should not emit primitive fallback instances"
        );
        assert_eq!(
            path_batches.len(),
            1,
            "expected one path batch for image fill"
        );
        assert!(matches!(
            path_batches[0].paint,
            style_engine::Paint::Image(_)
        ));
    }

    #[test]
    fn test_mask_node_clips_subsequent_sibling_bounds() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut mask = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new()
                    .solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4())
                    .is_mask(true),
            ),
        });
        mask.bounds = plat_core::Rect::new(0.0, 0.0, 40.0, 40.0);
        scene.add_node(root, mask);

        let mut masked = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new().solid_fill(Color::rgba(0.0, 1.0, 0.0, 1.0).as_vec4()),
            ),
        });
        masked.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 40.0);
        scene.add_node(root, masked);

        let (instances, _, _) = WgpuBackend::collect_instances_for_tests(&scene);
        let green = instances
            .iter()
            .find(|i| (i.color[1] - 1.0).abs() < 1e-6 && (i.color[0]).abs() < 1e-6)
            .expect("expected green masked instance");
        assert_eq!(green.size[0], 40.0);
        assert_eq!(green.size[1], 40.0);
    }

    #[test]
    fn test_mask_node_does_not_clip_preceding_sibling() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut before = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new().solid_fill(Color::rgba(0.0, 0.0, 1.0, 1.0).as_vec4()),
            ),
        });
        before.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 40.0);
        scene.add_node(root, before);

        let mut mask = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new()
                    .solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4())
                    .is_mask(true),
            ),
        });
        mask.bounds = plat_core::Rect::new(0.0, 0.0, 40.0, 40.0);
        scene.add_node(root, mask);

        let mut after = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new().solid_fill(Color::rgba(0.0, 1.0, 0.0, 1.0).as_vec4()),
            ),
        });
        after.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 40.0);
        scene.add_node(root, after);

        let (instances, _, _) = WgpuBackend::collect_instances_for_tests(&scene);
        let blue = instances
            .iter()
            .find(|i| (i.color[2] - 1.0).abs() < 1e-6 && (i.color[0]).abs() < 1e-6)
            .expect("expected blue pre-mask instance");
        let green = instances
            .iter()
            .find(|i| (i.color[1] - 1.0).abs() < 1e-6 && (i.color[0]).abs() < 1e-6)
            .expect("expected green post-mask instance");
        assert_eq!(
            blue.size[0], 100.0,
            "preceding sibling should remain unclipped"
        );
        assert_eq!(
            green.size[0], 40.0,
            "sibling after mask should be clipped by mask bounds"
        );
    }

    #[test]
    fn test_style_requires_multipass_detects_blend_and_blur() {
        let blend_only =
            style_engine::VisualStyle::new().blend_mode(style_engine::BlendMode::Multiply);
        assert!(super::style_requires_multipass(&blend_only));

        let layer_blur = style_engine::VisualStyle::new().effect(style_engine::Effect::LayerBlur(
            style_engine::LayerBlur {
                radius: 12.0,
                visible: true,
            },
        ));
        assert!(super::style_requires_multipass(&layer_blur));

        let normal =
            style_engine::VisualStyle::new().solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4());
        assert!(!super::style_requires_multipass(&normal));
    }

    #[test]
    fn test_collect_multipass_node_ids_preserves_visual_order() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut normal = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                style_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(0.2, 0.2, 0.2, 1.0).as_vec4()),
            ),
        });
        normal.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
        scene.add_node(root, normal);

        let mut blend = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                style_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(1.0, 0.0, 0.0, 0.8).as_vec4())
                    .blend_mode(style_engine::BlendMode::Screen),
            ),
        });
        blend.bounds = plat_core::Rect::new(10.0, 10.0, 40.0, 40.0);
        let blend_id = scene.add_node(root, blend);

        let mut blur = SceneNode::new(NodeContent::Styled {
            style: Box::new(style_engine::VisualStyle::new().effect(
                style_engine::Effect::LayerBlur(style_engine::LayerBlur {
                    radius: 8.0,
                    visible: true,
                }),
            )),
        });
        blur.bounds = plat_core::Rect::new(20.0, 20.0, 30.0, 30.0);
        let blur_id = scene.add_node(root, blur);

        let ids = super::collect_multipass_node_ids(&scene);
        assert_eq!(ids, vec![blend_id, blur_id]);
    }

    #[test]
    fn test_collect_instances_without_multipass_skips_effect_nodes() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut normal = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                style_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(0.0, 1.0, 0.0, 1.0).as_vec4()),
            ),
        });
        normal.bounds = plat_core::Rect::new(0.0, 0.0, 20.0, 20.0);
        scene.add_node(root, normal);

        let mut blend = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                style_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(1.0, 0.0, 0.0, 0.8).as_vec4())
                    .blend_mode(style_engine::BlendMode::Multiply),
            ),
        });
        blend.bounds = plat_core::Rect::new(30.0, 0.0, 20.0, 20.0);
        scene.add_node(root, blend);

        let (instances, _text_nodes, _path_batches) =
            WgpuBackend::collect_instances_without_multipass_for_tests(&scene);
        assert_eq!(instances.len(), 1);
        assert!((instances[0].color[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_classify_scene_effect_kinds_detects_offscreen_effects() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut node = SceneNode::new(NodeContent::Styled {
            style: Box::new(style_engine::VisualStyle::new().effect(
                style_engine::Effect::LayerBlur(style_engine::LayerBlur {
                    radius: 12.0,
                    visible: true,
                }),
            )),
        });
        node.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
        scene.add_node(root, node);

        let kinds = super::classify_scene_effect_kinds(&scene);
        assert!(kinds.contains(&super::EffectPassKind::OffscreenLayer));
        assert!(kinds.contains(&super::EffectPassKind::BlurHorizontal));
        assert!(kinds.contains(&super::EffectPassKind::BlurVertical));
    }
}
