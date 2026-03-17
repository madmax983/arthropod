//! WGPU backend implementation.

pub mod clipping;
pub mod context;
pub mod effects;
pub mod image_store;
pub mod instance_collector;
pub mod multipass_executor;
pub mod path_interner;
pub mod pipelines;
pub mod render_target_pool;

use crate::backend::text::TextRenderer;
use crate::{Color, RendererError, Scene};
#[cfg(not(target_arch = "wasm32"))]
use bevy_ecs::prelude::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tracing::{Level, instrument, span};

pub use crate::primitives::PrimitiveInstance;
use context::WgpuContext;
#[cfg(not(target_arch = "wasm32"))]
use effects::RenderTargetKey;
use effects::RenderTargetPool;
use pipelines::blend_pipeline::BlendPipeline;
use pipelines::blur_pipeline::BlurPipeline;
use pipelines::color_filter_pipeline::ColorFilterPipeline;
pub use pipelines::path_pipeline::TessellationCacheStats;
use pipelines::path_pipeline::{PathPipeline, TessellationCache};
use pipelines::primitive_pipeline::PrimitivePipeline;
use pipelines::stencil_pipeline::ClipStack;

pub(crate) const GLYPH_ATLAS_SIZE: u32 = 1024;
pub(crate) const GRADIENT_ATLAS_SIZE: f32 = 1024.0;

pub use instance_collector::create_node_instances;
use multipass_executor::{MultipassRenderer, collect_multipass_node_ids};
use path_interner::PathInterner;

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
    #[allow(dead_code)]
    color_filter_pipeline: ColorFilterPipeline,
    tessellation_cache: TessellationCache,
    path_interner: PathInterner,
    text_renderer: TextRenderer,
    glyph_texture: wgpu::Texture,
    effect_target_pool: RenderTargetPool,
    clip_stack: ClipStack,
    effect_sampler: wgpu::Sampler,
    traversal_stack: Vec<(crate::NodeId, f32)>,
    ordered_nodes_buffer: Vec<multipass_executor::OrderedRenderNode>,
    multipass_node_ids_buffer: Vec<crate::NodeId>,
    /// Pre-allocated buffer for effect pass kinds. Reused across frames via `clear()` to eliminate per-frame `Vec::new()` heap allocations on the hot path.
    effect_kinds_buffer: Vec<effects::EffectPassKind>,
    /// Pre-allocated buffer for background capture bounds. Reused across frames to avoid dynamic allocations during render pass planning.
    background_capture_bounds_buffer: Vec<[u32; 4]>,
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
        // SAFETY: The caller guarantees that `window` will outlive the returned
        // `WgpuBackend` and its internal `wgpu::Surface`. This ensures we don't
        // attempt to interact with a destroyed window from the GPU surface.
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
        // SAFETY: The caller guarantees that `window` will outlive the returned
        // `WgpuBackend` and its internal `wgpu::Surface`. This ensures we don't
        // attempt to interact with a destroyed window from the GPU surface.
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
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
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
        let color_filter_pipeline =
            ColorFilterPipeline::new(&context.device, context.config.format);
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
            color_filter_pipeline,
            tessellation_cache: TessellationCache::new(2048),
            path_interner: PathInterner::default(),
            text_renderer,
            glyph_texture,
            effect_target_pool: RenderTargetPool::new(256 * 1024 * 1024),
            clip_stack: ClipStack::default(),
            effect_sampler,
            traversal_stack: Vec::with_capacity(1024),
            ordered_nodes_buffer: Vec::with_capacity(1024),
            multipass_node_ids_buffer: Vec::with_capacity(128),
            effect_kinds_buffer: Vec::with_capacity(128),
            background_capture_bounds_buffer: Vec::with_capacity(128),
        })
    }

    /// Pre-tessellate visible path geometry for the current scene into the cache.
    ///
    /// Call this after large scene/style loads to bias toward cache-hit rendering.
    pub fn warm_path_cache(&mut self, scene: &Scene) -> PathCacheWarmupReport {
        use crate::NodeContent;
        let mut report = PathCacheWarmupReport::default();

        for (_node_id, node, _) in scene.iter_visuals() {
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
}

impl WgpuBackend {
    /// Configure an authored design-space for uniform fit scaling with centered letterboxing.
    ///
    /// When set, all scene coordinates are projected from design-space into the current
    /// render target while preserving aspect ratio.
    pub fn set_design_space(&mut self, width: f32, height: f32) {
        self.context.set_design_space(width, height);
    }

    /// Clear any configured design-space override and project directly in surface pixels.
    pub fn clear_design_space(&mut self) {
        self.context.clear_design_space();
    }

    /// Register font bytes for text shaping/rasterization.
    ///
    /// Returns number of newly visible faces in the backing text engine database.
    pub fn register_font_bytes(&mut self, bytes: Vec<u8>) -> usize {
        self.text_renderer.register_font_bytes(bytes)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_scene_to_rgba(
        &mut self,
        scene: &Scene,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, RendererError> {
        self.context.resize(width, height);

        let mut executor = MultipassRenderer {
            context: &mut self.context,
            primitive_pipeline: &mut self.primitive_pipeline,
            path_pipeline: &mut self.path_pipeline,
            blur_pipeline: &mut self.blur_pipeline,
            blend_pipeline: &mut self.blend_pipeline,
            color_filter_pipeline: &mut self.color_filter_pipeline,
            effect_target_pool: &mut self.effect_target_pool,
            effect_sampler: &self.effect_sampler,
            tessellation_cache: &mut self.tessellation_cache,
            path_interner: &mut self.path_interner,
            text_renderer: &mut self.text_renderer,
            glyph_texture: &self.glyph_texture,
            traversal_stack: &mut self.traversal_stack,
            ordered_nodes_buffer: &mut self.ordered_nodes_buffer,
            effect_kinds_buffer: &mut self.effect_kinds_buffer,
            background_capture_bounds_buffer: &mut self.background_capture_bounds_buffer,
        };

        executor.prepare_phase4_effect_state(scene);
        collect_multipass_node_ids(
            scene,
            executor.traversal_stack,
            &mut self.multipass_node_ids_buffer,
        );

        let frame_key = RenderTargetKey::new(width.max(1), height.max(1), false);
        let frame_handle = executor.acquire_effect_target(frame_key);
        // Borrow checker might complain here if executor holds mutable borrow of context.
        // executor holds &mut context.
        // We need to access context to get render target.
        // But context is borrowed by executor.
        // This is a typical issue.

        // Refactoring to avoid this split borrow is needed or `executor` methods should do the work.
        // `acquire_effect_target` is on executor.

        // But accessing `frame_view` from `executor.context` while `executor` exists?
        // `executor.context` is `&mut WgpuContext`.
        // If I use `executor` to get handle, I can use `executor.context` to get view.

        // Let's rely on re-borrowing if possible, or maybe I need to drop executor? No, I need it later.

        let frame_view = executor
            .context
            .get_render_target(frame_handle)
            .expect("missing frame render target")
            .color_view
            .clone();
        let frame_texture = executor
            .context
            .get_render_target(frame_handle)
            .expect("missing frame render target texture")
            .color_texture
            .clone();

        if self.multipass_node_ids_buffer.is_empty() {
            let (base_instances, base_path_batches) = executor.collect_frame_batches(scene);
            let clear_color = wgpu::Color {
                r: executor.context.clear_color.r() as f64,
                g: executor.context.clear_color.g() as f64,
                b: executor.context.clear_color.b() as f64,
                a: executor.context.clear_color.a() as f64,
            };
            let mut ctx = multipass_executor::MultipassContext {
                device: &executor.context.device,
                queue: &executor.context.queue,
                config: &executor.context.config,
                globals_bind_group: &executor.context.globals_bind_group,
                primitive_pipeline: &mut *executor.primitive_pipeline,
                path_pipeline: &mut *executor.path_pipeline,
                blur_pipeline: &mut *executor.blur_pipeline,
                blend_pipeline: &mut *executor.blend_pipeline,
                color_filter_pipeline: &mut *executor.color_filter_pipeline,
                effect_sampler: executor.effect_sampler,
            };
            multipass_executor::MultipassRenderer::draw_batches_to_view(
                &mut ctx,
                &frame_view,
                wgpu::LoadOp::Clear(clear_color),
                &base_instances,
                &base_path_batches,
                None,
            );
        } else {
            executor.render_scene_in_visual_order(scene, &frame_texture, &frame_view);
        }

        let rgba = executor
            .context
            .read_texture_to_rgba(&frame_texture, width, height)?;

        executor.release_effect_target(frame_handle);
        executor.end_frame();

        Ok(rgba)
    }
}

impl WgpuBackend {
    /// Render a scene.
    #[instrument(skip(self, scene))]
    pub fn render(&mut self, scene: &Scene) -> Result<(), RendererError> {
        let _span = span!(Level::TRACE, "render_frame").entered();

        let mut executor = MultipassRenderer {
            context: &mut self.context,
            primitive_pipeline: &mut self.primitive_pipeline,
            path_pipeline: &mut self.path_pipeline,
            blur_pipeline: &mut self.blur_pipeline,
            blend_pipeline: &mut self.blend_pipeline,
            color_filter_pipeline: &mut self.color_filter_pipeline,
            effect_target_pool: &mut self.effect_target_pool,
            effect_sampler: &self.effect_sampler,
            tessellation_cache: &mut self.tessellation_cache,
            path_interner: &mut self.path_interner,
            text_renderer: &mut self.text_renderer,
            glyph_texture: &self.glyph_texture,
            traversal_stack: &mut self.traversal_stack,
            ordered_nodes_buffer: &mut self.ordered_nodes_buffer,
            effect_kinds_buffer: &mut self.effect_kinds_buffer,
            background_capture_bounds_buffer: &mut self.background_capture_bounds_buffer,
        };

        executor.prepare_phase4_effect_state(scene);
        collect_multipass_node_ids(
            scene,
            executor.traversal_stack,
            &mut self.multipass_node_ids_buffer,
        );
        if self.multipass_node_ids_buffer.is_empty() {
            let (instances, path_batches) = executor.collect_frame_batches(scene);

            executor.primitive_pipeline.prepare(
                &executor.context.device,
                &executor.context.queue,
                &instances,
            );
            executor.path_pipeline.prepare(
                &executor.context.device,
                &executor.context.queue,
                &path_batches,
            );

            // We need to borrow context from executor again for render pass?
            // `executor` holds mutable borrow of context.
            // But we need `clip_stack` from `self`.
            // `executor` does NOT hold `clip_stack`.
            // So we can borrow `self.clip_stack`.
            // But `executor` holds `&mut self.context`.

            // `with_render_pass` takes `&mut self` on context.
            // `executor.context` IS `&mut context`.
            // So `executor.context.with_render_pass(...)`.

            // Inside closure we use `executor.primitive_pipeline` etc.
            // But `primitive_pipeline.render` takes `&mut RenderPass` and `&BindGroup`.
            // `executor.primitive_pipeline` is `&mut PrimitivePipeline`.

            let clip_stack = &mut self.clip_stack;

            // To avoid borrowing executor in closure while borrowing context from executor...
            // `context.with_render_pass` borrows `context`.
            // `executor` owns `&mut context`.
            // The closure uses `primitive_pipeline` which is also in `executor`.
            // Rust should allow splitting borrows of `executor`? No, `executor` is a struct, not `self`.
            // But `executor` fields are disjoint mutable borrows of `self` fields.
            // Wait, `executor` holds `&mut context` and `&mut primitive_pipeline`.
            // If I call `executor.context.with_render_pass`, `executor` is mutably borrowed (for context).
            // Can I use `executor.primitive_pipeline` in the closure?
            // `primitive_pipeline` is disjoint from `context` in `MultipassRenderer`.
            // If `MultipassRenderer` fields were public I could access them disjointly.
            // They are `pub(crate)`.
            // So `executor.primitive_pipeline` access inside closure while `executor.context` is borrowed might work if compiler is smart enough or if I destructure.

            // Destructuring executor seems best.
            let MultipassRenderer {
                context,
                primitive_pipeline,
                path_pipeline,
                ..
            } = executor;

            return context.with_render_pass(|render_pass, globals_bind_group| {
                primitive_pipeline.render(render_pass, globals_bind_group, instances.len() as u32);
                path_pipeline.render(render_pass, globals_bind_group);
                let _ = clip_stack.depth();
            });
        }

        let output = executor.context.surface.get_current_texture()?;
        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        executor.render_scene_in_visual_order(scene, &output.texture, &surface_view);

        executor.end_frame();

        output.present();
        Ok(())
    }

    /// Resize the rendering surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.context.resize(width, height);
    }

    /// Set the clear color.
    pub fn set_clear_color(&mut self, color: Color) {
        self.context.clear_color = color;
    }
}

#[cfg(test)]
mod tests;
