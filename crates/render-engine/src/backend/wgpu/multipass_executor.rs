use crate::Scene;
use crate::backend::text::TextRenderer;
use crate::backend::wgpu::GLYPH_ATLAS_SIZE;
use crate::backend::wgpu::clipping::{clipped_bounds_for_node, rect_to_scissor_bounds};
use crate::backend::wgpu::context::WgpuContext;
use crate::backend::wgpu::effects::{
    EffectPassKind, backdrop_capture_bounds, classify_effect_passes, color_filter_is_identity,
    style_requires_multipass,
};
use crate::backend::wgpu::instance_collector::{
    BatchCollectionContext, TEXT_PARALLEL_THRESHOLD, apply_node_transform_to_instances,
    apply_text_fill_to_glyph, collect_instances, collect_instances_excluding_multipass,
    collect_style_batches_for_bounds, inherited_node_opacity, resolve_text_fill,
    text_shape_options,
};
use crate::backend::wgpu::path_interner::PathInterner;
use crate::backend::wgpu::pipelines::blend_pipeline::{BlendParams, BlendPipeline};
use crate::backend::wgpu::pipelines::blur_pipeline::{
    BlurDirection, BlurParams, BlurPipeline, select_blur_tier,
};
use crate::backend::wgpu::pipelines::color_filter_pipeline::{
    ColorFilterParams, ColorFilterPipeline,
};
use crate::backend::wgpu::pipelines::path_pipeline::{PathBatch, PathPipeline, TessellationCache};
use crate::backend::wgpu::pipelines::primitive_pipeline::PrimitivePipeline;
use crate::backend::wgpu::render_target_pool::{
    RenderTargetHandle, RenderTargetKey, RenderTargetPool,
};
use crate::primitives::PrimitiveInstance;
use rayon::prelude::*;
use text_engine::{ShapedText, shape_text_parallel_with_options};

pub(crate) struct DrawBatchesParams<'a, 'b> {
    pub target_view: &'a wgpu::TextureView,
    pub load_op: wgpu::LoadOp<wgpu::Color>,
    pub instances: &'a [PrimitiveInstance],
    pub path_batches: &'a [PathBatch<'b>],
    pub scissor: Option<[u32; 4]>,
}

type ShapedTextResult<'a> = (
    glam::Vec2,
    f32,
    [f32; 4],
    &'a style_engine::VisualStyle,
    ShapedText,
    crate::Transform2D,
);

pub(crate) struct MultipassRenderer<'a> {
    pub(crate) context: &'a mut WgpuContext,
    pub(crate) primitive_pipeline: &'a mut PrimitivePipeline,
    pub(crate) path_pipeline: &'a mut PathPipeline,
    pub(crate) blur_pipeline: &'a mut BlurPipeline,
    pub(crate) blend_pipeline: &'a mut BlendPipeline,
    pub(crate) color_filter_pipeline: &'a mut ColorFilterPipeline,
    pub(crate) effect_target_pool: &'a mut RenderTargetPool,
    pub(crate) effect_sampler: &'a wgpu::Sampler,
    pub(crate) tessellation_cache: &'a mut TessellationCache,
    pub(crate) path_interner: &'a mut PathInterner,
    pub(crate) text_renderer: &'a mut TextRenderer,
    pub(crate) glyph_texture: &'a wgpu::Texture,
    pub(crate) traversal_stack: &'a mut Vec<(crate::NodeId, f32)>,
    pub(crate) ordered_nodes_buffer: &'a mut Vec<OrderedRenderNode>,
    pub(crate) effect_kinds_buffer: &'a mut Vec<EffectPassKind>,
    pub(crate) background_capture_bounds_buffer: &'a mut Vec<[u32; 4]>,
}

/// Extracts disjoint mutable borrows from `MultipassRenderer` to allow passing down state
/// into hot loop rendering functions without violating borrow checker aliasing rules.
/// This prevents requiring expensive atomic `.clone()` operations on `wgpu::TextureView` and
/// `wgpu::Texture` handles just to pass them alongside `&mut self`.
pub(crate) struct MultipassContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub globals_bind_group: &'a wgpu::BindGroup,
    pub primitive_pipeline: &'a mut PrimitivePipeline,
    pub path_pipeline: &'a mut PathPipeline,
    pub blur_pipeline: &'a mut BlurPipeline,
    pub blend_pipeline: &'a mut BlendPipeline,
    pub color_filter_pipeline: &'a mut ColorFilterPipeline,
    pub effect_sampler: &'a wgpu::Sampler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrderedRenderNodeKind {
    Direct,
    Multipass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OrderedRenderNode {
    pub node_id: crate::NodeId,
    pub kind: OrderedRenderNodeKind,
}

impl<'a> MultipassRenderer<'a> {
    pub(crate) fn prepare_phase4_effect_state(&mut self, scene: &Scene) {
        // Phase 4 planner: detect effects that require offscreen multipass work.
        // Current integration reserves pooled targets and keeps the direct renderer
        // path active until full per-node effect compositing is layered in.
        classify_scene_effect_kinds(scene, self.traversal_stack, self.effect_kinds_buffer);
        let _blur_tier = select_blur_tier(max_scene_blur_radius(scene, self.traversal_stack));
        collect_background_capture_bounds(
            scene,
            self.traversal_stack,
            self.context.config.width,
            self.context.config.height,
            self.background_capture_bounds_buffer,
        );
        let requires_offscreen = self.effect_kinds_buffer.iter().any(|kind| {
            matches!(
                kind,
                EffectPassKind::OffscreenLayer
                    | EffectPassKind::BackgroundCapture
                    | EffectPassKind::BlurHorizontal
                    | EffectPassKind::BlurVertical
                    | EffectPassKind::ColorFilter
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

    pub(crate) fn render_multipass_effect_nodes<'b>(
        &mut self,
        scene: &'b Scene,
        multipass_node_ids: &[crate::NodeId],
        surface_texture: &wgpu::Texture,
        surface_view: &wgpu::TextureView,
        instances_buffer: &mut Vec<PrimitiveInstance>,
        path_batches_buffer: &mut Vec<PathBatch<'b>>,
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
            if !node.visible {
                continue;
            }
            let inherited_opacity = inherited_node_opacity(scene, node_id);
            if inherited_opacity <= 0.0 {
                continue;
            }

            let style = style.as_ref();
            let effective_opacity = inherited_opacity * style.opacity;
            if effective_opacity <= 0.0 {
                continue;
            }
            let Some(render_bounds) = clipped_bounds_for_node(scene, node_id, node.bounds) else {
                continue;
            };
            let scissor_bounds = self.context.map_scene_rect_to_surface(render_bounds);
            let scissor = rect_to_scissor_bounds(
                scissor_bounds,
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
            let color_filter = style.effects.iter().find_map(|effect| match effect {
                style_engine::Effect::ColorFilter(filter)
                    if filter.visible && !color_filter_is_identity(*filter) =>
                {
                    Some(*filter)
                }
                _ => None,
            });

            let src_handle = self.acquire_effect_target(frame_key);
            let tmp_handle = self.acquire_effect_target(frame_key);
            let dst_handle = self.acquire_effect_target(frame_key);

            let mut batch_ctx = BatchCollectionContext {
                pipeline: self.primitive_pipeline,
                tessellation_cache: self.tessellation_cache,
                path_interner: self.path_interner,
                text_renderer: self.text_renderer,
                glyph_texture: self.glyph_texture,
                queue: &self.context.queue,
            };

            collect_style_batches_for_bounds(
                &mut batch_ctx,
                style,
                effective_opacity,
                render_bounds,
                node.transform,
                instances_buffer,
                path_batches_buffer,
            );

            let src_target = self
                .context
                .get_render_target(src_handle)
                .expect("missing src render target");
            let src_view = &src_target.color_view;
            let src_texture = &src_target.color_texture;

            let tmp_target = self
                .context
                .get_render_target(tmp_handle)
                .expect("missing temp render target");
            let tmp_view = &tmp_target.color_view;
            let tmp_texture = &tmp_target.color_texture;

            let dst_target = self
                .context
                .get_render_target(dst_handle)
                .expect("missing dst render target");
            let dst_view = &dst_target.color_view;
            let dst_texture = &dst_target.color_texture;

            let mut ctx = MultipassContext {
                device: &self.context.device,
                queue: &self.context.queue,
                config: &self.context.config,
                globals_bind_group: &self.context.globals_bind_group,
                primitive_pipeline: &mut *self.primitive_pipeline,
                path_pipeline: &mut *self.path_pipeline,
                blur_pipeline: &mut *self.blur_pipeline,
                blend_pipeline: &mut *self.blend_pipeline,
                color_filter_pipeline: &mut *self.color_filter_pipeline,
                effect_sampler: self.effect_sampler,
            };

            if background_blur_radius.is_none() {
                Self::draw_batches_to_view(
                    &mut ctx,
                    DrawBatchesParams {
                        target_view: src_view,
                        load_op: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        instances: instances_buffer,
                        path_batches: path_batches_buffer,
                        scissor: None,
                    },
                );
            } else {
                Self::copy_texture_full_frame(&ctx, surface_texture, src_texture);
            }

            if let Some(radius) = layer_blur_radius.or(background_blur_radius) {
                let _ = select_blur_tier(radius);
                Self::run_blur_pass(
                    &mut ctx,
                    src_view,
                    tmp_view,
                    radius,
                    BlurDirection::Horizontal,
                );
                Self::run_blur_pass(
                    &mut ctx,
                    tmp_view,
                    src_view,
                    radius,
                    BlurDirection::Vertical,
                );
            }

            if let Some(filter) = color_filter {
                Self::run_color_filter_pass(&mut ctx, src_view, tmp_view, filter);
                Self::copy_texture_full_frame(&ctx, tmp_texture, src_texture);
            }

            Self::copy_texture_full_frame(&ctx, surface_texture, dst_texture);

            let blend_mode = if matches!(
                style.blend_mode,
                style_engine::BlendMode::Normal | style_engine::BlendMode::PassThrough
            ) {
                style_engine::BlendMode::Normal
            } else {
                style.blend_mode
            };

            Self::run_blend_composite(
                &mut ctx,
                src_view,
                dst_view,
                surface_view,
                blend_mode,
                scissor,
            );

            if background_blur_radius.is_some() {
                Self::draw_batches_to_view(
                    &mut ctx,
                    DrawBatchesParams {
                        target_view: surface_view,
                        load_op: wgpu::LoadOp::Load,
                        instances: instances_buffer,
                        path_batches: path_batches_buffer,
                        scissor,
                    },
                );
            }

            self.release_effect_target(src_handle);
            self.release_effect_target(tmp_handle);
            self.release_effect_target(dst_handle);
        }
    }

    pub(crate) fn render_scene_in_visual_order(
        &mut self,
        scene: &'a Scene,
        surface_texture: &wgpu::Texture,
        surface_view: &wgpu::TextureView,
    ) {
        let clear_color = wgpu::Color {
            r: self.context.clear_color.r() as f64,
            g: self.context.clear_color.g() as f64,
            b: self.context.clear_color.b() as f64,
            a: self.context.clear_color.a() as f64,
        };
        let mut ctx = MultipassContext {
            device: &self.context.device,
            queue: &self.context.queue,
            config: &self.context.config,
            globals_bind_group: &self.context.globals_bind_group,
            primitive_pipeline: &mut *self.primitive_pipeline,
            path_pipeline: &mut *self.path_pipeline,
            blur_pipeline: &mut *self.blur_pipeline,
            blend_pipeline: &mut *self.blend_pipeline,
            color_filter_pipeline: &mut *self.color_filter_pipeline,
            effect_sampler: self.effect_sampler,
        };

        Self::draw_batches_to_view(
            &mut ctx,
            DrawBatchesParams {
                target_view: surface_view,
                load_op: wgpu::LoadOp::Clear(clear_color),
                instances: &[],
                path_batches: &[],
                scissor: None,
            },
        );

        collect_ordered_render_nodes(scene, self.traversal_stack, self.ordered_nodes_buffer);

        let mut instances_buffer = Vec::with_capacity(64);
        let mut path_batches_buffer = Vec::with_capacity(16);

        for i in 0..self.ordered_nodes_buffer.len() {
            instances_buffer.clear();
            path_batches_buffer.clear();

            let entry = self.ordered_nodes_buffer[i];
            match entry.kind {
                OrderedRenderNodeKind::Direct => {
                    self.render_direct_node(
                        scene,
                        entry.node_id,
                        surface_view,
                        &mut instances_buffer,
                        &mut path_batches_buffer,
                    );
                }
                OrderedRenderNodeKind::Multipass => {
                    self.render_multipass_effect_nodes(
                        scene,
                        std::slice::from_ref(&entry.node_id),
                        surface_texture,
                        surface_view,
                        &mut instances_buffer,
                        &mut path_batches_buffer,
                    );
                }
            }
        }
    }

    fn render_direct_node<'b>(
        &mut self,
        scene: &'b Scene,
        node_id: crate::NodeId,
        surface_view: &wgpu::TextureView,
        instances_buffer: &mut Vec<PrimitiveInstance>,
        path_batches_buffer: &mut Vec<PathBatch<'b>>,
    ) {
        let Some(node) = scene.get_node(node_id) else {
            return;
        };
        if !node.visible {
            return;
        }
        let inherited_opacity = inherited_node_opacity(scene, node_id);
        if inherited_opacity <= 0.0 {
            return;
        }
        let Some(render_bounds) = clipped_bounds_for_node(scene, node_id, node.bounds) else {
            return;
        };
        let scissor_bounds = self.context.map_scene_rect_to_surface(render_bounds);
        let scissor = rect_to_scissor_bounds(
            scissor_bounds,
            self.context.config.width,
            self.context.config.height,
        );

        match &node.content {
            crate::NodeContent::Styled { style } => {
                let effective_opacity = inherited_opacity * style.opacity;
                if effective_opacity <= 0.0 {
                    return;
                }
                let mut batch_ctx = BatchCollectionContext {
                    pipeline: self.primitive_pipeline,
                    tessellation_cache: self.tessellation_cache,
                    path_interner: self.path_interner,
                    text_renderer: self.text_renderer,
                    glyph_texture: self.glyph_texture,
                    queue: &self.context.queue,
                };
                collect_style_batches_for_bounds(
                    &mut batch_ctx,
                    style,
                    effective_opacity,
                    render_bounds,
                    node.transform,
                    instances_buffer,
                    path_batches_buffer,
                );
                let mut ctx = MultipassContext {
                    device: &self.context.device,
                    queue: &self.context.queue,
                    config: &self.context.config,
                    globals_bind_group: &self.context.globals_bind_group,
                    primitive_pipeline: &mut *self.primitive_pipeline,
                    path_pipeline: &mut *self.path_pipeline,
                    blur_pipeline: &mut *self.blur_pipeline,
                    blend_pipeline: &mut *self.blend_pipeline,
                    color_filter_pipeline: &mut *self.color_filter_pipeline,
                    effect_sampler: self.effect_sampler,
                };
                Self::draw_batches_to_view(
                    &mut ctx,
                    DrawBatchesParams {
                        target_view: surface_view,
                        load_op: wgpu::LoadOp::Load,
                        instances: instances_buffer,
                        path_batches: path_batches_buffer,
                        scissor,
                    },
                );
            }
            crate::NodeContent::SolidColor { color } => {
                let mut final_color = color.to_array();
                final_color[3] *= inherited_opacity;
                if final_color[3] <= 0.0 {
                    return;
                }
                let mut instance = PrimitiveInstance::solid(
                    [render_bounds.x, render_bounds.y],
                    [render_bounds.width, render_bounds.height],
                    final_color,
                );
                apply_node_transform_to_instances(
                    std::slice::from_mut(&mut instance),
                    node.transform,
                );
                let instances = [instance];
                let mut ctx = MultipassContext {
                    device: &self.context.device,
                    queue: &self.context.queue,
                    config: &self.context.config,
                    globals_bind_group: &self.context.globals_bind_group,
                    primitive_pipeline: &mut *self.primitive_pipeline,
                    path_pipeline: &mut *self.path_pipeline,
                    blur_pipeline: &mut *self.blur_pipeline,
                    blend_pipeline: &mut *self.blend_pipeline,
                    color_filter_pipeline: &mut *self.color_filter_pipeline,
                    effect_sampler: self.effect_sampler,
                };
                Self::draw_batches_to_view(
                    &mut ctx,
                    DrawBatchesParams {
                        target_view: surface_view,
                        load_op: wgpu::LoadOp::Load,
                        instances: &instances,
                        path_batches: &[],
                        scissor,
                    },
                );
            }
            crate::NodeContent::Empty => {}
        }
    }

    pub(crate) fn acquire_effect_target(&mut self, key: RenderTargetKey) -> RenderTargetHandle {
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

    pub(crate) fn release_effect_target(&mut self, handle: RenderTargetHandle) {
        let _ = self.effect_target_pool.release(handle);
    }

    /// Renders collected primitive and path instances to the given `target_view`.
    /// Accepts a struct-extracted `MultipassContext` instead of `&mut self` to avoid
    /// `wgpu::TextureView` clones in the caller's hot loop, keeping this abstraction zero-cost.
    pub(crate) fn draw_batches_to_view(
        ctx: &mut MultipassContext,
        params: DrawBatchesParams<'_, '_>,
    ) {
        let target_view = params.target_view;
        let load_op = params.load_op;
        let instances = params.instances;
        let path_batches = params.path_batches;
        let scissor = params.scissor;
        let should_skip = instances.is_empty()
            && path_batches.is_empty()
            && !matches!(load_op, wgpu::LoadOp::Clear(_));
        if should_skip {
            return;
        }

        ctx.primitive_pipeline
            .prepare(ctx.device, ctx.queue, instances);
        ctx.path_pipeline
            .prepare(ctx.device, ctx.queue, path_batches);

        let mut encoder = ctx
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

        ctx.primitive_pipeline.render(
            &mut render_pass,
            ctx.globals_bind_group,
            instances.len() as u32,
        );
        ctx.path_pipeline
            .render(&mut render_pass, ctx.globals_bind_group);

        drop(render_pass);
        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    pub(crate) fn run_blur_pass(
        ctx: &mut MultipassContext,
        source_view: &wgpu::TextureView,
        target_view: &wgpu::TextureView,
        radius: f32,
        direction: BlurDirection,
    ) {
        let params =
            BlurParams::from_radius(radius, ctx.config.width, ctx.config.height, direction);
        ctx.blur_pipeline.update_params(ctx.queue, &params);
        let bind_group =
            ctx.blur_pipeline
                .create_bind_group(ctx.device, source_view, ctx.effect_sampler);

        let mut encoder = ctx
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
        ctx.blur_pipeline.render(&mut render_pass, &bind_group);

        drop(render_pass);
        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    pub(crate) fn run_blend_composite(
        ctx: &mut MultipassContext,
        src_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        target_view: &wgpu::TextureView,
        blend_mode: style_engine::BlendMode,
        scissor: Option<[u32; 4]>,
    ) {
        ctx.blend_pipeline
            .update_params(ctx.queue, BlendParams::new(blend_mode));
        let bind_group = ctx.blend_pipeline.create_bind_group(
            ctx.device,
            src_view,
            dst_view,
            ctx.effect_sampler,
        );

        let mut encoder = ctx
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

        ctx.blend_pipeline.render(&mut render_pass, &bind_group);

        drop(render_pass);
        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    pub(crate) fn run_color_filter_pass(
        ctx: &mut MultipassContext,
        source_view: &wgpu::TextureView,
        target_view: &wgpu::TextureView,
        filter: style_engine::ColorFilter,
    ) {
        ctx.color_filter_pipeline
            .update_params(ctx.queue, ColorFilterParams::new(filter));
        let bind_group = ctx.color_filter_pipeline.create_bind_group(
            ctx.device,
            source_view,
            ctx.effect_sampler,
        );

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Multipass Color Filter Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Multipass Color Filter Pass"),
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

        ctx.color_filter_pipeline
            .render(&mut render_pass, &bind_group);

        drop(render_pass);
        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    pub(crate) fn copy_texture_full_frame(
        ctx: &MultipassContext,
        src: &wgpu::Texture,
        dst: &wgpu::Texture,
    ) {
        let mut encoder = ctx
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
                width: ctx.config.width,
                height: ctx.config.height,
                depth_or_array_layers: 1,
            },
        );
        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    fn collect_frame_batches_internal(
        &mut self,
        scene: &'a Scene,
        skip_multipass: bool,
    ) -> (Vec<PrimitiveInstance>, Vec<PathBatch<'a>>) {
        // Clear per-frame gradient data
        self.primitive_pipeline.clear_gradient_params();

        let (mut instances, raw_text_nodes, path_batches) = if skip_multipass {
            collect_instances_excluding_multipass(
                self.primitive_pipeline,
                self.tessellation_cache,
                self.path_interner,
                self.traversal_stack,
                scene,
            )
        } else {
            collect_instances(
                self.primitive_pipeline,
                self.tessellation_cache,
                self.path_interner,
                self.traversal_stack,
                scene,
            )
        };

        // Process text nodes
        if !raw_text_nodes.is_empty() {
            // Step 1: Shape text in parallel if above threshold
            // Shaping is CPU-intensive and read-only (uses thread-local FontSystem)
            if raw_text_nodes.len() >= TEXT_PARALLEL_THRESHOLD {
                // Parallel shaping
                let shaped_results: Vec<ShapedTextResult<'_>> = raw_text_nodes
                    .par_iter()
                    .filter(|(_, _, text_content, _, _)| !text_content.text.is_empty())
                    .map(|(_, node, text_content, effective_opacity, style)| {
                        let options = text_shape_options(text_content);
                        let shaped = shape_text_parallel_with_options(
                            &text_content.text,
                            text_content.font_size,
                            options,
                        );
                        let position = glam::Vec2::new(
                            node.bounds.x.round(),
                            (node.bounds.y + text_content.font_size).round(),
                        );
                        let text_bounds = [
                            node.bounds.x,
                            node.bounds.y,
                            node.bounds.width,
                            node.bounds.height,
                        ];
                        (
                            position,
                            *effective_opacity,
                            text_bounds,
                            *style,
                            shaped,
                            node.transform,
                        )
                    })
                    .collect();

                // Step 2: Generate glyph instances and add to primitives
                for (position, opacity, text_bounds, style, shaped, node_transform) in
                    shaped_results
                {
                    let fill =
                        resolve_text_fill(self.primitive_pipeline, style, opacity, text_bounds);
                    let mut glyph_instances =
                        self.text_renderer
                            .generate_instances(&shaped, position, glam::Vec4::ONE);
                    apply_node_transform_to_instances(&mut glyph_instances, node_transform);

                    // Apply text fill metadata to generated glyph primitive instances
                    for mut instance in glyph_instances {
                        apply_text_fill_to_glyph(&mut instance, fill);
                        instances.push(instance);
                    }
                }
            } else {
                // Sequential shaping for small counts: no intermediate allocation!
                for (_, node, text_content, effective_opacity, style) in raw_text_nodes
                    .iter()
                    .filter(|(_, _, text_content, _, _)| !text_content.text.is_empty())
                {
                    let options = text_shape_options(text_content);
                    let shaped = self
                        .text_renderer
                        .text_engine_mut()
                        .shape_text_with_options(
                            &text_content.text,
                            text_content.font_size,
                            options,
                        );
                    let position = glam::Vec2::new(
                        node.bounds.x.round(),
                        (node.bounds.y + text_content.font_size).round(),
                    );
                    let text_bounds = [
                        node.bounds.x,
                        node.bounds.y,
                        node.bounds.width,
                        node.bounds.height,
                    ];
                    let fill = resolve_text_fill(
                        self.primitive_pipeline,
                        style,
                        *effective_opacity,
                        text_bounds,
                    );
                    let mut glyph_instances =
                        self.text_renderer
                            .generate_instances(&shaped, position, glam::Vec4::ONE);
                    apply_node_transform_to_instances(&mut glyph_instances, node.transform);

                    for mut instance in glyph_instances {
                        apply_text_fill_to_glyph(&mut instance, fill);
                        instances.push(instance);
                    }
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

    pub(crate) fn collect_frame_batches(
        &mut self,
        scene: &'a Scene,
    ) -> (Vec<PrimitiveInstance>, Vec<PathBatch<'a>>) {
        self.collect_frame_batches_internal(scene, false)
    }

    pub(crate) fn end_frame(&mut self) {
        let pool = &mut self.effect_target_pool;
        let context = &mut self.context;
        pool.end_frame_with(|evicted| {
            let _ = context.remove_render_target(evicted);
        });
    }
}

/// Collects multipass node IDs directly into a pre-allocated buffer to avoid per-frame `Vec` allocations.
/// This optimization matters because creating new `Vec`s on the rendering hot path during scene traversal
/// causes significant memory fragmentation and GC/allocator overhead.
pub(crate) fn collect_multipass_node_ids(
    scene: &Scene,
    stack: &mut Vec<(crate::NodeId, f32)>,
    buffer: &mut Vec<crate::NodeId>,
) {
    use crate::NodeContent;

    buffer.clear();
    for (node_id, node, inherited_opacity) in scene.iter_visuals_custom(stack) {
        if !node.visible || inherited_opacity <= 0.0 {
            continue;
        }
        if matches!(&node.content, NodeContent::Styled { style } if style_requires_multipass(style))
        {
            buffer.push(node_id);
        }
    }
}

/// Collects ordered render nodes directly into a pre-allocated buffer to avoid per-frame `Vec` allocations.
/// Reusing the same `Vec` across frames via `clear()` and `push()` ensures zero allocations on the hot path
/// after the buffer reaches its maximum required capacity, improving frame times and lowering latency.
pub(crate) fn collect_ordered_render_nodes(
    scene: &Scene,
    stack: &mut Vec<(crate::NodeId, f32)>,
    buffer: &mut Vec<OrderedRenderNode>,
) {
    use crate::NodeContent;

    buffer.clear();
    for (node_id, node, inherited_opacity) in scene.iter_visuals_custom(stack) {
        if !node.visible || inherited_opacity <= 0.0 {
            continue;
        }
        match &node.content {
            NodeContent::Styled { style } => buffer.push(OrderedRenderNode {
                node_id,
                kind: if style_requires_multipass(style) {
                    OrderedRenderNodeKind::Multipass
                } else {
                    OrderedRenderNodeKind::Direct
                },
            }),
            NodeContent::SolidColor { .. } => buffer.push(OrderedRenderNode {
                node_id,
                kind: OrderedRenderNodeKind::Direct,
            }),
            NodeContent::Empty => {}
        }
    }
}

/// Populates a pre-allocated buffer with the effect pass kinds for the current scene.
/// Passing `&mut Vec` and calling `clear()` prevents a heap allocation per frame,
/// improving rendering latency and reducing GC overhead.
pub(crate) fn classify_scene_effect_kinds(
    scene: &Scene,
    stack: &mut Vec<(crate::NodeId, f32)>,
    buffer: &mut Vec<EffectPassKind>,
) {
    use crate::NodeContent;

    buffer.clear();
    for (_node_id, node, inherited_opacity) in scene.iter_visuals_custom(stack) {
        if !node.visible {
            continue;
        }
        if inherited_opacity <= 0.0 {
            continue;
        }
        let NodeContent::Styled { style } = &node.content else {
            continue;
        };
        classify_effect_passes(style, !node.children.is_empty(), buffer);
    }
}

pub(crate) fn max_scene_blur_radius(scene: &Scene, stack: &mut Vec<(crate::NodeId, f32)>) -> f32 {
    use crate::NodeContent;
    use style_engine::Effect;

    let mut max_radius = 0.0f32;
    for (_node_id, node, inherited_opacity) in scene.iter_visuals_custom(stack) {
        if !node.visible {
            continue;
        }
        if inherited_opacity <= 0.0 {
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

/// Populates a pre-allocated buffer with background capture bounds.
/// Passing `&mut Vec` avoids dynamically allocating memory (`Vec::new()`)
/// inside the hot path during phase 4 multipass effect planning.
pub(crate) fn collect_background_capture_bounds(
    scene: &Scene,
    stack: &mut Vec<(crate::NodeId, f32)>,
    frame_width: u32,
    frame_height: u32,
    buffer: &mut Vec<[u32; 4]>,
) {
    use crate::NodeContent;
    use style_engine::Effect;

    buffer.clear();
    let frame = [0, 0, frame_width, frame_height];

    for (_node_id, node, inherited_opacity) in scene.iter_visuals_custom(stack) {
        if !node.visible {
            continue;
        }
        if inherited_opacity <= 0.0 {
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
                buffer.push(backdrop_capture_bounds(node_bounds, blur.radius, frame));
            }
        }
    }
}
