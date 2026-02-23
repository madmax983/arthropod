//! Phase 4 multi-pass effects infrastructure.
//!
//! This module holds render-target pooling, effect pass classification,
//! and CPU-side helpers shared by blur/blend pipelines.

use crate::{NodeContent, NodeId, SceneNode};
use style_engine::{BlendMode, Effect, VisualStyle};

pub use super::render_target_pool::{RenderTargetHandle, RenderTargetKey, RenderTargetPool};

/// Render pass kinds used by the Phase 4 effect planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectPassKind {
    /// Render directly with the primitive/path pipelines.
    DirectPrimitive,
    /// Render the node (and subtree) into an offscreen layer.
    OffscreenLayer,
    /// Capture already-rendered backdrop content for background blur.
    BackgroundCapture,
    /// Horizontal blur pass.
    BlurHorizontal,
    /// Vertical blur pass.
    BlurVertical,
    /// Inner shadow pass.
    InnerShadow,
    /// Composite with non-normal blend mode.
    BlendComposite,
    /// Push clip mask into stencil.
    StencilPush,
    /// Pop clip mask from stencil stack.
    StencilPop,
}

/// Planned effect pass entry for a node.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectPass {
    pub node_id: NodeId,
    pub kind: EffectPassKind,
    pub target: Option<RenderTargetHandle>,
    /// Pixel bounds: [x, y, width, height]
    pub bounds_px: [u32; 4],
    pub blend_mode: BlendMode,
}

/// Planner input extracted from a scene node.
#[derive(Debug, Clone)]
pub struct EffectPlanNode {
    pub node_id: NodeId,
    pub style: Option<VisualStyle>,
    pub bounds: plat_core::Rect,
    pub has_children: bool,
    pub visible: bool,
    pub opacity: f32,
}

impl EffectPlanNode {
    #[must_use]
    pub fn from_scene(node_id: NodeId, node: &SceneNode) -> Self {
        let style = match &node.content {
            NodeContent::Styled { style } => Some((**style).clone()),
            NodeContent::Empty | NodeContent::SolidColor { .. } => None,
        };
        Self {
            node_id,
            style,
            bounds: node.bounds,
            has_children: !node.children.is_empty(),
            visible: node.visible,
            opacity: node.opacity,
        }
    }
}

/// Generate a normalized 1D Gaussian kernel.
///
/// `radius` is in pixels; `tap_count` is clamped to odd values in `[1, 255]`.
#[must_use]
pub fn gaussian_kernel_1d(radius: f32, tap_count: usize) -> Vec<f32> {
    let clamped_taps = tap_count.clamp(1, 255);
    let taps = if clamped_taps.is_multiple_of(2) {
        clamped_taps + 1
    } else {
        clamped_taps
    };
    let sigma = (radius.max(0.001)) / 3.0;
    let center = (taps / 2) as i32;

    let mut weights = Vec::with_capacity(taps);
    let mut total = 0.0f32;

    for i in 0..taps {
        let x = (i as i32 - center) as f32;
        let weight = (-x * x / (2.0 * sigma * sigma)).exp();
        weights.push(weight);
        total += weight;
    }

    if total > 0.0 {
        for w in &mut weights {
            *w /= total;
        }
    }

    weights
}

/// CPU reference multiply blend.
#[must_use]
pub fn blend_multiply(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    [src[0] * dst[0], src[1] * dst[1], src[2] * dst[2]]
}

/// CPU reference screen blend.
#[must_use]
pub fn blend_screen(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    [
        src[0] + dst[0] - src[0] * dst[0],
        src[1] + dst[1] - src[1] * dst[1],
        src[2] + dst[2] - src[2] * dst[2],
    ]
}

/// CPU reference overlay blend.
#[must_use]
pub fn blend_overlay(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    let chan = |s: f32, d: f32| {
        if d <= 0.5 {
            2.0 * s * d
        } else {
            1.0 - 2.0 * (1.0 - s) * (1.0 - d)
        }
    };
    [
        chan(src[0], dst[0]),
        chan(src[1], dst[1]),
        chan(src[2], dst[2]),
    ]
}

/// CPU reference darken blend.
#[must_use]
pub fn blend_darken(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    [src[0].min(dst[0]), src[1].min(dst[1]), src[2].min(dst[2])]
}

/// CPU reference lighten blend.
#[must_use]
pub fn blend_lighten(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    [src[0].max(dst[0]), src[1].max(dst[1]), src[2].max(dst[2])]
}

/// CPU reference difference blend.
#[must_use]
pub fn blend_difference(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    [
        (src[0] - dst[0]).abs(),
        (src[1] - dst[1]).abs(),
        (src[2] - dst[2]).abs(),
    ]
}

/// CPU reference exclusion blend.
#[must_use]
pub fn blend_exclusion(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    [
        src[0] + dst[0] - 2.0 * src[0] * dst[0],
        src[1] + dst[1] - 2.0 * src[1] * dst[1],
        src[2] + dst[2] - 2.0 * src[2] * dst[2],
    ]
}

/// CPU reference color burn blend.
#[must_use]
pub fn blend_color_burn(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    let channel = |s: f32, d: f32| {
        let eps = 1e-6;
        1.0 - ((1.0 - d) / s.max(eps)).min(1.0)
    };
    [
        channel(src[0], dst[0]),
        channel(src[1], dst[1]),
        channel(src[2], dst[2]),
    ]
}

/// CPU reference color dodge blend.
#[must_use]
pub fn blend_color_dodge(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    let channel = |s: f32, d: f32| {
        let eps = 1e-6;
        (d / (1.0 - s).max(eps)).min(1.0)
    };
    [
        channel(src[0], dst[0]),
        channel(src[1], dst[1]),
        channel(src[2], dst[2]),
    ]
}

/// CPU reference linear burn blend.
#[must_use]
pub fn blend_linear_burn(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    [
        (src[0] + dst[0] - 1.0).max(0.0),
        (src[1] + dst[1] - 1.0).max(0.0),
        (src[2] + dst[2] - 1.0).max(0.0),
    ]
}

/// CPU reference linear dodge blend.
#[must_use]
pub fn blend_linear_dodge(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    [
        (src[0] + dst[0]).min(1.0),
        (src[1] + dst[1]).min(1.0),
        (src[2] + dst[2]).min(1.0),
    ]
}

/// CPU reference soft light blend.
///
/// Current parity target matches the shader fallback branch.
#[must_use]
pub fn blend_soft_light(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    blend_screen(src, dst)
}

/// CPU reference hard light blend.
#[must_use]
pub fn blend_hard_light(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    blend_overlay(dst, src)
}

fn rgb_to_hsl(rgb: [f32; 3]) -> [f32; 3] {
    let r = rgb[0];
    let g = rgb[1];
    let b = rgb[2];
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    let delta = max - min;
    if delta <= 1e-6 {
        return [0.0, 0.0, l];
    }
    let s = {
        let denom = 1.0 - (2.0 * l - 1.0).abs();
        if denom <= 1e-6 { 0.0 } else { delta / denom }
    };
    let mut h = if (max - r).abs() <= 1e-6 {
        (g - b) / delta + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() <= 1e-6 {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    h /= 6.0;
    if h < 0.0 {
        h += 1.0;
    }
    [h, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0)]
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

fn hsl_to_rgb(hsl: [f32; 3]) -> [f32; 3] {
    let h = hsl[0];
    let s = hsl[1];
    let l = hsl[2];
    if s <= 1e-6 {
        return [l, l, l];
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    [
        hue_to_rgb(p, q, h + 1.0 / 3.0).clamp(0.0, 1.0),
        hue_to_rgb(p, q, h).clamp(0.0, 1.0),
        hue_to_rgb(p, q, h - 1.0 / 3.0).clamp(0.0, 1.0),
    ]
}

/// CPU reference hue blend (non-separable).
#[must_use]
pub fn blend_hue(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    let src_hsl = rgb_to_hsl(src);
    let dst_hsl = rgb_to_hsl(dst);
    hsl_to_rgb([src_hsl[0], dst_hsl[1], dst_hsl[2]])
}

/// CPU reference saturation blend (non-separable).
#[must_use]
pub fn blend_saturation(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    let src_hsl = rgb_to_hsl(src);
    let dst_hsl = rgb_to_hsl(dst);
    hsl_to_rgb([dst_hsl[0], src_hsl[1], dst_hsl[2]])
}

/// CPU reference color blend (non-separable).
#[must_use]
pub fn blend_color(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    let src_hsl = rgb_to_hsl(src);
    let dst_hsl = rgb_to_hsl(dst);
    hsl_to_rgb([src_hsl[0], src_hsl[1], dst_hsl[2]])
}

/// CPU reference luminosity blend (non-separable).
#[must_use]
pub fn blend_luminosity(src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    let src_hsl = rgb_to_hsl(src);
    let dst_hsl = rgb_to_hsl(dst);
    hsl_to_rgb([dst_hsl[0], dst_hsl[1], src_hsl[2]])
}

fn apply_blend_mode(mode: BlendMode, src: [f32; 3], dst: [f32; 3]) -> [f32; 3] {
    match mode {
        BlendMode::Darken => blend_darken(src, dst),
        BlendMode::Multiply => blend_multiply(src, dst),
        BlendMode::ColorBurn => blend_color_burn(src, dst),
        BlendMode::Lighten => blend_lighten(src, dst),
        BlendMode::Screen => blend_screen(src, dst),
        BlendMode::ColorDodge => blend_color_dodge(src, dst),
        BlendMode::Overlay => blend_overlay(src, dst),
        BlendMode::SoftLight => blend_soft_light(src, dst),
        BlendMode::HardLight => blend_hard_light(src, dst),
        BlendMode::Difference => blend_difference(src, dst),
        BlendMode::Exclusion => blend_exclusion(src, dst),
        BlendMode::Hue => blend_hue(src, dst),
        BlendMode::Saturation => blend_saturation(src, dst),
        BlendMode::Color => blend_color(src, dst),
        BlendMode::Luminosity => blend_luminosity(src, dst),
        BlendMode::LinearBurn => blend_linear_burn(src, dst),
        BlendMode::LinearDodge => blend_linear_dodge(src, dst),
        BlendMode::Normal | BlendMode::PassThrough => src,
    }
}

/// Reference compositing for blend-mode source-over.
///
/// Inputs are straight-alpha RGBA in `[0, 1]`.
/// Output is premultiplied RGBA in `[0, 1]`, matching render-target storage.
#[must_use]
pub fn composite_blend_over(mode: BlendMode, src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let src_alpha = src[3].clamp(0.0, 1.0);
    let dst_alpha = dst[3].clamp(0.0, 1.0);
    let src_rgb = [
        src[0].clamp(0.0, 1.0),
        src[1].clamp(0.0, 1.0),
        src[2].clamp(0.0, 1.0),
    ];
    let dst_rgb = [
        dst[0].clamp(0.0, 1.0),
        dst[1].clamp(0.0, 1.0),
        dst[2].clamp(0.0, 1.0),
    ];

    let blended = apply_blend_mode(mode, src_rgb, dst_rgb);
    let out_alpha = src_alpha + dst_alpha - src_alpha * dst_alpha;

    // Porter-Duff source-over with blend-mode color function B(Cb, Cs):
    // Co = as*(1-ab)*Cs + as*ab*B(Cb,Cs) + (1-as)*ab*Cb
    let out_rgb = [
        (src_alpha * (1.0 - dst_alpha) * src_rgb[0])
            + (src_alpha * dst_alpha * blended[0])
            + ((1.0 - src_alpha) * dst_alpha * dst_rgb[0]),
        (src_alpha * (1.0 - dst_alpha) * src_rgb[1])
            + (src_alpha * dst_alpha * blended[1])
            + ((1.0 - src_alpha) * dst_alpha * dst_rgb[1]),
        (src_alpha * (1.0 - dst_alpha) * src_rgb[2])
            + (src_alpha * dst_alpha * blended[2])
            + ((1.0 - src_alpha) * dst_alpha * dst_rgb[2]),
    ];

    [out_rgb[0], out_rgb[1], out_rgb[2], out_alpha]
}

/// Classify a style into the effect pass kinds required for rendering.
#[must_use]
pub fn classify_effect_passes(style: &VisualStyle, has_children: bool) -> Vec<EffectPassKind> {
    let mut passes = Vec::new();
    let mut needs_blur = false;
    let mut needs_offscreen = false;

    for effect in &style.effects {
        match effect {
            Effect::LayerBlur(blur) if blur.visible && blur.radius > 0.0 => {
                needs_offscreen = true;
                needs_blur = true;
                passes.push(EffectPassKind::OffscreenLayer);
            }
            Effect::BackgroundBlur(blur) if blur.visible && blur.radius > 0.0 => {
                needs_blur = true;
                passes.push(EffectPassKind::BackgroundCapture);
            }
            Effect::InnerShadow(shadow) if shadow.visible => {
                needs_offscreen = true;
                passes.push(EffectPassKind::InnerShadow);
            }
            _ => {}
        }
    }

    if style.clips_content && has_children {
        passes.push(EffectPassKind::StencilPush);
        passes.push(EffectPassKind::StencilPop);
    }

    if needs_blur {
        passes.push(EffectPassKind::BlurHorizontal);
        passes.push(EffectPassKind::BlurVertical);
    }

    if !matches!(style.blend_mode, BlendMode::Normal | BlendMode::PassThrough) {
        needs_offscreen = true;
        passes.push(EffectPassKind::BlendComposite);
    }

    if !needs_offscreen && passes.is_empty() {
        passes.push(EffectPassKind::DirectPrimitive);
    }

    passes
}

fn clamp_bounds_to_frame(bounds: plat_core::Rect, frame_width: u32, frame_height: u32) -> [u32; 4] {
    let frame_w = frame_width as f32;
    let frame_h = frame_height as f32;

    let x0 = bounds.x.max(0.0).min(frame_w).floor();
    let y0 = bounds.y.max(0.0).min(frame_h).floor();
    let x1 = (bounds.x + bounds.width).max(0.0).min(frame_w).ceil();
    let y1 = (bounds.y + bounds.height).max(0.0).min(frame_h).ceil();

    let width = (x1 - x0).max(0.0) as u32;
    let height = (y1 - y0).max(0.0) as u32;

    [x0 as u32, y0 as u32, width, height]
}

/// Build a deterministic pass sequence for visible styled nodes.
///
/// The returned list is in frame execution order.
#[must_use]
pub fn plan_effect_passes(
    nodes: &[EffectPlanNode],
    frame_width: u32,
    frame_height: u32,
) -> Vec<EffectPass> {
    let mut planned = Vec::new();

    for node in nodes {
        if !node.visible || node.opacity <= 0.0 {
            continue;
        }
        let Some(style) = &node.style else {
            continue;
        };

        let mut kinds = classify_effect_passes(style, node.has_children);
        let requires_composite = kinds.iter().any(|kind| {
            matches!(
                kind,
                EffectPassKind::OffscreenLayer
                    | EffectPassKind::BackgroundCapture
                    | EffectPassKind::InnerShadow
            )
        });

        if requires_composite && !kinds.contains(&EffectPassKind::BlendComposite) {
            kinds.push(EffectPassKind::BlendComposite);
        }

        let bounds_px = clamp_bounds_to_frame(node.bounds, frame_width, frame_height);
        planned.extend(kinds.into_iter().map(|kind| EffectPass {
            node_id: node.node_id,
            kind,
            target: None,
            bounds_px,
            blend_mode: style.blend_mode,
        }));
    }

    planned
}

/// Compute backdrop capture bounds for background blur.
///
/// `node_bounds` and `frame_bounds` are `[x, y, width, height]` in pixels.
/// Bounds are inflated by `ceil(radius * 2.0)` and clamped to frame.
#[must_use]
pub fn backdrop_capture_bounds(
    node_bounds: [u32; 4],
    radius: f32,
    frame_bounds: [u32; 4],
) -> [u32; 4] {
    let inflate = (radius.max(0.0) * 2.0).ceil() as i32;

    let nx0 = node_bounds[0] as i32 - inflate;
    let ny0 = node_bounds[1] as i32 - inflate;
    let nx1 = node_bounds[0] as i32 + node_bounds[2] as i32 + inflate;
    let ny1 = node_bounds[1] as i32 + node_bounds[3] as i32 + inflate;

    let fx0 = frame_bounds[0] as i32;
    let fy0 = frame_bounds[1] as i32;
    let fx1 = frame_bounds[0] as i32 + frame_bounds[2] as i32;
    let fy1 = frame_bounds[1] as i32 + frame_bounds[3] as i32;

    let x0 = nx0.max(fx0);
    let y0 = ny0.max(fy0);
    let x1 = nx1.min(fx1);
    let y1 = ny1.min(fy1);

    let width = (x1 - x0).max(0) as u32;
    let height = (y1 - y0).max(0) as u32;

    [x0 as u32, y0 as u32, width, height]
}

/// Compute inner shadow alpha from original shape mask and blurred-offset mask.
#[must_use]
pub fn inner_shadow_alpha(mask: f32, blurred_offset_mask: f32) -> f32 {
    (blurred_offset_mask - (1.0 - mask)).clamp(0.0, 1.0) * mask
}

pub fn style_requires_multipass(style: &style_engine::VisualStyle) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use style_engine::{BackgroundBlur, LayerBlur};

    #[test]
    fn test_kernel_is_odd_length() {
        let k = gaussian_kernel_1d(8.0, 24);
        assert_eq!(k.len() % 2, 1);
    }

    #[test]
    fn test_classify_direct_primitive_when_no_effects() {
        let style = VisualStyle::new();
        let passes = classify_effect_passes(&style, false);
        assert_eq!(passes, vec![EffectPassKind::DirectPrimitive]);
    }

    #[test]
    fn test_classify_blur_effects() {
        let style = VisualStyle::new()
            .effect(Effect::LayerBlur(LayerBlur {
                radius: 10.0,
                visible: true,
            }))
            .effect(Effect::BackgroundBlur(BackgroundBlur {
                radius: 8.0,
                visible: true,
            }));
        let passes = classify_effect_passes(&style, false);
        assert!(passes.contains(&EffectPassKind::OffscreenLayer));
        assert!(passes.contains(&EffectPassKind::BackgroundCapture));
        assert!(passes.contains(&EffectPassKind::BlurHorizontal));
        assert!(passes.contains(&EffectPassKind::BlurVertical));
    }

    #[test]
    fn test_backdrop_capture_bounds_inflate_and_clamp() {
        let b = backdrop_capture_bounds([10, 10, 100, 100], 12.0, [0, 0, 120, 120]);
        assert_eq!(b, [0, 0, 120, 120]);
    }

    #[test]
    fn test_inner_shadow_alpha_is_masked() {
        assert_eq!(inner_shadow_alpha(0.0, 1.0), 0.0);
        assert!(inner_shadow_alpha(1.0, 0.8) > 0.0);
    }
}
