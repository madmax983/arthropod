use render_engine::backend::wgpu::effects::{
    blend_darken, blend_difference, blend_exclusion, blend_lighten, blend_overlay, blend_screen,
    composite_blend_over,
};

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
