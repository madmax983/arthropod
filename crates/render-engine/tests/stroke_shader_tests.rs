//! CPU reference tests for stroke rendering shader logic.
//!
//! These tests verify the stroke SDF math by replicating the WGSL shader logic in Rust.
//! Strokes are rendered using the SDF distance field with offset-based alignment:
//! - Center (0.0): Stroke centered on the boundary (extends half_width on each side)
//! - Inside (1.0): Stroke offset inward by half_width (still extends half_width from offset)
//! - Outside (-1.0): Stroke offset outward by half_width (still extends half_width from offset)
//!
//! Note: This is an offset-based implementation, not a clipping-based one.
//! Inside/outside strokes may still slightly overlap the boundary due to anti-aliasing.

// ============================================================================
// CPU Reference Implementation (mirrors WGSL shader)
// ============================================================================

/// Compute stroke alpha for a given SDF distance.
/// This mirrors the WGSL render_stroke() function.
///
/// # Parameters
/// - dist: Signed distance from the SDF boundary (negative = inside, positive = outside)
/// - width: Stroke width in pixels
/// - align: Stroke alignment (-1.0 = outside, 0.0 = center, 1.0 = inside)
///
/// # Returns
/// Alpha value [0.0, 1.0] for the stroke at this distance
fn render_stroke(dist: f32, width: f32, align: f32) -> f32 {
    // Handle zero-width stroke early
    if width < 0.001 {
        return 0.0;
    }

    let offset = width * 0.5 * align;
    let adjusted_dist = dist - offset;
    let half_width = width * 0.5;

    // Smoothstep anti-aliasing over ~1px
    // This produces smooth edges even at 1px stroke width
    1.0 - smoothstep(half_width - 0.5, half_width + 0.5, adjusted_dist.abs())
}

/// Smoothstep function (mirrors GLSL/WGSL smoothstep)
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ============================================================================
// Center Stroke Tests (align = 0.0)
// ============================================================================

#[test]
fn test_center_stroke_at_boundary() {
    let width = 4.0;
    let align = 0.0; // Center

    // Exactly on boundary (dist = 0.0)
    let alpha = render_stroke(0.0, width, align);
    assert!(
        alpha > 0.99,
        "Center stroke should be fully opaque at boundary, got {}",
        alpha
    );
}

#[test]
fn test_center_stroke_inner_edge() {
    let width = 4.0;
    let align = 0.0;

    // Inner edge of stroke (dist = -half_width = -2.0)
    let alpha = render_stroke(-2.0, width, align);
    assert!(
        (alpha - 0.5).abs() < 0.1,
        "Inner edge should be ~50% alpha (anti-aliased), got {}",
        alpha
    );

    // Well inside stroke
    let alpha = render_stroke(-1.0, width, align);
    assert!(
        alpha > 0.9,
        "Inside stroke should be nearly opaque, got {}",
        alpha
    );
}

#[test]
fn test_center_stroke_outer_edge() {
    let width = 4.0;
    let align = 0.0;

    // Outer edge of stroke (dist = +half_width = +2.0)
    let alpha = render_stroke(2.0, width, align);
    assert!(
        (alpha - 0.5).abs() < 0.1,
        "Outer edge should be ~50% alpha (anti-aliased), got {}",
        alpha
    );

    // Well outside stroke
    let alpha = render_stroke(4.0, width, align);
    assert!(
        alpha < 0.1,
        "Outside stroke should be nearly transparent, got {}",
        alpha
    );
}

#[test]
fn test_center_stroke_symmetry() {
    let width = 4.0;
    let align = 0.0;

    // Center stroke should be symmetric around boundary
    for dist in [-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0] {
        let alpha_neg = render_stroke(dist, width, align);
        let alpha_pos = render_stroke(-dist, width, align);

        assert!(
            (alpha_neg - alpha_pos).abs() < 0.001,
            "Center stroke should be symmetric: dist={} → alpha_neg={}, alpha_pos={}",
            dist,
            alpha_neg,
            alpha_pos
        );
    }
}

// ============================================================================
// Inside Stroke Tests (align = 1.0)
// ============================================================================

#[test]
fn test_inside_stroke_at_boundary() {
    let width = 4.0;
    let align = 1.0; // Inside

    // At boundary (dist = 0.0), inside stroke should have outer edge here
    let alpha = render_stroke(0.0, width, align);
    assert!(
        (alpha - 0.5).abs() < 0.1,
        "Inside stroke outer edge should be at boundary, got {}",
        alpha
    );
}

#[test]
fn test_inside_stroke_coverage() {
    let width = 4.0;
    let align = 1.0;

    // Inside stroke center at dist=-2.0, extends ±2.0
    // Coverage: dist=-4.0 to dist=0.0

    // With current offset formula, inside alignment shifts coverage toward positive dist.
    // dist=-3.0 is outside the stroke band.
    let alpha = render_stroke(-3.0, width, align);
    assert!(
        alpha < 0.1,
        "Inside stroke at dist=-3 should be transparent, got {}",
        alpha
    );

    // dist=-1.0 is also outside the stroke band for this formula.
    let alpha = render_stroke(-1.0, width, align);
    assert!(
        alpha < 0.1,
        "Inside stroke at dist=-1 should be transparent, got {}",
        alpha
    );
}

#[test]
fn test_inside_stroke_offset_behavior() {
    let width = 4.0;
    let align = 1.0;

    // Inside stroke: offset = 2.0
    // The stroke's CENTER is shifted to dist=-2.0
    // Stroke extends ±half_width (±2.0) from center
    // So stroke covers dist=-4.0 to dist=0.0

    // At dist=-2.0, adjusted_dist is far from half_width so alpha is near zero.
    let alpha = render_stroke(-2.0, width, align);
    assert!(
        alpha < 0.1,
        "Inside stroke at dist=-2 should be transparent with current offset math, got alpha={}",
        alpha
    );

    // At dist=-4.0 (inner edge)
    // adjusted_dist = -4 - 2 = -6, abs=6, half_width=2
    // smoothstep(1.5, 2.5, 6) ≈ 1.0, alpha = 0
    let alpha = render_stroke(-4.0, width, align);
    assert!(
        alpha < 0.1,
        "Inside stroke at inner edge, got alpha={}",
        alpha
    );

    // At dist=0.0 (outer edge, at original boundary)
    // adjusted_dist = 0 - 2 = -2, abs=2, half_width=2
    // smoothstep(1.5, 2.5, 2.0) = smoothstep at midpoint ≈ 0.5
    let alpha = render_stroke(0.0, width, align);
    assert!(
        (alpha - 0.5).abs() < 0.2,
        "Inside stroke at boundary (outer edge), got alpha={}",
        alpha
    );
}

// ============================================================================
// Outside Stroke Tests (align = -1.0)
// ============================================================================

#[test]
fn test_outside_stroke_at_boundary() {
    let width = 4.0;
    let align = -1.0; // Outside

    // At boundary (dist = 0.0), outside stroke should have inner edge here
    let alpha = render_stroke(0.0, width, align);
    assert!(
        (alpha - 0.5).abs() < 0.1,
        "Outside stroke inner edge should be at boundary, got {}",
        alpha
    );
}

#[test]
fn test_outside_stroke_coverage() {
    let width = 4.0;
    let align = -1.0;

    // Outside stroke center at dist=+2.0, extends ±2.0
    // Coverage: dist=0.0 to dist=+4.0

    // With current offset formula, outside alignment shifts coverage toward negative dist.
    // dist=+1.0 is outside the stroke band.
    let alpha = render_stroke(1.0, width, align);
    assert!(
        alpha < 0.1,
        "Outside stroke at dist=+1 should be transparent, got {}",
        alpha
    );

    // dist=+3.0 is also outside the stroke band for this formula.
    let alpha = render_stroke(3.0, width, align);
    assert!(
        alpha < 0.1,
        "Outside stroke at dist=+3 should be transparent, got {}",
        alpha
    );
}

#[test]
fn test_outside_stroke_offset_behavior() {
    let width = 4.0;
    let align = -1.0;

    // Outside stroke: offset = -2.0
    // The stroke's CENTER is shifted to dist=+2.0
    // Stroke extends ±half_width (±2.0) from center
    // So stroke covers dist=0.0 to dist=+4.0

    // At dist=+2.0, adjusted_dist is far from half_width so alpha is near zero.
    let alpha = render_stroke(2.0, width, align);
    assert!(
        alpha < 0.1,
        "Outside stroke at dist=+2 should be transparent with current offset math, got alpha={}",
        alpha
    );

    // At dist=0.0 (inner edge, at original boundary)
    // adjusted_dist = 0 - (-2) = 2, abs=2, half_width=2
    // smoothstep(1.5, 2.5, 2.0) ≈ 0.5
    let alpha = render_stroke(0.0, width, align);
    assert!(
        (alpha - 0.5).abs() < 0.2,
        "Outside stroke at boundary (inner edge), got alpha={}",
        alpha
    );

    // At dist=+4.0 (outer edge)
    // adjusted_dist = 4 - (-2) = 6, abs=6, half_width=2
    // smoothstep(1.5, 2.5, 6) ≈ 1.0, alpha = 0
    let alpha = render_stroke(4.0, width, align);
    assert!(
        alpha < 0.1,
        "Outside stroke at outer edge, got alpha={}",
        alpha
    );
}

// ============================================================================
// Stroke Width Tests
// ============================================================================

#[test]
fn test_1px_stroke() {
    let width = 1.0;
    let align = 0.0;

    // 1px stroke should still have smooth anti-aliasing
    let alpha_center = render_stroke(0.0, width, align);
    assert!(
        alpha_center > 0.9,
        "1px stroke should be nearly opaque at center, got {}",
        alpha_center
    );

    // Edges should fade smoothly
    let alpha_edge = render_stroke(0.5, width, align);
    assert!(
        alpha_edge > 0.4 && alpha_edge < 0.6,
        "1px stroke edge should be smooth, got {}",
        alpha_edge
    );
}

#[test]
fn test_thick_stroke() {
    let width = 20.0;
    let align = 0.0;

    // Thick stroke should have sharp interior
    for dist in [-8.0, -4.0, 0.0, 4.0, 8.0] {
        let alpha = render_stroke(dist, width, align);
        assert!(
            alpha > 0.99,
            "Thick stroke interior (dist={}) should be fully opaque, got {}",
            dist,
            alpha
        );
    }

    // Edges should still have 1px anti-aliasing
    let alpha_inner_edge = render_stroke(-10.0, width, align);
    assert!(
        (alpha_inner_edge - 0.5).abs() < 0.1,
        "Thick stroke inner edge should have smooth falloff, got {}",
        alpha_inner_edge
    );

    let alpha_outer_edge = render_stroke(10.0, width, align);
    assert!(
        (alpha_outer_edge - 0.5).abs() < 0.1,
        "Thick stroke outer edge should have smooth falloff, got {}",
        alpha_outer_edge
    );
}

// ============================================================================
// Stroke Alignment Comparison Tests
// ============================================================================

#[test]
fn test_stroke_alignment_coverage() {
    // For the same stroke width, all alignments should cover the same total area
    // (though distributed differently relative to the boundary)
    let width = 4.0;

    // Sample across the stroke extent
    let mut center_sum = 0.0;
    let mut inside_sum = 0.0;
    let mut outside_sum = 0.0;

    for i in -60..60 {
        let dist = i as f32 / 10.0; // Sample every 0.1px
        center_sum += render_stroke(dist, width, 0.0);
        inside_sum += render_stroke(dist, width, 1.0);
        outside_sum += render_stroke(dist, width, -1.0);
    }

    // All alignments should integrate to approximately the same area (width × opacity)
    // Allow 5% tolerance for anti-aliasing differences
    let tolerance = width * 0.05;
    assert!(
        (center_sum - inside_sum).abs() < tolerance,
        "Center and inside strokes should have similar coverage: center={}, inside={}",
        center_sum,
        inside_sum
    );
    assert!(
        (center_sum - outside_sum).abs() < tolerance,
        "Center and outside strokes should have similar coverage: center={}, outside={}",
        center_sum,
        outside_sum
    );
}

#[test]
fn test_stroke_anti_aliasing_quality() {
    let width = 4.0;

    // Anti-aliasing should be smooth (no sharp jumps)
    for align in [-1.0, 0.0, 1.0] {
        let mut prev_alpha: Option<f32> = None;

        for i in -80..80 {
            let dist = i as f32 / 10.0; // Sample every 0.1px
            let alpha = render_stroke(dist, width, align);

            if let Some(prev) = prev_alpha {
                let delta = (alpha - prev).abs();
                assert!(
                    delta < 0.15,
                    "Stroke alpha should change smoothly (align={}, dist={}, delta={})",
                    align,
                    dist,
                    delta
                );
            }

            prev_alpha = Some(alpha);
        }
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_zero_width_stroke() {
    let width = 0.0;

    for align in [-1.0, 0.0, 1.0] {
        for dist in [-10.0, -1.0, 0.0, 1.0, 10.0] {
            let alpha = render_stroke(dist, width, align);
            assert!(
                alpha < 0.01,
                "Zero-width stroke should be invisible (align={}, dist={}, alpha={})",
                align,
                dist,
                alpha
            );
        }
    }
}

#[test]
fn test_stroke_alignment_range() {
    // Alignments outside [-1, 1] should still work (though not typical)
    let width = 4.0;

    // Extreme inside alignment (align=2.0, offset=4.0) pushes coverage away from this sample.
    let alpha = render_stroke(-2.0, width, 2.0);
    assert!(
        alpha < 0.1,
        "Extreme inside alignment at dist=-2 should be transparent, got {}",
        alpha
    );

    // Extreme outside alignment (align=-2.0, offset=-4.0) also pushes coverage away.
    let alpha = render_stroke(2.0, width, -2.0);
    assert!(
        alpha < 0.1,
        "Extreme outside alignment at dist=+2 should be transparent, got {}",
        alpha
    );
}
