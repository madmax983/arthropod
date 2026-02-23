//! CPU reference tests for drop shadow shader logic.
//!
//! These tests verify the SDF-based shadow blur by replicating the WGSL shader logic in Rust.
//! Shadows use a smoothstep falloff to approximate Gaussian blur for blur radii < 20px.
//! This is the "fast path" - larger blurs use multi-pass Gaussian blur (Phase 4).

// ============================================================================
// CPU Reference Implementation (mirrors WGSL shader)
// ============================================================================

/// Compute shadow alpha for a given SDF distance.
/// This mirrors the WGSL shadow_alpha() function.
///
/// # Parameters
/// - dist: Signed distance from the SDF boundary (negative = inside, positive = outside)
/// - blur_radius: Shadow blur radius in pixels
///
/// # Returns
/// Alpha value [0.0, 1.0] for the shadow at this distance
fn shadow_alpha(dist: f32, blur_radius: f32) -> f32 {
    // Approximate gaussian blur with smoothstep
    // Shadow is visible where dist < blur_radius
    // Smooth falloff from -blur_radius to +blur_radius
    1.0 - smoothstep(-blur_radius, blur_radius, dist)
}

/// Smoothstep function (mirrors GLSL/WGSL smoothstep)
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ============================================================================
// Basic Shadow Tests
// ============================================================================

#[test]
fn test_shadow_at_boundary() {
    let blur_radius = 8.0;

    // At the SDF boundary (dist = 0.0), shadow should be at 50% alpha
    let alpha = shadow_alpha(0.0, blur_radius);
    assert!(
        (alpha - 0.5).abs() < 0.01,
        "Shadow at boundary should be ~50%, got {}",
        alpha
    );
}

#[test]
fn test_shadow_inside_shape() {
    let blur_radius = 8.0;

    // Well inside the shape (dist < -blur_radius)
    let alpha = shadow_alpha(-10.0, blur_radius);
    assert!(
        alpha > 0.99,
        "Shadow well inside should be fully opaque, got {}",
        alpha
    );

    // Partially inside (dist = -half_blur_radius)
    let alpha = shadow_alpha(-4.0, blur_radius);
    assert!(
        alpha > 0.7,
        "Shadow at half blur radius inside should be strong, got {}",
        alpha
    );
}

#[test]
fn test_shadow_outside_shape() {
    let blur_radius = 8.0;

    // Well outside the shape (dist > blur_radius)
    let alpha = shadow_alpha(10.0, blur_radius);
    assert!(
        alpha < 0.01,
        "Shadow well outside should be nearly transparent, got {}",
        alpha
    );

    // Partially outside (dist = half_blur_radius)
    let alpha = shadow_alpha(4.0, blur_radius);
    assert!(
        alpha < 0.3,
        "Shadow at half blur radius outside should be faint, got {}",
        alpha
    );
}

#[test]
fn test_shadow_smooth_falloff() {
    let blur_radius = 8.0;

    // Shadow should have smooth gradient from inside to outside
    let mut prev_alpha: Option<f32> = None;

    for i in -20..20 {
        let dist = i as f32;
        let alpha = shadow_alpha(dist, blur_radius);

        // Alpha should decrease monotonically as dist increases
        if let Some(prev) = prev_alpha {
            assert!(
                alpha <= prev + 0.001, // Allow tiny floating point error
                "Shadow alpha should decrease monotonically (dist={}, alpha={}, prev={})",
                dist,
                alpha,
                prev
            );
        }

        prev_alpha = Some(alpha);
    }
}

// ============================================================================
// Blur Radius Tests
// ============================================================================

#[test]
fn test_small_blur_radius() {
    let blur_radius = 2.0;

    // Small blur should have sharp falloff
    // At boundary
    let alpha_boundary = shadow_alpha(0.0, blur_radius);
    assert!(
        (alpha_boundary - 0.5).abs() < 0.01,
        "Small blur at boundary, got {}",
        alpha_boundary
    );

    // Just outside blur extent
    let alpha_outside = shadow_alpha(3.0, blur_radius);
    assert!(
        alpha_outside < 0.05,
        "Small blur outside extent should be nearly invisible, got {}",
        alpha_outside
    );

    // Just inside blur extent
    let alpha_inside = shadow_alpha(-3.0, blur_radius);
    assert!(
        alpha_inside > 0.95,
        "Small blur inside extent should be nearly opaque, got {}",
        alpha_inside
    );
}

#[test]
fn test_large_blur_radius() {
    let blur_radius = 20.0;

    // Large blur should have gradual falloff
    // At boundary
    let alpha_boundary = shadow_alpha(0.0, blur_radius);
    assert!(
        (alpha_boundary - 0.5).abs() < 0.01,
        "Large blur at boundary, got {}",
        alpha_boundary
    );

    // Quarter way through blur
    let alpha_quarter = shadow_alpha(10.0, blur_radius);
    assert!(
        alpha_quarter > 0.1 && alpha_quarter < 0.4,
        "Large blur at quarter distance should have gentle falloff, got {}",
        alpha_quarter
    );

    // Far inside
    let alpha_far_inside = shadow_alpha(-30.0, blur_radius);
    assert!(
        alpha_far_inside > 0.99,
        "Large blur far inside should be opaque, got {}",
        alpha_far_inside
    );
}

#[test]
fn test_blur_radius_symmetry() {
    // Shadow should be symmetric around the boundary
    for blur_radius in [2.0, 8.0, 20.0] {
        for dist in [0.0, 2.0, 5.0, 10.0] {
            let alpha_pos = shadow_alpha(dist, blur_radius);
            let alpha_neg = shadow_alpha(-dist, blur_radius);

            // Sum should be approximately 1.0 due to symmetry
            let sum = alpha_pos + alpha_neg;
            assert!(
                (sum - 1.0).abs() < 0.01,
                "Shadow should be symmetric: dist={}, blur={}, sum={}",
                dist,
                blur_radius,
                sum
            );
        }
    }
}

// ============================================================================
// Shadow Extent Tests
// ============================================================================

#[test]
fn test_shadow_extent_matches_blur_radius() {
    let blur_radius = 10.0;

    // Shadow should be nearly opaque at -blur_radius
    let alpha_inner = shadow_alpha(-blur_radius, blur_radius);
    assert!(
        alpha_inner > 0.9,
        "Shadow at inner extent should be strong, got {}",
        alpha_inner
    );

    // Shadow should be nearly transparent at +blur_radius
    let alpha_outer = shadow_alpha(blur_radius, blur_radius);
    assert!(
        alpha_outer < 0.1,
        "Shadow at outer extent should be faint, got {}",
        alpha_outer
    );

    // Shadow should be ~50% at the boundary (dist=0)
    let alpha_boundary = shadow_alpha(0.0, blur_radius);
    assert!(
        (alpha_boundary - 0.5).abs() < 0.05,
        "Shadow at boundary should be mid-tone, got {}",
        alpha_boundary
    );
}

#[test]
fn test_shadow_beyond_blur_extent() {
    let blur_radius = 8.0;

    // Shadow should be fully opaque far inside
    for dist in [-20.0, -15.0, -10.0] {
        let alpha = shadow_alpha(dist, blur_radius);
        assert!(
            alpha > 0.99,
            "Shadow far inside (dist={}) should be opaque, got {}",
            dist,
            alpha
        );
    }

    // Shadow should be fully transparent far outside
    for dist in [10.0, 15.0, 20.0] {
        let alpha = shadow_alpha(dist, blur_radius);
        assert!(
            alpha < 0.01,
            "Shadow far outside (dist={}) should be transparent, got {}",
            dist,
            alpha
        );
    }
}

// ============================================================================
// Shadow Quality Tests
// ============================================================================

#[test]
fn test_shadow_smoothstep_quality() {
    let blur_radius = 10.0;

    // Smoothstep should produce smooth gradients (no sharp jumps)
    let mut prev_alpha: Option<f32> = None;

    for i in -30..30 {
        let dist = i as f32 / 2.0; // Sample every 0.5 pixels
        let alpha = shadow_alpha(dist, blur_radius);

        if let Some(prev) = prev_alpha {
            let delta = (alpha - prev).abs();
            assert!(
                delta < 0.1,
                "Shadow should change smoothly (dist={}, delta={})",
                dist,
                delta
            );
        }

        prev_alpha = Some(alpha);
    }
}

#[test]
fn test_shadow_gaussian_approximation() {
    let blur_radius = 10.0;

    // Smoothstep provides a reasonable approximation to Gaussian blur
    // for small blur radii (<20px), which is the design goal

    // At boundary (dist=0): smoothstep returns 0.5 (matches Gaussian at mean)
    let alpha_boundary = shadow_alpha(0.0, blur_radius);
    assert!(
        (alpha_boundary - 0.5).abs() < 0.01,
        "Shadow at boundary should be 0.5, got {}",
        alpha_boundary
    );

    // At dist=half_blur: smoothstep(−blur, +blur, +half_blur) = smoothstep(−10, 10, 5)
    // t = (5 − (−10)) / (10 − (−10)) = 15/20 = 0.75
    // smoothstep(0.75) = 0.75² × (3 − 2×0.75) = 0.5625 × 1.5 = 0.84375
    // alpha = 1 − 0.84375 = 0.15625
    let alpha_at_half = shadow_alpha(blur_radius / 2.0, blur_radius);
    assert!(
        (alpha_at_half - 0.15625).abs() < 0.01,
        "Shadow at half blur radius, got {}",
        alpha_at_half
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_zero_blur_radius() {
    let blur_radius = 0.0;

    // Zero blur = hard edge shadow (step function)
    let alpha_inside = shadow_alpha(-1.0, blur_radius);
    let alpha_outside = shadow_alpha(1.0, blur_radius);

    // With zero blur, smoothstep degenerates
    // smoothstep(0, 0, x) is undefined, but our impl clamps
    // Just verify no crash/NaN
    assert!(
        alpha_inside.is_finite(),
        "Zero blur inside should not produce NaN/inf"
    );
    assert!(
        alpha_outside.is_finite(),
        "Zero blur outside should not produce NaN/inf"
    );
}

#[test]
fn test_negative_blur_radius() {
    // Negative blur radius is nonsensical, but shouldn't crash
    let blur_radius = -5.0;

    let alpha = shadow_alpha(0.0, blur_radius);
    assert!(
        alpha.is_finite(),
        "Negative blur should not produce NaN/inf, got {}",
        alpha
    );
}

#[test]
fn test_very_large_blur_radius() {
    // Very large blur (should use multi-pass in practice, but test the function)
    let blur_radius = 100.0;

    // Should still produce valid alpha values
    let alpha_boundary = shadow_alpha(0.0, blur_radius);
    assert!(
        (alpha_boundary - 0.5).abs() < 0.01,
        "Large blur at boundary, got {}",
        alpha_boundary
    );

    let alpha_far = shadow_alpha(50.0, blur_radius);
    assert!(
        alpha_far > 0.1 && alpha_far < 0.5,
        "Large blur at half distance, got {}",
        alpha_far
    );
}

// ============================================================================
// Integration Tests (Shadow + SDF)
// ============================================================================

#[test]
fn test_shadow_with_sdf_offset() {
    // Typical usage: shadow offset by (2px, 4px), blur 6px
    let blur_radius = 6.0;
    let shadow_offset_dist = (2.0f32.powi(2) + 4.0f32.powi(2)).sqrt(); // ~4.47px

    // Shadow is rendered as a separate SDF, offset from the original
    // At the original shape boundary, the shadow SDF is at dist ≈ -4.47
    let alpha_at_shape_boundary = shadow_alpha(-shadow_offset_dist, blur_radius);

    // Shadow should be fairly visible at the shape boundary
    assert!(
        alpha_at_shape_boundary > 0.6,
        "Shadow with offset should be visible at original boundary, got {}",
        alpha_at_shape_boundary
    );
}

#[test]
fn test_shadow_spread() {
    let blur_radius = 8.0;
    let spread = 3.0;

    // Spread expands the shadow SDF by `spread` pixels
    // This is equivalent to rendering the shadow at dist - spread
    let dist = 5.0;
    let alpha_no_spread = shadow_alpha(dist, blur_radius);
    let alpha_with_spread = shadow_alpha(dist - spread, blur_radius);

    // With spread, shadow extends further out
    assert!(
        alpha_with_spread > alpha_no_spread,
        "Shadow with spread should extend further (no_spread={}, with_spread={})",
        alpha_no_spread,
        alpha_with_spread
    );
}
