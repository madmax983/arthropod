use glam::Vec2;
use style_engine::{BlendMode, ColorStop, CornerRadii, Paint, VisualStyle};

use super::gradient_atlas::{GradientAtlas, GradientParams};
use super::primitive_pipeline::PrimitivePipeline;
use crate::primitives::{
    FLAG_HAS_STROKE, FLAG_IS_SHADOW, PrimitiveInstance, with_blend_mode, with_fill_type,
    with_stroke_cap_join,
};

fn paint_has_visible_alpha(paint: &Paint) -> bool {
    match paint {
        Paint::Solid(color) => color.w > f32::EPSILON,
        Paint::Linear(gradient) => gradient
            .stops
            .iter()
            .any(|stop| stop.color.w > f32::EPSILON),
        Paint::Radial(gradient) => gradient
            .stops
            .iter()
            .any(|stop| stop.color.w > f32::EPSILON),
        Paint::Angular(gradient) => gradient
            .stops
            .iter()
            .any(|stop| stop.color.w > f32::EPSILON),
        Paint::Diamond(gradient) => gradient
            .stops
            .iter()
            .any(|stop| stop.color.w > f32::EPSILON),
        Paint::Image(_) => true,
    }
}

/// Convert a VisualStyle into one or more PrimitiveInstances.
///
/// A single styled node may generate multiple instances:
/// - One per fill (solid/gradient)
/// - One for stroke (if present)
/// - One per effect (shadows, blur)
/// - Text glyphs (if text is present)
fn create_primitive_instances_impl(
    mut pipeline: Option<&mut PrimitivePipeline>,
    style: &VisualStyle,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
) -> Vec<PrimitiveInstance> {
    let mut instances = Vec::new();

    // For text styles, fill[0] is treated as the glyph paint.
    // Any additional visible paints are treated as sticker/background surfaces.
    let text_has_background_surface =
        style.text.is_some() && style.fills.iter().skip(1).any(paint_has_visible_alpha);
    let fills_to_render: &[Paint] = if style.text.is_some() {
        if style.fills.len() > 1 {
            &style.fills[1..]
        } else {
            &[]
        }
    } else {
        &style.fills
    };

    // 1. Render shadows FIRST (behind everything)
    for effect in &style.effects {
        if style.text.is_some() && !text_has_background_surface {
            // Text-only drop shadows should be handled in glyph space, not as
            // rectangular bounds shadows.
            continue;
        }
        if let style_engine::Effect::DropShadow(shadow) = effect
            && shadow.visible
        {
            // Create shadow instance with offset position
            let shadow_pos = pos + shadow.offset;

            let mut shadow_color = shadow.color;
            shadow_color.w *= opacity;

            let mut shadow_instance = PrimitiveInstance::rounded(
                [shadow_pos.x, shadow_pos.y],
                [size.x, size.y],
                [
                    shadow_color.x,
                    shadow_color.y,
                    shadow_color.z,
                    shadow_color.w,
                ],
                style.corner_radii,
            );

            // Store blur radius in stroke_params[0] (shadows don't use stroke)
            shadow_instance.stroke_params[0] = shadow.blur;

            // Set blend mode and internal shadow flag
            shadow_instance.flags = with_blend_mode(shadow_instance.flags, style.blend_mode);
            shadow_instance.flags |= FLAG_IS_SHADOW;

            instances.push(shadow_instance);
        }
    }

    // 2. Render fills (solid or gradient backgrounds)
    for fill in fills_to_render {
        let fill_instance = if let Some(pipeline) = pipeline.as_mut() {
            create_fill_instance(
                pipeline,
                fill,
                pos,
                size,
                opacity,
                &style.corner_radii,
                style.blend_mode,
            )
        } else {
            create_fill_instance_without_pipeline(
                fill,
                pos,
                size,
                opacity,
                &style.corner_radii,
                style.blend_mode,
            )
        };
        instances.push(fill_instance);
    }

    // 3. Render stroke paints bottom-to-top (on top of fills)
    if let Some(stroke) = &style.stroke {
        let stroke_instances = if let Some(pipeline) = pipeline.as_mut() {
            create_stroke_instances(
                pipeline,
                stroke,
                pos,
                size,
                opacity,
                &style.corner_radii,
                style.blend_mode,
            )
        } else {
            create_stroke_instances_without_pipeline(
                stroke,
                pos,
                size,
                opacity,
                &style.corner_radii,
                style.blend_mode,
            )
        };
        instances.extend(stroke_instances);
    }

    instances
}

pub fn create_primitive_instances(
    style: &VisualStyle,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
) -> Vec<PrimitiveInstance> {
    create_primitive_instances_impl(None, style, pos, size, opacity)
}

pub fn create_primitive_instances_with_pipeline(
    pipeline: &mut PrimitivePipeline,
    style: &VisualStyle,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
) -> Vec<PrimitiveInstance> {
    create_primitive_instances_impl(Some(pipeline), style, pos, size, opacity)
}

/// Create one stroke instance per stroke paint layer.
fn create_stroke_instances(
    pipeline: &mut PrimitivePipeline,
    stroke: &style_engine::StrokeStyle,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
    corner_radii: &CornerRadii,
    blend_mode: BlendMode,
) -> Vec<PrimitiveInstance> {
    use style_engine::StrokeAlign;

    let stroke_width = stroke.weight;
    let stroke_align = match stroke.align {
        StrokeAlign::Center => 0.0,
        StrokeAlign::Inside => 1.0,
        StrokeAlign::Outside => -1.0,
    };

    let mut instances = Vec::new();
    if stroke.paints.is_empty() {
        let mut instance = PrimitiveInstance::rounded(
            [pos.x, pos.y],
            [size.x, size.y],
            [1.0, 0.0, 1.0, opacity],
            *corner_radii,
        );
        instance.stroke_params = [stroke_width, stroke_align];
        instance.flags |= FLAG_HAS_STROKE;
        instance.flags = with_blend_mode(instance.flags, blend_mode);
        instance.flags = with_stroke_cap_join(instance.flags, stroke.cap, stroke.join);
        instances.push(instance);
        return instances;
    }

    for stroke_paint in &stroke.paints {
        let mut instance = match stroke_paint {
            Paint::Solid(color) => {
                let mut final_color = *color;
                final_color.w *= opacity;

                PrimitiveInstance::rounded(
                    [pos.x, pos.y],
                    [size.x, size.y],
                    [final_color.x, final_color.y, final_color.z, final_color.w],
                    *corner_radii,
                )
            }
            Paint::Linear(gradient) => {
                let atlas_row = pipeline.add_gradient(&gradient.stops);
                let params = GradientParams {
                    start: gradient.start.to_array(),
                    end: gradient.end.to_array(),
                    atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
                    gradient_type: 0,
                    _padding: [0.0; 2],
                };
                let param_index = pipeline.add_gradient_params(params);

                let mut instance = PrimitiveInstance::rounded(
                    [pos.x, pos.y],
                    [size.x, size.y],
                    [1.0, 1.0, 1.0, opacity],
                    *corner_radii,
                );
                instance.gradient_params = [param_index as f32, 0.0, 0.0, 0.0];
                instance.flags = with_fill_type(instance.flags, 1);
                instance
            }
            _ => {
                // Non-linear gradient/image stroke paint fallback in primitive stroke path.
                PrimitiveInstance::rounded(
                    [pos.x, pos.y],
                    [size.x, size.y],
                    [1.0, 1.0, 1.0, opacity],
                    *corner_radii,
                )
            }
        };

        instance.stroke_params = [stroke_width, stroke_align];
        instance.flags |= FLAG_HAS_STROKE;
        instance.flags = with_blend_mode(instance.flags, blend_mode);
        instance.flags = with_stroke_cap_join(instance.flags, stroke.cap, stroke.join);
        instances.push(instance);
    }

    instances
}

fn fallback_gradient_color(stops: &[ColorStop], opacity: f32) -> [f32; 4] {
    let mut color = style_engine::Paint::interpolate_stops(0.5, stops);
    color.w *= opacity;
    [color.x, color.y, color.z, color.w]
}

fn create_stroke_instances_without_pipeline(
    stroke: &style_engine::StrokeStyle,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
    corner_radii: &CornerRadii,
    blend_mode: BlendMode,
) -> Vec<PrimitiveInstance> {
    use style_engine::StrokeAlign;

    let stroke_width = stroke.weight;
    let stroke_align = match stroke.align {
        StrokeAlign::Center => 0.0,
        StrokeAlign::Inside => 1.0,
        StrokeAlign::Outside => -1.0,
    };

    let mut instances = Vec::new();
    if stroke.paints.is_empty() {
        let mut instance = PrimitiveInstance::rounded(
            [pos.x, pos.y],
            [size.x, size.y],
            [1.0, 0.0, 1.0, opacity],
            *corner_radii,
        );
        instance.stroke_params = [stroke_width, stroke_align];
        instance.flags |= FLAG_HAS_STROKE;
        instance.flags = with_blend_mode(instance.flags, blend_mode);
        instance.flags = with_stroke_cap_join(instance.flags, stroke.cap, stroke.join);
        instances.push(instance);
        return instances;
    }

    for stroke_paint in &stroke.paints {
        let stroke_color = match stroke_paint {
            Paint::Solid(color) => {
                let mut final_color = *color;
                final_color.w *= opacity;
                [final_color.x, final_color.y, final_color.z, final_color.w]
            }
            Paint::Linear(gradient) => fallback_gradient_color(&gradient.stops, opacity),
            Paint::Radial(gradient) => fallback_gradient_color(&gradient.stops, opacity),
            Paint::Angular(gradient) => fallback_gradient_color(&gradient.stops, opacity),
            Paint::Diamond(gradient) => fallback_gradient_color(&gradient.stops, opacity),
            Paint::Image(_) => [1.0, 0.0, 1.0, opacity],
        };

        let mut instance = PrimitiveInstance::rounded(
            [pos.x, pos.y],
            [size.x, size.y],
            stroke_color,
            *corner_radii,
        );
        instance.stroke_params = [stroke_width, stroke_align];
        instance.flags |= FLAG_HAS_STROKE;
        instance.flags = with_blend_mode(instance.flags, blend_mode);
        instance.flags = with_stroke_cap_join(instance.flags, stroke.cap, stroke.join);
        instances.push(instance);
    }

    instances
}

/// Create a single fill instance (solid or gradient)
fn create_fill_instance(
    pipeline: &mut PrimitivePipeline,
    fill: &Paint,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
    corner_radii: &CornerRadii,
    blend_mode: BlendMode,
) -> PrimitiveInstance {
    let mut instance = match fill {
        Paint::Solid(color) => {
            let mut final_color = *color;
            final_color.w *= opacity;

            PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [final_color.x, final_color.y, final_color.z, final_color.w],
                *corner_radii,
            )
        }
        Paint::Linear(gradient) => {
            // Add gradient to atlas
            let atlas_row = pipeline.add_gradient(&gradient.stops);

            // Create gradient params
            let params = GradientParams {
                start: gradient.start.to_array(),
                end: gradient.end.to_array(),
                atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
                gradient_type: 0, // Linear
                _padding: [0.0; 2],
            };
            let param_index = pipeline.add_gradient_params(params);

            // Create instance with gradient
            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [1.0, 1.0, 1.0, opacity], // Color unused for gradients, just alpha
                *corner_radii,
            );
            instance.gradient_params = [param_index as f32, 0.0, 0.0, 0.0];
            instance.flags = with_fill_type(instance.flags, 1);
            instance
        }
        Paint::Radial(gradient) => {
            let atlas_row = pipeline.add_gradient(&gradient.stops);
            let params = GradientParams {
                start: gradient.center.to_array(),
                end: (gradient.center + Vec2::new(gradient.radius, 0.0)).to_array(), // End = center + radius vector
                atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
                gradient_type: 1, // Radial
                _padding: [0.0; 2],
            };
            let param_index = pipeline.add_gradient_params(params);

            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [1.0, 1.0, 1.0, opacity],
                *corner_radii,
            );
            instance.gradient_params = [param_index as f32, 0.0, 0.0, 0.0];
            instance.flags = with_fill_type(instance.flags, 2);
            instance
        }
        Paint::Angular(gradient) => {
            let atlas_row = pipeline.add_gradient(&gradient.stops);
            let params = GradientParams {
                start: gradient.center.to_array(),
                end: gradient.center.to_array(), // End unused for angular
                atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
                gradient_type: 2, // Angular
                _padding: [0.0; 2],
            };
            let param_index = pipeline.add_gradient_params(params);

            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [1.0, 1.0, 1.0, opacity],
                *corner_radii,
            );
            instance.gradient_params = [param_index as f32, 0.0, 0.0, 0.0];
            instance.flags = with_fill_type(instance.flags, 3);
            instance
        }
        Paint::Diamond(gradient) => {
            let atlas_row = pipeline.add_gradient(&gradient.stops);
            let params = GradientParams {
                start: gradient.center.to_array(),
                end: (gradient.center + Vec2::new(gradient.scale, gradient.scale)).to_array(),
                atlas_row: (atlas_row as f32 + 0.5) / GradientAtlas::ATLAS_SIZE as f32,
                gradient_type: 3, // Diamond
                _padding: [0.0; 2],
            };
            let param_index = pipeline.add_gradient_params(params);

            let mut instance = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [1.0, 1.0, 1.0, opacity],
                *corner_radii,
            );
            instance.gradient_params = [param_index as f32, 0.0, 0.0, 0.0];
            instance.flags = with_fill_type(instance.flags, 4);
            instance
        }
        Paint::Image(_) => {
            // Image fills will be implemented in Phase 3
            PrimitiveInstance::solid([pos.x, pos.y], [size.x, size.y], [1.0, 0.0, 1.0, 1.0])
        }
    };
    instance.flags = with_blend_mode(instance.flags, blend_mode);
    instance
}

fn create_fill_instance_without_pipeline(
    fill: &Paint,
    pos: Vec2,
    size: Vec2,
    opacity: f32,
    corner_radii: &CornerRadii,
    blend_mode: BlendMode,
) -> PrimitiveInstance {
    let mut instance = match fill {
        Paint::Solid(color) => {
            let mut final_color = *color;
            final_color.w *= opacity;

            PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                [final_color.x, final_color.y, final_color.z, final_color.w],
                *corner_radii,
            )
        }
        Paint::Linear(gradient) => {
            let mut inst = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                fallback_gradient_color(&gradient.stops, opacity),
                *corner_radii,
            );
            inst.flags = with_fill_type(inst.flags, 1);
            inst
        }
        Paint::Radial(gradient) => {
            let mut inst = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                fallback_gradient_color(&gradient.stops, opacity),
                *corner_radii,
            );
            inst.flags = with_fill_type(inst.flags, 2);
            inst
        }
        Paint::Angular(gradient) => {
            let mut inst = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                fallback_gradient_color(&gradient.stops, opacity),
                *corner_radii,
            );
            inst.flags = with_fill_type(inst.flags, 3);
            inst
        }
        Paint::Diamond(gradient) => {
            let mut inst = PrimitiveInstance::rounded(
                [pos.x, pos.y],
                [size.x, size.y],
                fallback_gradient_color(&gradient.stops, opacity),
                *corner_radii,
            );
            inst.flags = with_fill_type(inst.flags, 4);
            inst
        }
        Paint::Image(_) => {
            PrimitiveInstance::solid([pos.x, pos.y], [size.x, size.y], [1.0, 0.0, 1.0, 1.0])
        }
    };
    instance.flags = with_blend_mode(instance.flags, blend_mode);
    instance
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::wgpu::pipelines::primitive_instance::{
        FLAG_BLEND_MODE_MASK, FLAG_BLEND_MODE_SHIFT, FLAG_FILL_TYPE_MASK, FLAG_HAS_STROKE,
        FLAG_STROKE_CAP_MASK, FLAG_STROKE_CAP_SHIFT, FLAG_STROKE_JOIN_MASK, FLAG_STROKE_JOIN_SHIFT,
    };
    use glam::Vec4;

    #[test]
    fn test_gradient_fill_types_are_distinct() {
        let linear = VisualStyle::new().fill(Paint::Linear(style_engine::LinearGradient {
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(1.0, 0.0),
            stops: vec![
                ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
                ColorStop::new(1.0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
            ],
        }));
        let radial = VisualStyle::new().fill(Paint::Radial(style_engine::RadialGradient {
            center: Vec2::new(0.5, 0.5),
            radius: 0.5,
            stops: vec![
                ColorStop::new(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                ColorStop::new(1.0, Vec4::new(0.0, 0.0, 0.0, 1.0)),
            ],
        }));
        let angular = VisualStyle::new().fill(Paint::Angular(style_engine::AngularGradient {
            center: Vec2::new(0.5, 0.5),
            angle: 0.0,
            stops: vec![
                ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
                ColorStop::new(1.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
            ],
        }));
        let diamond = VisualStyle::new().fill(Paint::Diamond(style_engine::DiamondGradient {
            center: Vec2::new(0.5, 0.5),
            scale: 0.5,
            stops: vec![
                ColorStop::new(0.0, Vec4::new(1.0, 1.0, 0.0, 1.0)),
                ColorStop::new(1.0, Vec4::new(0.5, 0.0, 0.5, 1.0)),
            ],
        }));

        let linear_i = create_primitive_instances(&linear, Vec2::ZERO, Vec2::ONE, 1.0);
        let radial_i = create_primitive_instances(&radial, Vec2::ZERO, Vec2::ONE, 1.0);
        let angular_i = create_primitive_instances(&angular, Vec2::ZERO, Vec2::ONE, 1.0);
        let diamond_i = create_primitive_instances(&diamond, Vec2::ZERO, Vec2::ONE, 1.0);

        assert_eq!(linear_i[0].flags & FLAG_FILL_TYPE_MASK, 1);
        assert_eq!(radial_i[0].flags & FLAG_FILL_TYPE_MASK, 2);
        assert_eq!(angular_i[0].flags & FLAG_FILL_TYPE_MASK, 3);
        assert_eq!(diamond_i[0].flags & FLAG_FILL_TYPE_MASK, 4);
    }

    #[test]
    fn test_shadow_flag_uses_internal_high_bit() {
        let style = VisualStyle::new()
            .solid_fill(Vec4::new(1.0, 1.0, 1.0, 1.0))
            .drop_shadow(Vec2::new(2.0, 2.0), 6.0, Vec4::new(0.0, 0.0, 0.0, 0.3));
        let instances = create_primitive_instances(&style, Vec2::ZERO, Vec2::ONE, 1.0);

        assert!(instances.iter().any(|i| (i.flags & FLAG_IS_SHADOW) != 0));
    }

    #[test]
    fn test_stroke_packs_cap_join_and_blend_mode_bits() {
        let mut stroke = style_engine::StrokeStyle::solid(
            Paint::solid(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            2.0,
            style_engine::StrokeAlign::Inside,
        );
        stroke.cap = style_engine::StrokeCap::Square;
        stroke.join = style_engine::StrokeJoin::Round;

        let style = VisualStyle::new()
            .solid_fill(Vec4::new(0.2, 0.2, 0.2, 1.0))
            .blend_mode(style_engine::BlendMode::Screen)
            .stroke(stroke);
        let instances = create_primitive_instances(&style, Vec2::ZERO, Vec2::ONE, 1.0);
        let stroke_instance = instances
            .iter()
            .find(|i| (i.flags & FLAG_HAS_STROKE) != 0)
            .expect("missing stroke instance");

        let blend = (stroke_instance.flags & FLAG_BLEND_MODE_MASK) >> FLAG_BLEND_MODE_SHIFT;
        let cap = (stroke_instance.flags & FLAG_STROKE_CAP_MASK) >> FLAG_STROKE_CAP_SHIFT;
        let join = (stroke_instance.flags & FLAG_STROKE_JOIN_MASK) >> FLAG_STROKE_JOIN_SHIFT;

        assert_eq!(blend, style_engine::BlendMode::Screen.to_flag_bits() as u32);
        assert_eq!(cap, 2);
        assert_eq!(join, 2);
    }

    #[test]
    fn test_create_primitive_instances_emits_all_stroke_paints_in_order() {
        let stroke = style_engine::StrokeStyle {
            paints: vec![
                Paint::solid(Vec4::new(1.0, 0.0, 0.0, 1.0)),
                Paint::solid(Vec4::new(0.0, 0.0, 1.0, 1.0)),
            ],
            weight: 2.0,
            align: style_engine::StrokeAlign::Center,
            ..Default::default()
        };
        let style = VisualStyle::new().stroke(stroke);

        let instances = create_primitive_instances(&style, Vec2::ZERO, Vec2::ONE, 1.0);
        let stroke_instances: Vec<_> = instances
            .iter()
            .filter(|instance| (instance.flags & FLAG_HAS_STROKE) != 0)
            .collect();

        assert_eq!(
            stroke_instances.len(),
            2,
            "expected one instance per stroke paint"
        );
        assert_eq!(stroke_instances[0].color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(stroke_instances[1].color, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn test_create_primitive_instances_solid() {
        let style = VisualStyle::new()
            .solid_fill(Vec4::new(1.0, 0.0, 0.0, 1.0))
            .corner_radius(12.0);

        let instances =
            create_primitive_instances(&style, Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0), 1.0);

        assert_eq!(
            instances.len(),
            1,
            "Single solid fill should create 1 instance"
        );
        assert_eq!(instances[0].pos, [10.0, 20.0]);
        assert_eq!(instances[0].size, [100.0, 50.0]);
        assert_eq!(instances[0].color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(instances[0].corner_radii, [12.0, 12.0, 12.0, 12.0]);
    }

    #[test]
    fn test_text_nodes_emit_background_fill_instances_from_secondary_paints() {
        let style = VisualStyle::new()
            .text(style_engine::TextContent::new("SCUM".to_string(), 72.0))
            .fill(Paint::solid(Vec4::new(0.0, 0.0, 0.0, 1.0)))
            .fill(Paint::solid(Vec4::new(1.0, 1.0, 1.0, 1.0)));

        let instances =
            create_primitive_instances(&style, Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0), 1.0);

        assert_eq!(
            instances.len(),
            1,
            "text-node secondary fills should render as background sticker surfaces"
        );
        assert_eq!(instances[0].color, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_text_only_drop_shadow_does_not_emit_rect_shadow_instance() {
        let style = VisualStyle::new()
            .text(style_engine::TextContent::new("TODAY".to_string(), 20.0))
            .fill(Paint::solid(Vec4::new(0.8, 1.0, 0.0, 1.0)))
            .drop_shadow(Vec2::new(4.0, 4.0), 0.0, Vec4::new(0.8, 1.0, 0.0, 1.0));

        let instances =
            create_primitive_instances(&style, Vec2::new(10.0, 20.0), Vec2::new(100.0, 40.0), 1.0);

        assert!(
            instances.is_empty(),
            "text-only styles should not emit rectangular drop-shadow instances"
        );
    }

    #[test]
    fn test_create_primitive_instances_with_opacity() {
        let style = VisualStyle::new().solid_fill(Vec4::new(1.0, 0.0, 0.0, 1.0));

        let instances = create_primitive_instances(&style, Vec2::ZERO, Vec2::ONE, 0.5);

        assert_eq!(instances.len(), 1);
        assert_eq!(
            instances[0].color[3], 0.5,
            "Opacity should be applied to alpha"
        );
    }

    #[test]
    fn test_create_primitive_instances_empty() {
        let style = VisualStyle::new(); // No fills

        let instances = create_primitive_instances(&style, Vec2::ZERO, Vec2::ONE, 1.0);

        assert_eq!(instances.len(), 0, "Empty style should create 0 instances");
    }
}
