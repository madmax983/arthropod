use render_engine::backend::wgpu::effects::{
    blend_darken, blend_difference, blend_exclusion, blend_lighten, blend_overlay, blend_screen,
    composite_blend_over,
};

fn assert_rgb_close(actual: [f32; 4], expected: [f32; 3]) {
    assert!((actual[0] - expected[0]).abs() < 1e-6);
    assert!((actual[1] - expected[1]).abs() < 1e-6);
    assert!((actual[2] - expected[2]).abs() < 1e-6);
    assert!((actual[3] - 1.0).abs() < 1e-6);
}

fn color_burn_channel(src: f32, dst: f32) -> f32 {
    let eps = 1e-6;
    1.0 - ((1.0 - dst) / src.max(eps)).min(1.0)
}

fn color_dodge_channel(src: f32, dst: f32) -> f32 {
    let eps = 1e-6;
    (dst / (1.0 - src).max(eps)).min(1.0)
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
    let s = delta / (1.0 - (2.0 * l - 1.0).abs());
    let mut h = if (max - r).abs() <= 1e-6 {
        (g - b) / delta + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() <= 1e-6 {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    h /= 6.0;
    [h, s, l]
}

fn hue_distance(a: f32, b: f32) -> f32 {
    let d = (a - b).abs();
    d.min(1.0 - d)
}

#[test]
fn test_blend_mode_screen_matches_reference() {
    let src = [0.2, 0.5, 0.8];
    let dst = [0.9, 0.4, 0.25];
    let out = blend_screen(src, dst);
    assert!((out[0] - 0.92).abs() < 1e-6);
    assert!((out[1] - 0.7).abs() < 1e-6);
    assert!((out[2] - 0.85).abs() < 1e-6);
}

#[test]
fn test_blend_mode_overlay_matches_reference() {
    let src = [0.2, 0.5, 0.8];
    let dst = [0.9, 0.4, 0.25];
    let out = blend_overlay(src, dst);
    assert!((out[0] - 0.84).abs() < 1e-6);
    assert!((out[1] - 0.4).abs() < 1e-6);
    assert!((out[2] - 0.4).abs() < 1e-6);
}

#[test]
fn test_blend_mode_darken_matches_reference() {
    let src = [0.2, 0.5, 0.8];
    let dst = [0.9, 0.4, 0.25];
    assert_eq!(blend_darken(src, dst), [0.2, 0.4, 0.25]);
}

#[test]
fn test_blend_mode_lighten_matches_reference() {
    let src = [0.2, 0.5, 0.8];
    let dst = [0.9, 0.4, 0.25];
    assert_eq!(blend_lighten(src, dst), [0.9, 0.5, 0.8]);
}

#[test]
fn test_blend_mode_difference_matches_reference() {
    let src = [0.2, 0.5, 0.8];
    let dst = [0.9, 0.4, 0.25];
    let out = blend_difference(src, dst);
    assert!((out[0] - 0.7).abs() < 1e-6);
    assert!((out[1] - 0.1).abs() < 1e-6);
    assert!((out[2] - 0.55).abs() < 1e-6);
}

#[test]
fn test_blend_mode_exclusion_matches_reference() {
    let src = [0.2, 0.5, 0.8];
    let dst = [0.9, 0.4, 0.25];
    let out = blend_exclusion(src, dst);
    assert!((out[0] - 0.74).abs() < 1e-6);
    assert!((out[1] - 0.5).abs() < 1e-6);
    assert!((out[2] - 0.65).abs() < 1e-6);
}

#[test]
fn test_composite_blend_over_preserves_destination_when_source_alpha_zero() {
    let src = [0.0, 0.0, 0.0, 0.0];
    let dst = [0.22, 0.41, 0.64, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::Multiply, src, dst);
    assert!((out[0] - dst[0]).abs() < 1e-6);
    assert!((out[1] - dst[1]).abs() < 1e-6);
    assert!((out[2] - dst[2]).abs() < 1e-6);
    assert!((out[3] - dst[3]).abs() < 1e-6);
}

#[test]
fn test_composite_blend_over_uses_source_alpha_for_blend_mix() {
    let src = [0.95, 0.35, 0.30, 0.78];
    let dst = [0.23, 0.46, 0.94, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::Multiply, src, dst);

    // Expected with source-over alpha mix:
    // mix(dst, src*dst, src_a)
    let expected = [
        dst[0] * (1.0 - src[3]) + (src[0] * dst[0]) * src[3],
        dst[1] * (1.0 - src[3]) + (src[1] * dst[1]) * src[3],
        dst[2] * (1.0 - src[3]) + (src[2] * dst[2]) * src[3],
        1.0,
    ];

    assert!(
        (out[0] - expected[0]).abs() < 1e-6,
        "out[0]={}, expected[0]={}",
        out[0],
        expected[0]
    );
    assert!(
        (out[1] - expected[1]).abs() < 1e-6,
        "out[1]={}, expected[1]={}",
        out[1],
        expected[1]
    );
    assert!(
        (out[2] - expected[2]).abs() < 1e-6,
        "out[2]={}, expected[2]={}",
        out[2],
        expected[2]
    );
    assert!(
        (out[3] - expected[3]).abs() < 1e-6,
        "out[3]={}, expected[3]={}",
        out[3],
        expected[3]
    );
}

#[test]
fn test_composite_blend_over_color_burn_matches_reference() {
    let src = [0.65, 0.25, 0.90, 1.0];
    let dst = [0.40, 0.85, 0.30, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::ColorBurn, src, dst);

    let expected = [
        color_burn_channel(src[0], dst[0]),
        color_burn_channel(src[1], dst[1]),
        color_burn_channel(src[2], dst[2]),
    ];
    assert_rgb_close(out, expected);
}

#[test]
fn test_composite_blend_over_color_dodge_matches_reference() {
    let src = [0.35, 0.75, 0.20, 1.0];
    let dst = [0.60, 0.15, 0.80, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::ColorDodge, src, dst);

    let expected = [
        color_dodge_channel(src[0], dst[0]),
        color_dodge_channel(src[1], dst[1]),
        color_dodge_channel(src[2], dst[2]),
    ];
    assert_rgb_close(out, expected);
}

#[test]
fn test_composite_blend_over_linear_burn_matches_reference() {
    let src = [0.20, 0.80, 0.35, 1.0];
    let dst = [0.45, 0.25, 0.90, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::LinearBurn, src, dst);

    let expected = [
        (src[0] + dst[0] - 1.0).max(0.0),
        (src[1] + dst[1] - 1.0).max(0.0),
        (src[2] + dst[2] - 1.0).max(0.0),
    ];
    assert_rgb_close(out, expected);
}

#[test]
fn test_composite_blend_over_linear_dodge_matches_reference() {
    let src = [0.20, 0.80, 0.35, 1.0];
    let dst = [0.45, 0.25, 0.90, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::LinearDodge, src, dst);

    let expected = [
        (src[0] + dst[0]).min(1.0),
        (src[1] + dst[1]).min(1.0),
        (src[2] + dst[2]).min(1.0),
    ];
    assert_rgb_close(out, expected);
}

#[test]
fn test_composite_blend_over_soft_light_uses_screen_fallback() {
    let src = [0.30, 0.65, 0.40, 1.0];
    let dst = [0.70, 0.25, 0.85, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::SoftLight, src, dst);
    let expected = blend_screen([src[0], src[1], src[2]], [dst[0], dst[1], dst[2]]);
    assert_rgb_close(out, expected);
}

#[test]
fn test_composite_blend_over_hard_light_matches_overlay_swapped() {
    let src = [0.30, 0.65, 0.40, 1.0];
    let dst = [0.70, 0.25, 0.85, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::HardLight, src, dst);
    let expected = blend_overlay([dst[0], dst[1], dst[2]], [src[0], src[1], src[2]]);
    assert_rgb_close(out, expected);
}

#[test]
fn test_composite_blend_over_hue_preserves_dst_sat_lum_and_src_hue() {
    let src = [0.90, 0.20, 0.50, 1.0];
    let dst = [0.20, 0.70, 0.40, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::Hue, src, dst);

    let src_hsl = rgb_to_hsl([src[0], src[1], src[2]]);
    let dst_hsl = rgb_to_hsl([dst[0], dst[1], dst[2]]);
    let out_hsl = rgb_to_hsl([out[0], out[1], out[2]]);

    assert!(hue_distance(out_hsl[0], src_hsl[0]) < 1e-4);
    assert!((out_hsl[1] - dst_hsl[1]).abs() < 1e-4);
    assert!((out_hsl[2] - dst_hsl[2]).abs() < 1e-4);
}

#[test]
fn test_composite_blend_over_saturation_preserves_dst_hue_lum_and_src_sat() {
    let src = [0.90, 0.20, 0.50, 1.0];
    let dst = [0.20, 0.70, 0.40, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::Saturation, src, dst);

    let src_hsl = rgb_to_hsl([src[0], src[1], src[2]]);
    let dst_hsl = rgb_to_hsl([dst[0], dst[1], dst[2]]);
    let out_hsl = rgb_to_hsl([out[0], out[1], out[2]]);

    assert!(hue_distance(out_hsl[0], dst_hsl[0]) < 1e-4);
    assert!((out_hsl[1] - src_hsl[1]).abs() < 1e-4);
    assert!((out_hsl[2] - dst_hsl[2]).abs() < 1e-4);
}

#[test]
fn test_composite_blend_over_color_preserves_src_hue_sat_and_dst_lum() {
    let src = [0.90, 0.20, 0.50, 1.0];
    let dst = [0.20, 0.70, 0.40, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::Color, src, dst);

    let src_hsl = rgb_to_hsl([src[0], src[1], src[2]]);
    let dst_hsl = rgb_to_hsl([dst[0], dst[1], dst[2]]);
    let out_hsl = rgb_to_hsl([out[0], out[1], out[2]]);

    assert!(hue_distance(out_hsl[0], src_hsl[0]) < 1e-4);
    assert!((out_hsl[1] - src_hsl[1]).abs() < 1e-4);
    assert!((out_hsl[2] - dst_hsl[2]).abs() < 1e-4);
}

#[test]
fn test_composite_blend_over_luminosity_preserves_dst_hue_sat_and_src_lum() {
    let src = [0.90, 0.20, 0.50, 1.0];
    let dst = [0.20, 0.70, 0.40, 1.0];
    let out = composite_blend_over(style_engine::BlendMode::Luminosity, src, dst);

    let src_hsl = rgb_to_hsl([src[0], src[1], src[2]]);
    let dst_hsl = rgb_to_hsl([dst[0], dst[1], dst[2]]);
    let out_hsl = rgb_to_hsl([out[0], out[1], out[2]]);

    assert!(hue_distance(out_hsl[0], dst_hsl[0]) < 1e-4);
    assert!((out_hsl[1] - dst_hsl[1]).abs() < 1e-4);
    assert!((out_hsl[2] - src_hsl[2]).abs() < 1e-4);
}
