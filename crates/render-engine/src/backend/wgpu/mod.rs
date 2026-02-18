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

use context::WgpuContext;
use effects::{RenderTargetKey, RenderTargetPool};
use pipelines::blend_pipeline::BlendPipeline;
use pipelines::blur_pipeline::BlurPipeline;
pub use pipelines::path_pipeline::TessellationCacheStats;
use pipelines::path_pipeline::{PathPipeline, TessellationCache};
pub use pipelines::primitive_instance::PrimitiveInstance;
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
}

impl WgpuBackend {
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
            effect_target_pool: &mut self.effect_target_pool,
            effect_sampler: &self.effect_sampler,
            tessellation_cache: &mut self.tessellation_cache,
            path_interner: &mut self.path_interner,
            text_renderer: &mut self.text_renderer,
            glyph_texture: &self.glyph_texture,
        };

        executor.prepare_phase4_effect_state(scene);
        let multipass_node_ids = collect_multipass_node_ids(scene);

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

        // collect_frame_batches uses `self`. But `self` is borrowed by `executor`.
        // So `collect_frame_batches` must be moved to `executor` or `instance_collector`.
        // It uses `self.primitive_pipeline` etc. which are borrowed by `executor`.
        // So `collect_frame_batches` must be called using `executor` components?
        // Or I can't use `self` anymore.

        // This confirms `collect_frame_batches` should be in `instance_collector` or `MultipassRenderer`.
        // `collect_frame_batches` is basically `instance_collector::collect_instances` + text shaping.
        // I can move it to `MultipassRenderer`?

        // For now, I have to duplicate logic or move `collect_frame_batches` out of `WgpuBackend` (to `instance_collector`).
        // I'll assume I can call it on `self`? No, `self` is mutably borrowed.

        // I need to use `executor` to do everything.

        // I'll implement `collect_frame_batches` on `MultipassRenderer` or free function in `instance_collector`.
        // `collect_frame_batches_internal` was using `self`.

        // I will use `instance_collector::collect_frame_batches` (which I need to create/move).

        // Wait, `collect_frame_batches_internal` is complex.

        // Let's implement `collect_frame_batches` on `MultipassRenderer`?
        // Or just move it to `instance_collector.rs`.
        // `instance_collector.rs` already has `collect_instances` which does most of it.
        // The text shaping part is in `collect_frame_batches_internal`.

        // I will assume `executor` has a method `collect_frame_batches`.
        // I will add it to `MultipassRenderer` later.
        // Or I can inline it using `executor` fields.

        // This is getting complicated to do in one step.
        // Maybe I should have moved `collect_frame_batches` first.

        // I'll revert to just replacing the methods first, but keep `render_scene_to_rgba` logic commented out or broken?
        // No, I want it to compile.

        // I'll implement `collect_frame_batches` in `MultipassRenderer` in `multipass_executor.rs`.
        // It needs access to `glyph_texture` (it has it), `text_renderer` (it has it), `queue` (via context).

        // I'll assume I add `collect_frame_batches` to `MultipassRenderer`.

        let (base_instances, base_path_batches) = if multipass_node_ids.is_empty() {
            executor.collect_frame_batches(scene)
        } else {
            executor.collect_frame_batches_without_multipass(scene)
        };

        executor.draw_batches_to_view(
            &frame_view,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: executor.context.clear_color.r() as f64,
                g: executor.context.clear_color.g() as f64,
                b: executor.context.clear_color.b() as f64,
                a: executor.context.clear_color.a() as f64,
            }),
            &base_instances,
            &base_path_batches,
            None,
        );

        if !multipass_node_ids.is_empty() {
            executor.render_multipass_effect_nodes(
                scene,
                &multipass_node_ids,
                &frame_texture,
                &frame_view,
            );
        }

        let rgba = executor
            .context
            .read_texture_to_rgba(&frame_texture, width, height)?;

        executor.release_effect_target(frame_handle);
        executor.end_frame();

        Ok(rgba)
    }
}

impl super::RenderBackend for WgpuBackend {
    #[instrument(skip(self, scene))]
    fn render(&mut self, scene: &Scene) -> Result<(), RendererError> {
        let _span = span!(Level::TRACE, "render_frame").entered();

        let mut executor = MultipassRenderer {
            context: &mut self.context,
            primitive_pipeline: &mut self.primitive_pipeline,
            path_pipeline: &mut self.path_pipeline,
            blur_pipeline: &mut self.blur_pipeline,
            blend_pipeline: &mut self.blend_pipeline,
            effect_target_pool: &mut self.effect_target_pool,
            effect_sampler: &self.effect_sampler,
            tessellation_cache: &mut self.tessellation_cache,
            path_interner: &mut self.path_interner,
            text_renderer: &mut self.text_renderer,
            glyph_texture: &self.glyph_texture,
        };

        executor.prepare_phase4_effect_state(scene);
        let multipass_node_ids = collect_multipass_node_ids(scene);
        if multipass_node_ids.is_empty() {
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

        let (base_instances, base_path_batches) =
            executor.collect_frame_batches_without_multipass(scene);

        let output = executor.context.surface.get_current_texture()?;
        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        executor.draw_batches_to_view(
            &surface_view,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: executor.context.clear_color.r() as f64,
                g: executor.context.clear_color.g() as f64,
                b: executor.context.clear_color.b() as f64,
                a: executor.context.clear_color.a() as f64,
            }),
            &base_instances,
            &base_path_batches,
            None,
        );

        executor.render_multipass_effect_nodes(
            scene,
            &multipass_node_ids,
            &output.texture,
            &surface_view,
        );

        executor.end_frame();

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
    use crate::backend::wgpu::effects;
    use crate::backend::wgpu::instance_collector;
    use crate::backend::wgpu::instance_collector::{TextFill, apply_text_fill_to_glyph};
    use crate::backend::wgpu::multipass_executor;
    use crate::backend::wgpu::pipelines::primitive_instance::FLAG_FILL_TYPE_MASK;
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
        let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);

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
            instance_collector::collect_instances_for_tests(&scene);
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

        let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);
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

        let (_, text_nodes, _) = instance_collector::collect_instances_for_tests(&scene);
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

        let (_, text_nodes, _) = instance_collector::collect_instances_for_tests(&scene);
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

        let (_, text_nodes, _) = instance_collector::collect_instances_for_tests(&scene);
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
                .any(|i| (i.flags & pipelines::primitive_instance::FLAG_IS_SHADOW) != 0),
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
            instance_collector::collect_instances_for_tests(&scene);
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
            instance_collector::collect_instances_for_tests(&scene);
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
            instance_collector::collect_instances_for_tests(&scene);
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

        let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);
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

        let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);
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

        let (instances, _, path_batches) = instance_collector::collect_instances_for_tests(&scene);
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

        let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);
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

        let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);
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
        assert!(effects::style_requires_multipass(&blend_only));

        let layer_blur = style_engine::VisualStyle::new().effect(style_engine::Effect::LayerBlur(
            style_engine::LayerBlur {
                radius: 12.0,
                visible: true,
            },
        ));
        assert!(effects::style_requires_multipass(&layer_blur));

        let normal =
            style_engine::VisualStyle::new().solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4());
        assert!(!effects::style_requires_multipass(&normal));
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

        let ids = multipass_executor::collect_multipass_node_ids(&scene);
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
            instance_collector::collect_instances_without_multipass_for_tests(&scene);
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

        let kinds = multipass_executor::classify_scene_effect_kinds(&scene);
        assert!(kinds.contains(&effects::EffectPassKind::OffscreenLayer));
        assert!(kinds.contains(&effects::EffectPassKind::BlurHorizontal));
        assert!(kinds.contains(&effects::EffectPassKind::BlurVertical));
    }
}
