use crate::backend::text::TextRenderer;
use crate::backend::wgpu::GLYPH_ATLAS_SIZE;
use crate::backend::wgpu::GRADIENT_ATLAS_SIZE;
use crate::backend::wgpu::clipping::{clipped_bounds_for_node, rounded_rect_path_for_size};
use crate::backend::wgpu::effects::style_requires_multipass;
use crate::backend::wgpu::path_interner::PathInterner;
use crate::backend::wgpu::pipelines::gradient_atlas::GradientParams;
use crate::backend::wgpu::pipelines::path_pipeline::{
    PathBatch, TessellationCache, tessellate_fill, tessellate_stroke,
};
use crate::backend::wgpu::pipelines::primitive_builder::{
    create_primitive_instances, create_primitive_instances_with_pipeline,
};
use crate::backend::wgpu::pipelines::primitive_pipeline::PrimitivePipeline;
use crate::primitives::{FLAG_FILL_TYPE_MASK, PrimitiveInstance};
use crate::{NodeContent, NodeId, Scene, SceneNode, Transform2D};
use std::sync::Arc;
use text_engine::{ShapedText, TextFontStyle, TextShapeOptions};

/// Threshold for parallelizing text shaping
/// Below this count, sequential shaping is faster due to thread overhead
pub(crate) const TEXT_PARALLEL_THRESHOLD: usize = 8;

/// Text node data for shaping: (node id, node, text content, effective_opacity, style)
pub(crate) type TextNodeData<'a> = (
    NodeId,
    &'a SceneNode,
    &'a style_engine::TextContent,
    f32,
    &'a style_engine::VisualStyle,
);

pub(crate) fn text_shape_options(text_content: &style_engine::TextContent) -> TextShapeOptions<'_> {
    let style = match text_content.font_style {
        style_engine::FontStyle::Normal => TextFontStyle::Normal,
        style_engine::FontStyle::Italic => TextFontStyle::Italic,
        style_engine::FontStyle::Oblique => TextFontStyle::Oblique,
    };
    TextShapeOptions {
        family: text_content
            .font_family
            .as_deref()
            .map(str::trim)
            .filter(|family| !family.is_empty()),
        weight: Some(text_content.font_weight),
        style,
    }
}

/// Compute cumulative node opacity by traversing ancestor chain to the root.
///
/// This models group/frame opacity semantics where ancestor opacity attenuates
/// all descendant rendering.
pub(crate) fn inherited_node_opacity(scene: &Scene, node_id: crate::NodeId) -> f32 {
    let mut opacity = 1.0_f32;
    let mut current = Some(node_id);
    while let Some(id) = current {
        let Some(node) = scene.get_node(id) else {
            break;
        };
        if !node.visible {
            return 0.0;
        }
        opacity *= node.opacity.clamp(0.0, 1.0);
        if opacity <= 0.0 {
            return 0.0;
        }
        current = node.parent;
    }
    opacity
}

/// Apply a scene-node transform to collected primitive instances.
///
/// The primitive shader rotates each instance around its local center using
/// the per-instance packed angle, so here we only need to transform centers.
pub(crate) fn apply_node_transform_to_instances(
    instances: &mut [PrimitiveInstance],
    transform: Transform2D,
) {
    if transform == Transform2D::IDENTITY {
        return;
    }

    let rotation = transform.rotation_radians();
    for instance in instances {
        let half_w = instance.size[0] * 0.5;
        let half_h = instance.size[1] * 0.5;
        let center = glam::Vec2::new(instance.pos[0] + half_w, instance.pos[1] + half_h);
        let transformed_center = transform.transform_point(center);
        instance.pos = [transformed_center.x - half_w, transformed_center.y - half_h];
        instance.set_rotation_radians(rotation);
    }
}

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

fn apply_text_letter_spacing(shaped: &mut ShapedText, letter_spacing: f32) {
    if letter_spacing.abs() <= f32::EPSILON {
        return;
    }
    let glyph_count = shaped.glyphs.len();
    if glyph_count <= 1 {
        return;
    }

    let mut accumulated_shift = 0.0_f32;
    for (index, glyph) in shaped.glyphs.iter_mut().enumerate() {
        glyph.x_offset += accumulated_shift;
        if index + 1 < glyph_count {
            accumulated_shift += letter_spacing;
        }
    }
    shaped.bounds.width = (shaped.bounds.width + accumulated_shift).max(0.0);
}

fn text_shadow_layers(
    style: &style_engine::VisualStyle,
    effective_opacity: f32,
) -> Vec<(glam::Vec2, TextFill)> {
    let mut layers = Vec::new();
    for effect in &style.effects {
        let style_engine::Effect::DropShadow(shadow) = effect else {
            continue;
        };
        if !shadow.visible {
            continue;
        }

        let mut core_color = shadow.color;
        core_color.w *= effective_opacity;
        if core_color.w <= f32::EPSILON {
            continue;
        }

        if shadow.blur > f32::EPSILON {
            let spread = shadow.blur.min(16.0) * 0.35;
            if spread > f32::EPSILON {
                let mut halo_color = core_color;
                halo_color.w *= 0.22;
                if halo_color.w > f32::EPSILON {
                    for extra in [
                        glam::Vec2::new(-spread, 0.0),
                        glam::Vec2::new(spread, 0.0),
                        glam::Vec2::new(0.0, -spread),
                        glam::Vec2::new(0.0, spread),
                    ] {
                        layers.push((shadow.offset + extra, TextFill::Solid(halo_color)));
                    }
                }
            }
        }

        layers.push((shadow.offset, TextFill::Solid(core_color)));
    }
    layers
}

fn text_has_background_surface(style: &style_engine::VisualStyle) -> bool {
    style.text.is_some() && style.fills.iter().skip(1).any(paint_has_visible_alpha)
}

/// Helper to create PrimitiveInstances from a SceneNode (ECS compatibility).
///
/// Uses the no-pipeline fallback path:
/// - solid fills are emitted directly
/// - gradients are approximated to a representative color
/// - strokes and drop shadows are emitted
/// - text glyphs are skipped (text shaping requires backend-owned text/glyph resources)
///   but text background fills/effects may still emit primitive instances
///
/// Returns empty vec if the node is invisible or has no styled content.
pub fn create_node_instances(node: &SceneNode, instances: &mut Vec<PrimitiveInstance>) {
    if !node.visible || node.opacity <= 0.0 {
        return;
    }

    let start_idx = instances.len();

    match &node.content {
        NodeContent::Styled { style } => {
            let pos = glam::Vec2::new(node.bounds.x.round(), node.bounds.y.round());
            let size = glam::Vec2::new(node.bounds.width.round(), node.bounds.height.round());
            create_primitive_instances(style, pos, size, node.opacity * style.opacity, instances);
        }
        NodeContent::SolidColor { color } => {
            let pos = [node.bounds.x.round(), node.bounds.y.round()];
            let size = [node.bounds.width.round(), node.bounds.height.round()];
            let mut final_color = color.to_array();
            final_color[3] *= node.opacity;

            instances.push(PrimitiveInstance::solid(pos, size, final_color));
        }
        NodeContent::Empty => {}
    };

    if node.transform != Transform2D::IDENTITY {
        apply_node_transform_to_instances(&mut instances[start_idx..], node.transform);
    }
}

pub(crate) fn resolve_text_fill(
    pipeline: &mut PrimitivePipeline,
    style: &style_engine::VisualStyle,
    opacity: f32,
    text_bounds: [f32; 4],
) -> TextFill {
    let Some(fill) = pick_text_fill_paint(style) else {
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

fn paint_has_visible_alpha(paint: &style_engine::Paint) -> bool {
    match paint {
        style_engine::Paint::Solid(color) => color.w > f32::EPSILON,
        style_engine::Paint::Linear(gradient) => gradient
            .stops
            .iter()
            .any(|stop| stop.color.w > f32::EPSILON),
        style_engine::Paint::Radial(gradient) => gradient
            .stops
            .iter()
            .any(|stop| stop.color.w > f32::EPSILON),
        style_engine::Paint::Angular(gradient) => gradient
            .stops
            .iter()
            .any(|stop| stop.color.w > f32::EPSILON),
        style_engine::Paint::Diamond(gradient) => gradient
            .stops
            .iter()
            .any(|stop| stop.color.w > f32::EPSILON),
        style_engine::Paint::Image(_) => true,
    }
}

fn pick_text_fill_paint(style: &style_engine::VisualStyle) -> Option<&style_engine::Paint> {
    style
        .fills
        .iter()
        .find(|paint| paint_has_visible_alpha(paint))
        .or_else(|| style.fills.first())
}

static DEFAULT_FILL: std::sync::OnceLock<Vec<style_engine::Paint>> = std::sync::OnceLock::new();
static DEFAULT_STROKE: std::sync::OnceLock<Vec<style_engine::Paint>> = std::sync::OnceLock::new();

pub(crate) fn resolve_path_fill_paints(
    style: &style_engine::VisualStyle,
) -> &[style_engine::Paint] {
    if style.fills.is_empty() {
        DEFAULT_FILL.get_or_init(|| {
            vec![style_engine::Paint::solid(glam::Vec4::new(
                1.0, 0.0, 1.0, 1.0,
            ))]
        })
    } else {
        &style.fills
    }
}

pub(crate) fn resolve_path_stroke_paints(
    stroke: &style_engine::StrokeStyle,
) -> &[style_engine::Paint] {
    if stroke.paints.is_empty() {
        DEFAULT_STROKE.get_or_init(|| {
            vec![style_engine::Paint::solid(glam::Vec4::new(
                1.0, 0.0, 1.0, 1.0,
            ))]
        })
    } else {
        &stroke.paints
    }
}

/// Collect instances from the scene.
///
/// Returns a tuple of (primitive_instances, text_nodes_for_shaping, path_batches).
/// Text nodes are extracted from VisualStyle and returned for shaping.
pub(crate) fn collect_instances<'a>(
    pipeline: &mut PrimitivePipeline,
    tessellation_cache: &mut TessellationCache,
    path_interner: &mut PathInterner,
    stack: &mut Vec<(crate::NodeId, f32)>,
    scene: &'a Scene,
) -> (
    Vec<PrimitiveInstance>,
    Vec<TextNodeData<'a>>,
    Vec<PathBatch<'a>>,
) {
    collect_instances_impl(
        Some(pipeline),
        Some(tessellation_cache),
        Some(path_interner),
        stack,
        scene,
        false,
    )
}

pub(crate) fn collect_instances_excluding_multipass<'a>(
    pipeline: &mut PrimitivePipeline,
    tessellation_cache: &mut TessellationCache,
    path_interner: &mut PathInterner,
    stack: &mut Vec<(crate::NodeId, f32)>,
    scene: &'a Scene,
) -> (
    Vec<PrimitiveInstance>,
    Vec<TextNodeData<'a>>,
    Vec<PathBatch<'a>>,
) {
    collect_instances_impl(
        Some(pipeline),
        Some(tessellation_cache),
        Some(path_interner),
        stack,
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
    Vec<PathBatch<'a>>,
) {
    let mut stack = Vec::new();
    collect_instances_impl(None, None, None, &mut stack, scene, false)
}

#[cfg(test)]
pub(crate) fn collect_instances_without_multipass_for_tests<'a>(
    scene: &'a Scene,
) -> (
    Vec<PrimitiveInstance>,
    Vec<TextNodeData<'a>>,
    Vec<PathBatch<'a>>,
) {
    let mut stack = Vec::new();
    collect_instances_impl(None, None, None, &mut stack, scene, true)
}

fn collect_path_geometry_batches<'a>(
    style: &'a style_engine::VisualStyle,
    render_bounds: &plat_core::Rect,
    effective_opacity: f32,
    tessellation_cache: &mut Option<&mut TessellationCache>,
    path_interner: &mut Option<&mut PathInterner>,
    path_batches: &mut Vec<PathBatch<'a>>,
) {
    let Some(paths) = &style.fill_geometry else {
        return;
    };

    let fill_paints = resolve_path_fill_paints(style);
    let mut fill_meshes = Vec::new();
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
            fill_meshes.push(mesh);
        }
    }

    for fill_paint in fill_paints {
        for mesh in &fill_meshes {
            path_batches.push(PathBatch {
                mesh: Arc::clone(mesh),
                paint: fill_paint,
                opacity: effective_opacity,
                size: [render_bounds.width, render_bounds.height],
                offset: [render_bounds.x, render_bounds.y],
            });
        }
    }

    if let Some(stroke) = &style.stroke {
        let stroke_paints = resolve_path_stroke_paints(stroke);
        if let Some(stroke_paths) = style
            .stroke_geometry
            .as_ref()
            .or(style.fill_geometry.as_ref())
        {
            let mut stroke_meshes = Vec::new();
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
                if let Ok(mesh) = mesh_result
                    && !mesh.indices.is_empty()
                {
                    stroke_meshes.push(mesh);
                }
            }

            for stroke_paint in stroke_paints {
                for mesh in &stroke_meshes {
                    path_batches.push(PathBatch {
                        mesh: Arc::clone(mesh),
                        paint: stroke_paint,
                        opacity: effective_opacity,
                        size: [render_bounds.width, render_bounds.height],
                        offset: [render_bounds.x, render_bounds.y],
                    });
                }
            }
        }
    }
}

fn collect_image_fill_batches<'a>(
    style: &'a style_engine::VisualStyle,
    render_bounds: &plat_core::Rect,
    effective_opacity: f32,
    tessellation_cache: &mut Option<&mut TessellationCache>,
    path_batches: &mut Vec<PathBatch<'a>>,
) {
    let fill_paints = resolve_path_fill_paints(style);
    let rect_path = rounded_rect_path_for_size(
        render_bounds.width,
        render_bounds.height,
        style.corner_radii,
    );
    let path_hash = TessellationCache::fill_key(&rect_path);
    let mesh_result = if let Some(cache) = tessellation_cache.as_deref_mut() {
        cache.get_or_tessellate_fill_with_key(path_hash, &rect_path)
    } else {
        tessellate_fill(&rect_path).map(Arc::new)
    };
    if let Ok(mesh) = mesh_result
        && !mesh.indices.is_empty()
    {
        for fill_paint in fill_paints {
            path_batches.push(PathBatch {
                mesh: Arc::clone(&mesh),
                paint: fill_paint,
                opacity: effective_opacity,
                size: [render_bounds.width, render_bounds.height],
                offset: [render_bounds.x, render_bounds.y],
            });
        }
    }

    if let Some(stroke) = &style.stroke {
        let stroke_paints = resolve_path_stroke_paints(stroke);
        let stroke_key = TessellationCache::stroke_key_from_path_hash(path_hash, stroke);
        let stroke_mesh_result = if let Some(cache) = tessellation_cache.as_deref_mut() {
            cache.get_or_tessellate_stroke_with_key(stroke_key, &rect_path, stroke)
        } else {
            tessellate_stroke(&rect_path, stroke).map(Arc::new)
        };
        if let Ok(mesh) = stroke_mesh_result
            && !mesh.indices.is_empty()
        {
            for stroke_paint in stroke_paints {
                path_batches.push(PathBatch {
                    mesh: Arc::clone(&mesh),
                    paint: stroke_paint,
                    opacity: effective_opacity,
                    size: [render_bounds.width, render_bounds.height],
                    offset: [render_bounds.x, render_bounds.y],
                });
            }
        }
    }
}

fn collect_instances_impl<'a>(
    mut pipeline: Option<&mut PrimitivePipeline>,
    mut tessellation_cache: Option<&mut TessellationCache>,
    mut path_interner: Option<&mut PathInterner>,
    stack: &mut Vec<(crate::NodeId, f32)>,
    scene: &'a Scene,
    skip_multipass: bool,
) -> (
    Vec<PrimitiveInstance>,
    Vec<TextNodeData<'a>>,
    Vec<PathBatch<'a>>,
) {
    // We need style_requires_multipass but it is in mod.rs (or multipass_executor.rs later).
    // For now I will inline or import it.
    // It seems it is better to move it to a shared place or redefine it here since it is pure logic on VisualStyle.
    // Or I can pass a predicate.

    // I will duplicate `style_requires_multipass` for now as I plan to move it to `multipass_executor` which is next.
    // But `instance_collector` needs it.
    // Maybe `style_requires_multipass` should be in `effects.rs` or `style-engine`.

    // Pre-allocate to reduce heap re-allocations during iteration.
    let capacity = scene.node_count().clamp(16, 4096);
    let mut instances = Vec::with_capacity(capacity);
    let mut text_nodes_for_shaping = Vec::with_capacity(capacity / 4);
    let mut path_batches = Vec::with_capacity(capacity / 4);

    for (node_id, node, inherited_opacity) in scene.iter_visuals_custom(stack) {
        if !node.visible {
            continue;
        }
        if inherited_opacity <= 0.0 {
            continue;
        }

        match &node.content {
            NodeContent::Styled { style } => {
                if skip_multipass && style_requires_multipass(style) {
                    continue;
                }
                let effective_opacity = inherited_opacity * style.opacity;
                let Some(render_bounds) = clipped_bounds_for_node(scene, node_id, node.bounds)
                else {
                    continue;
                };

                if style.fill_geometry.is_some() {
                    collect_path_geometry_batches(
                        style,
                        &render_bounds,
                        effective_opacity,
                        &mut tessellation_cache,
                        &mut path_interner,
                        &mut path_batches,
                    );
                    continue;
                }

                // Route image-filled rectangles through path batches so Paint::Image
                // samples are resolved by the path pipeline instead of magenta fallback.
                if style
                    .fills
                    .iter()
                    .any(|fill| matches!(fill, style_engine::Paint::Image(_)))
                {
                    collect_image_fill_batches(
                        style,
                        &render_bounds,
                        effective_opacity,
                        &mut tessellation_cache,
                        &mut path_batches,
                    );
                    continue;
                }

                // Check if this style has text that needs shaping
                if let Some(text_content) = &style.text {
                    text_nodes_for_shaping.push((
                        node_id,
                        node,
                        text_content,
                        effective_opacity,
                        style.as_ref(),
                    ));
                }

                // Create primitive instances for this style
                let pos = glam::Vec2::new(render_bounds.x, render_bounds.y);
                let size = glam::Vec2::new(render_bounds.width, render_bounds.height);
                let start_idx = instances.len();
                if let Some(pipeline) = pipeline.as_deref_mut() {
                    create_primitive_instances_with_pipeline(
                        pipeline,
                        style,
                        pos,
                        size,
                        effective_opacity,
                        &mut instances,
                    )
                } else {
                    create_primitive_instances(style, pos, size, effective_opacity, &mut instances)
                };
                apply_node_transform_to_instances(&mut instances[start_idx..], node.transform);
            }
            NodeContent::SolidColor { color } => {
                if let Some(render_bounds) = clipped_bounds_for_node(scene, node_id, node.bounds) {
                    let mut final_color = color.to_array();
                    final_color[3] *= inherited_opacity;

                    let mut solid = PrimitiveInstance::solid(
                        [render_bounds.x, render_bounds.y],
                        [render_bounds.width, render_bounds.height],
                        final_color,
                    );
                    apply_node_transform_to_instances(
                        std::slice::from_mut(&mut solid),
                        node.transform,
                    );
                    instances.push(solid);
                }
            }
            NodeContent::Empty => {}
        }
    }
    (instances, text_nodes_for_shaping, path_batches)
}

pub(crate) fn collect_style_batches_for_bounds<'a>(
    ctx: &mut BatchCollectionContext,
    style: &'a style_engine::VisualStyle,
    effective_opacity: f32,
    render_bounds: plat_core::Rect,
    node_transform: Transform2D,
    instances: &mut Vec<PrimitiveInstance>,
    path_batches: &mut Vec<PathBatch<'a>>,
) {
    instances.clear();
    path_batches.clear();

    if style.fill_geometry.is_some() {
        let mut cache: Option<&mut TessellationCache> = Some(&mut *ctx.tessellation_cache);
        let mut interner: Option<&mut PathInterner> = Some(&mut *ctx.path_interner);
        collect_path_geometry_batches(
            style,
            &render_bounds,
            effective_opacity,
            &mut cache,
            &mut interner,
            path_batches,
        );
    } else if style
        .fills
        .iter()
        .any(|fill| matches!(fill, style_engine::Paint::Image(_)))
    {
        let mut cache: Option<&mut TessellationCache> = Some(&mut *ctx.tessellation_cache);
        collect_image_fill_batches(
            style,
            &render_bounds,
            effective_opacity,
            &mut cache,
            path_batches,
        );
    } else {
        let pos = glam::Vec2::new(render_bounds.x, render_bounds.y);
        let size = glam::Vec2::new(render_bounds.width, render_bounds.height);
        create_primitive_instances_with_pipeline(
            ctx.pipeline,
            style,
            pos,
            size,
            effective_opacity,
            instances,
        );
    }

    if let Some(text_content) = &style.text
        && !text_content.text.is_empty()
    {
        let options = text_shape_options(text_content);
        let mut shaped = ctx.text_renderer.text_engine_mut().shape_text_with_options(
            &text_content.text,
            text_content.font_size,
            options,
        );
        apply_text_letter_spacing(&mut shaped, text_content.letter_spacing);
        let position = glam::Vec2::new(
            render_bounds.x.round(),
            (render_bounds.y + text_content.font_size).round(),
        );
        let text_bounds = [
            render_bounds.x,
            render_bounds.y,
            render_bounds.width,
            render_bounds.height,
        ];
        let fill = resolve_text_fill(ctx.pipeline, style, effective_opacity, text_bounds);
        if !text_has_background_surface(style) {
            let shadow_layers = text_shadow_layers(style, effective_opacity);
            for (offset, shadow_fill) in shadow_layers {
                let shadow_position = position + offset;
                let start_idx = instances.len();
                ctx.text_renderer.generate_instances_into(
                    &shaped,
                    shadow_position,
                    glam::Vec4::ONE,
                    instances,
                );
                for instance in &mut instances[start_idx..] {
                    apply_text_fill_to_glyph(instance, shadow_fill);
                }
            }
        }
        let start_idx = instances.len();
        ctx.text_renderer
            .generate_instances_into(&shaped, position, glam::Vec4::ONE, instances);
        for instance in &mut instances[start_idx..] {
            apply_text_fill_to_glyph(instance, fill);
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

    apply_node_transform_to_instances(instances, node_transform);
}

#[cfg(test)]
mod tests {
    use super::{
        TextFill, apply_node_transform_to_instances, apply_text_letter_spacing,
        pick_text_fill_paint, text_has_background_surface, text_shadow_layers, text_shape_options,
    };
    use crate::Transform2D;
    use crate::primitives::PrimitiveInstance;
    use glam::Vec2;
    use glam::Vec4;
    use style_engine::{Paint, TextContent, VisualStyle};
    use text_engine::TextEngine;
    use text_engine::TextFontStyle;

    #[test]
    fn pick_text_fill_prefers_first_non_transparent_solid() {
        let style = VisualStyle::new()
            .fill(Paint::solid(Vec4::new(0.0, 0.0, 0.0, 1.0)))
            .fill(Paint::solid(Vec4::new(1.0, 1.0, 1.0, 1.0)));

        let paint = pick_text_fill_paint(&style).expect("expected a text fill");
        let Paint::Solid(color) = paint else {
            panic!("expected selected text fill to be solid");
        };
        assert_eq!(*color, Vec4::new(0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn pick_text_fill_skips_fully_transparent_first_solid() {
        let style = VisualStyle::new()
            .fill(Paint::solid(Vec4::new(1.0, 1.0, 1.0, 0.0)))
            .fill(Paint::solid(Vec4::new(0.2, 0.3, 0.4, 1.0)));

        let paint = pick_text_fill_paint(&style).expect("expected selected text fill");
        let Paint::Solid(color) = paint else {
            panic!("expected selected text fill to be solid");
        };
        assert_eq!(*color, Vec4::new(0.2, 0.3, 0.4, 1.0));
    }

    #[test]
    fn text_shape_options_maps_family_weight_and_style() {
        let mut text = TextContent::new("icon", 24.0)
            .family("  Material Symbols Outlined  ")
            .weight(700);
        text.font_style = style_engine::FontStyle::Italic;

        let options = text_shape_options(&text);
        assert_eq!(options.family, Some("Material Symbols Outlined"));
        assert_eq!(options.weight, Some(700));
        assert_eq!(options.style, TextFontStyle::Italic);
    }

    #[test]
    fn apply_node_transform_to_instances_rotates_centers_and_sets_angle() {
        let mut instances = vec![PrimitiveInstance::solid(
            [10.0, 10.0],
            [20.0, 10.0],
            [1.0, 0.0, 0.0, 1.0],
        )];
        let transform = Transform2D::translate(20.0, 15.0)
            .compose(&Transform2D::rotate_radians(10.0_f32.to_radians()))
            .compose(&Transform2D::translate(-20.0, -15.0));

        let original_center = glam::Vec2::new(20.0, 15.0);
        apply_node_transform_to_instances(&mut instances, transform);

        let instance = &instances[0];
        let transformed_center = glam::Vec2::new(
            instance.pos[0] + instance.size[0] * 0.5,
            instance.pos[1] + instance.size[1] * 0.5,
        );
        assert!((transformed_center - transform.transform_point(original_center)).length() < 1e-4);
        assert!((instance.rotation_radians() - 10.0_f32.to_radians()).abs() < 1e-6);
    }

    #[test]
    fn text_shadow_layers_emits_core_and_blur_halo_layers() {
        let style = VisualStyle::new()
            .text(TextContent::new("THE PIT", 20.0))
            .drop_shadow(Vec2::new(4.0, 4.0), 5.0, Vec4::new(0.8, 1.0, 0.0, 1.0));

        let layers = text_shadow_layers(&style, 0.5);
        assert_eq!(layers.len(), 5, "expected 4 halo layers plus core layer");

        let core = layers
            .iter()
            .find(|(offset, _)| (*offset - Vec2::new(4.0, 4.0)).length() < 1e-4)
            .expect("missing core shadow layer");
        let TextFill::Solid(core_color) = core.1 else {
            panic!("core shadow layer should be solid");
        };
        assert!((core_color.w - 0.5).abs() < 1e-5);
    }

    #[test]
    fn text_has_background_surface_only_when_secondary_fill_is_visible() {
        let with_surface = VisualStyle::new()
            .text(TextContent::new("SCUM", 72.0))
            .fill(Paint::solid(Vec4::new(0.0, 0.0, 0.0, 1.0)))
            .fill(Paint::solid(Vec4::new(1.0, 1.0, 1.0, 1.0)));
        assert!(
            text_has_background_surface(&with_surface),
            "second visible fill should be treated as text background surface"
        );

        let transparent_secondary = VisualStyle::new()
            .text(TextContent::new("SCUM", 72.0))
            .fill(Paint::solid(Vec4::new(0.0, 0.0, 0.0, 1.0)))
            .fill(Paint::solid(Vec4::new(1.0, 1.0, 1.0, 0.0)));
        assert!(
            !text_has_background_surface(&transparent_secondary),
            "fully transparent secondary fill should not count as a text background surface"
        );

        let no_secondary = VisualStyle::new()
            .text(TextContent::new("SCUM", 72.0))
            .fill(Paint::solid(Vec4::new(0.0, 0.0, 0.0, 1.0)));
        assert!(
            !text_has_background_surface(&no_secondary),
            "single fill text should render glyph-space shadows"
        );
    }

    #[test]
    fn apply_text_letter_spacing_offsets_following_glyphs() {
        let mut engine = TextEngine::new();
        let mut shaped = engine.shape_text("TEST", 24.0);
        assert!(
            shaped.glyphs.len() >= 2,
            "expected at least two glyphs in shaped text"
        );
        let second_before = shaped.glyphs[1].x_offset;
        let width_before = shaped.bounds.width;

        apply_text_letter_spacing(&mut shaped, 2.0);

        assert!(
            shaped.glyphs[1].x_offset > second_before + 1.5,
            "second glyph should shift by letter spacing"
        );
        assert!(shaped.bounds.width > width_before + 5.0);
    }
}
