use crate::backend::text::TextRenderer;
use crate::backend::wgpu::GLYPH_ATLAS_SIZE;
use crate::backend::wgpu::GRADIENT_ATLAS_SIZE;
use crate::backend::wgpu::clipping::{clipped_bounds_for_node, rect_path_for_size};
use crate::backend::wgpu::effects::style_requires_multipass;
use crate::backend::wgpu::path_interner::PathInterner;
use crate::backend::wgpu::pipelines::gradient_atlas::GradientParams;
use crate::backend::wgpu::pipelines::path_pipeline::{
    PathBatch, TessellationCache, tessellate_fill, tessellate_stroke,
};
use crate::backend::wgpu::pipelines::primitive_builder::{
    create_primitive_instances, create_primitive_instances_with_pipeline,
};
use crate::backend::wgpu::pipelines::primitive_instance::{FLAG_FILL_TYPE_MASK, PrimitiveInstance};
use crate::backend::wgpu::pipelines::primitive_pipeline::PrimitivePipeline;
use crate::{NodeContent, Scene, SceneNode};
use std::sync::Arc;

/// Threshold for parallelizing text shaping
/// Below this count, sequential shaping is faster due to thread overhead
pub(crate) const TEXT_PARALLEL_THRESHOLD: usize = 8;

/// Text node data for shaping: (node, text, font_size, style)
pub(crate) type TextNodeData<'a> = (&'a SceneNode, &'a str, f32, &'a style_engine::VisualStyle);

#[derive(Clone, Copy)]
pub(crate) enum TextFill {
    Solid(glam::Vec4),
    Gradient {
        param_index: u32,
        fill_type: u32,
        opacity: f32,
        text_bounds: [f32; 4], // x, y, width, height in scene space
    },
}

pub(crate) struct BatchCollectionContext<'a> {
    pub pipeline: &'a mut PrimitivePipeline,
    pub tessellation_cache: &'a mut TessellationCache,
    pub path_interner: &'a mut PathInterner,
    pub text_renderer: &'a mut TextRenderer,
    pub glyph_texture: &'a wgpu::Texture,
    pub queue: &'a wgpu::Queue,
}

pub(crate) fn apply_text_fill_to_glyph(instance: &mut PrimitiveInstance, fill: TextFill) {
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

pub(crate) fn resolve_text_fill(
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
                end: (gradient.center + glam::Vec2::new(gradient.scale, gradient.scale)).to_array(),
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

pub(crate) fn resolve_path_fill_paint(style: &style_engine::VisualStyle) -> style_engine::Paint {
    style
        .fills
        .first()
        .cloned()
        .unwrap_or_else(|| style_engine::Paint::solid(glam::Vec4::new(1.0, 0.0, 1.0, 1.0)))
}

pub(crate) fn resolve_path_stroke_paint(stroke: &style_engine::StrokeStyle) -> style_engine::Paint {
    stroke
        .top_paint()
        .cloned()
        .unwrap_or_else(|| style_engine::Paint::solid(glam::Vec4::new(1.0, 0.0, 1.0, 1.0)))
}

/// Collect instances from the scene.
///
/// Returns a tuple of (primitive_instances, text_nodes_for_shaping, path_batches).
/// Text nodes are extracted from VisualStyle and returned for shaping.
pub(crate) fn collect_instances<'a>(
    pipeline: &mut PrimitivePipeline,
    tessellation_cache: &mut TessellationCache,
    path_interner: &mut PathInterner,
    scene: &'a Scene,
) -> (
    Vec<PrimitiveInstance>,
    Vec<TextNodeData<'a>>,
    Vec<PathBatch>,
) {
    collect_instances_impl(
        Some(pipeline),
        Some(tessellation_cache),
        Some(path_interner),
        scene,
        false,
    )
}

pub(crate) fn collect_instances_excluding_multipass<'a>(
    pipeline: &mut PrimitivePipeline,
    tessellation_cache: &mut TessellationCache,
    path_interner: &mut PathInterner,
    scene: &'a Scene,
) -> (
    Vec<PrimitiveInstance>,
    Vec<TextNodeData<'a>>,
    Vec<PathBatch>,
) {
    collect_instances_impl(
        Some(pipeline),
        Some(tessellation_cache),
        Some(path_interner),
        scene,
        true,
    )
}

#[cfg(test)]
pub(crate) fn collect_instances_for_tests<'a>(
    scene: &'a Scene,
) -> (
    Vec<PrimitiveInstance>,
    Vec<TextNodeData<'a>>,
    Vec<PathBatch>,
) {
    collect_instances_impl(None, None, None, scene, false)
}

#[cfg(test)]
pub(crate) fn collect_instances_without_multipass_for_tests<'a>(
    scene: &'a Scene,
) -> (
    Vec<PrimitiveInstance>,
    Vec<TextNodeData<'a>>,
    Vec<PathBatch>,
) {
    collect_instances_impl(None, None, None, scene, true)
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
    // We need style_requires_multipass but it is in mod.rs (or multipass_executor.rs later).
    // For now I will inline or import it.
    // It seems it is better to move it to a shared place or redefine it here since it is pure logic on VisualStyle.
    // Or I can pass a predicate.

    // I will duplicate `style_requires_multipass` for now as I plan to move it to `multipass_executor` which is next.
    // But `instance_collector` needs it.
    // Maybe `style_requires_multipass` should be in `effects.rs` or `style-engine`.
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
            let Some(render_bounds) = clipped_bounds_for_node(scene, node_id, node.bounds) else {
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
                            let path_hash = if let Some(interner) = path_interner.as_deref_mut() {
                                interner.hash_for(path)
                            } else {
                                TessellationCache::fill_key(path)
                            };
                            let stroke_key =
                                TessellationCache::stroke_key_from_path_hash(path_hash, stroke);
                            let mesh_result = if let Some(cache) = tessellation_cache.as_deref_mut()
                            {
                                cache.get_or_tessellate_stroke_with_key(stroke_key, path, stroke)
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
                    let stroke_mesh_result = if let Some(cache) = tessellation_cache.as_deref_mut()
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

pub(crate) fn collect_style_batches_for_bounds(
    ctx: &mut BatchCollectionContext,
    style: &style_engine::VisualStyle,
    effective_opacity: f32,
    render_bounds: plat_core::Rect,
) -> (Vec<PrimitiveInstance>, Vec<PathBatch>) {
    let mut instances = Vec::new();
    let mut path_batches = Vec::new();

    if let Some(paths) = &style.fill_geometry {
        let fill_paint = resolve_path_fill_paint(style);

        for path in paths {
            let path_hash = ctx.path_interner.hash_for(path);
            let mesh_result = ctx
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
                    let path_hash = ctx.path_interner.hash_for(path);
                    let stroke_key =
                        TessellationCache::stroke_key_from_path_hash(path_hash, stroke);
                    let mesh_result = ctx
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
            ctx.pipeline,
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
        let shaped = ctx
            .text_renderer
            .text_engine_mut()
            .shape_text(&text_content.text, text_content.font_size);
        let position = glam::Vec2::new(render_bounds.x, render_bounds.y + text_content.font_size);
        let text_bounds = [
            render_bounds.x,
            render_bounds.y,
            render_bounds.width,
            render_bounds.height,
        ];
        let fill = resolve_text_fill(ctx.pipeline, style, effective_opacity, text_bounds);
        let glyph_instances =
            ctx.text_renderer
                .generate_instances(&shaped, position, glam::Vec4::ONE);
        for mut instance in glyph_instances {
            apply_text_fill_to_glyph(&mut instance, fill);
            instances.push(instance);
        }

        ctx.queue.write_texture(
            ctx.glyph_texture.as_image_copy(),
            ctx.text_renderer.atlas().texture_data(),
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
