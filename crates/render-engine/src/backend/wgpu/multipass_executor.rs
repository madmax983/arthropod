use crate::Scene;
use crate::backend::text::TextRenderer;
use crate::backend::wgpu::GLYPH_ATLAS_SIZE;
use crate::backend::wgpu::clipping::{clipped_bounds_for_node, rect_to_scissor_bounds};
use crate::backend::wgpu::context::WgpuContext;
use crate::backend::wgpu::effects::{
    EffectPassKind, backdrop_capture_bounds, classify_effect_passes, style_requires_multipass,
};
use crate::backend::wgpu::instance_collector::{
    BatchCollectionContext, TEXT_PARALLEL_THRESHOLD, apply_text_fill_to_glyph, collect_instances,
    collect_instances_excluding_multipass, collect_style_batches_for_bounds, resolve_text_fill,
};
use crate::backend::wgpu::path_interner::PathInterner;
use crate::backend::wgpu::pipelines::blend_pipeline::{BlendParams, BlendPipeline};
use crate::backend::wgpu::pipelines::blur_pipeline::{
    BlurDirection, BlurParams, BlurPipeline, select_blur_tier,
};
use crate::backend::wgpu::pipelines::path_pipeline::{PathBatch, PathPipeline, TessellationCache};
use crate::backend::wgpu::pipelines::primitive_instance::PrimitiveInstance;
use crate::backend::wgpu::pipelines::primitive_pipeline::PrimitivePipeline;
use crate::backend::wgpu::pipelines::stencil_pipeline::plan_clip_sequence_for_nested_clips;
use crate::backend::wgpu::render_target_pool::{
    RenderTargetHandle, RenderTargetKey, RenderTargetPool,
};
use rayon::prelude::*;
use text_engine::{ShapedText, shape_text_parallel};

pub(crate) struct MultipassRenderer<'a> {
    pub(crate) context: &'a mut WgpuContext,
    pub(crate) primitive_pipeline: &'a mut PrimitivePipeline,
    pub(crate) path_pipeline: &'a mut PathPipeline,
    pub(crate) blur_pipeline: &'a mut BlurPipeline,
    pub(crate) blend_pipeline: &'a mut BlendPipeline,
    pub(crate) effect_target_pool: &'a mut RenderTargetPool,
    pub(crate) effect_sampler: &'a wgpu::Sampler,
    pub(crate) tessellation_cache: &'a mut TessellationCache,
    pub(crate) path_interner: &'a mut PathInterner,
    pub(crate) text_renderer: &'a mut TextRenderer,
    pub(crate) glyph_texture: &'a wgpu::Texture,
}

impl<'a> MultipassRenderer<'a> {
    pub(crate) fn prepare_phase4_effect_state(&mut self, scene: &Scene) {
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

    pub(crate) fn render_multipass_effect_nodes(
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

            let mut batch_ctx = BatchCollectionContext {
                pipeline: self.primitive_pipeline,
                tessellation_cache: self.tessellation_cache,
                path_interner: self.path_interner,
                text_renderer: self.text_renderer,
                glyph_texture: self.glyph_texture,
                queue: &self.context.queue,
            };

            let (node_instances, node_path_batches) = collect_style_batches_for_bounds(
                &mut batch_ctx,
                &style,
                effective_opacity,
                render_bounds,
            );

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

    pub(crate) fn draw_batches_to_view(
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

    pub(crate) fn run_blur_pass(
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
            self.effect_sampler,
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

    pub(crate) fn run_blend_composite(
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
            self.effect_sampler,
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

    pub(crate) fn copy_texture_full_frame(&self, src: &wgpu::Texture, dst: &wgpu::Texture) {
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

    fn collect_frame_batches_internal(
        &mut self,
        scene: &Scene,
        skip_multipass: bool,
    ) -> (Vec<PrimitiveInstance>, Vec<PathBatch>) {
        // Clear per-frame gradient data
        self.primitive_pipeline.clear_gradient_params();

        let (mut instances, raw_text_nodes, path_batches) = if skip_multipass {
            collect_instances_excluding_multipass(
                self.primitive_pipeline,
                self.tessellation_cache,
                self.path_interner,
                scene,
            )
        } else {
            collect_instances(
                self.primitive_pipeline,
                self.tessellation_cache,
                self.path_interner,
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
                let fill = resolve_text_fill(self.primitive_pipeline, style, opacity, text_bounds);
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

    pub(crate) fn collect_frame_batches(
        &mut self,
        scene: &Scene,
    ) -> (Vec<PrimitiveInstance>, Vec<PathBatch>) {
        self.collect_frame_batches_internal(scene, false)
    }

    pub(crate) fn collect_frame_batches_without_multipass(
        &mut self,
        scene: &Scene,
    ) -> (Vec<PrimitiveInstance>, Vec<PathBatch>) {
        self.collect_frame_batches_internal(scene, true)
    }

    pub(crate) fn end_frame(&mut self) {
        let pool = &mut self.effect_target_pool;
        let context = &mut self.context;
        pool.end_frame_with(|evicted| {
            let _ = context.remove_render_target(evicted);
        });
    }
}

pub(crate) fn collect_multipass_node_ids(scene: &Scene) -> Vec<crate::NodeId> {
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

pub(crate) fn classify_scene_effect_kinds(scene: &Scene) -> Vec<EffectPassKind> {
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

pub(crate) fn max_scene_blur_radius(scene: &Scene) -> f32 {
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

pub(crate) fn collect_background_capture_bounds(
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
