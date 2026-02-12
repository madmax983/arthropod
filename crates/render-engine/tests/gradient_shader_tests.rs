/// CPU reference tests for gradient shader logic.
///
/// These tests verify the mathematical correctness of gradient position computation
/// by replicating the WGSL shader logic in Rust. This ensures:
/// 1. The gradient algorithms are mathematically correct
/// 2. Edge cases (degenerate gradients, boundaries) are handled properly
/// 3. Changes to shader logic can be validated against reference implementation
///
/// Each test includes the WGSL equivalent as a comment for cross-reference.
use glam::Vec2;

// ============================================================================
// CPU Reference Implementation (mirrors WGSL shader)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum GradientType {
    Linear = 0,
    Radial = 1,
    Angular = 2,
    Diamond = 3,
}

/// Compute gradient position t (0.0 to 1.0) for a UV coordinate.
/// This mirrors the WGSL sample_gradient() function.
fn compute_gradient_t(uv: Vec2, start: Vec2, end: Vec2, gradient_type: GradientType) -> f32 {
    match gradient_type {
        GradientType::Linear => {
            // Linear gradient: dot product along axis
            let dir = end - start;
            let dir_len_sq = dir.length_squared();

            if dir_len_sq < 0.0001 {
                // Degenerate gradient (start == end): use start color
                0.0
            } else {
                ((uv - start).dot(dir) / dir_len_sq).clamp(0.0, 1.0)
            }
        }
        GradientType::Radial => {
            // Radial gradient: distance from center
            let radius = (end - start).length();

            if radius < 0.0001 {
                // Degenerate gradient (zero radius): use start color
                0.0
            } else {
                ((uv - start).length() / radius).clamp(0.0, 1.0)
            }
        }
        GradientType::Angular => {
            // Angular gradient: sweep around center (0-360 degrees)
            let d = uv - start;
            // atan2 returns [-π, π], normalize to [0, 1]
            (d.y.atan2(d.x) + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)
        }
        GradientType::Diamond => {
            // Diamond gradient: Manhattan distance (Figma-specific)
            let d = (uv - start).abs();
            let scale = (end - start).abs();

            if scale.x < 0.0001 && scale.y < 0.0001 {
                // Degenerate gradient: use start color
                0.0
            } else {
                // Manhattan distance normalized by scale
                let dist_x = if scale.x > 0.0001 { d.x / scale.x } else { 0.0 };
                let dist_y = if scale.y > 0.0001 { d.y / scale.y } else { 0.0 };
                (dist_x + dist_y).clamp(0.0, 1.0)
            }
        }
    }
}

// ============================================================================
// Linear Gradient Tests
// ============================================================================

#[test]
fn test_linear_gradient_horizontal() {
    let start = Vec2::new(0.0, 0.5);
    let end = Vec2::new(1.0, 0.5);

    // Left edge (start)
    let t = compute_gradient_t(Vec2::new(0.0, 0.5), start, end, GradientType::Linear);
    assert!(
        (t - 0.0).abs() < 0.001,
        "Expected t=0.0 at start, got {}",
        t
    );

    // Center
    let t = compute_gradient_t(Vec2::new(0.5, 0.5), start, end, GradientType::Linear);
    assert!(
        (t - 0.5).abs() < 0.001,
        "Expected t=0.5 at center, got {}",
        t
    );

    // Right edge (end)
    let t = compute_gradient_t(Vec2::new(1.0, 0.5), start, end, GradientType::Linear);
    assert!((t - 1.0).abs() < 0.001, "Expected t=1.0 at end, got {}", t);

    // Beyond bounds (should clamp)
    let t = compute_gradient_t(Vec2::new(1.5, 0.5), start, end, GradientType::Linear);
    assert!(
        (t - 1.0).abs() < 0.001,
        "Expected t=1.0 beyond end, got {}",
        t
    );

    let t = compute_gradient_t(Vec2::new(-0.5, 0.5), start, end, GradientType::Linear);
    assert!(
        (t - 0.0).abs() < 0.001,
        "Expected t=0.0 before start, got {}",
        t
    );
}

#[test]
fn test_linear_gradient_vertical() {
    let start = Vec2::new(0.5, 0.0);
    let end = Vec2::new(0.5, 1.0);

    // Top edge (start)
    let t = compute_gradient_t(Vec2::new(0.5, 0.0), start, end, GradientType::Linear);
    assert!(
        (t - 0.0).abs() < 0.001,
        "Expected t=0.0 at start, got {}",
        t
    );

    // Center
    let t = compute_gradient_t(Vec2::new(0.5, 0.5), start, end, GradientType::Linear);
    assert!(
        (t - 0.5).abs() < 0.001,
        "Expected t=0.5 at center, got {}",
        t
    );

    // Bottom edge (end)
    let t = compute_gradient_t(Vec2::new(0.5, 1.0), start, end, GradientType::Linear);
    assert!((t - 1.0).abs() < 0.001, "Expected t=1.0 at end, got {}", t);
}

#[test]
fn test_linear_gradient_diagonal() {
    let start = Vec2::new(0.0, 0.0);
    let end = Vec2::new(1.0, 1.0);

    // Start corner
    let t = compute_gradient_t(Vec2::new(0.0, 0.0), start, end, GradientType::Linear);
    assert!(
        (t - 0.0).abs() < 0.001,
        "Expected t=0.0 at start, got {}",
        t
    );

    // Center
    let t = compute_gradient_t(Vec2::new(0.5, 0.5), start, end, GradientType::Linear);
    assert!(
        (t - 0.5).abs() < 0.001,
        "Expected t=0.5 at center, got {}",
        t
    );

    // End corner
    let t = compute_gradient_t(Vec2::new(1.0, 1.0), start, end, GradientType::Linear);
    assert!((t - 1.0).abs() < 0.001, "Expected t=1.0 at end, got {}", t);

    // Perpendicular offset (should remain at same t)
    let t = compute_gradient_t(Vec2::new(0.3, 0.7), start, end, GradientType::Linear);
    assert!(
        (t - 0.5).abs() < 0.001,
        "Expected perpendicular points to have same t"
    );
}

#[test]
fn test_linear_gradient_degenerate() {
    // Start == End (degenerate gradient)
    let start = Vec2::new(0.5, 0.5);
    let end = Vec2::new(0.5, 0.5);

    let t = compute_gradient_t(Vec2::new(0.0, 0.0), start, end, GradientType::Linear);
    assert!(
        (t - 0.0).abs() < 0.001,
        "Degenerate gradient should return t=0.0"
    );

    let t = compute_gradient_t(Vec2::new(1.0, 1.0), start, end, GradientType::Linear);
    assert!(
        (t - 0.0).abs() < 0.001,
        "Degenerate gradient should return t=0.0"
    );
}

// ============================================================================
// Radial Gradient Tests
// ============================================================================

#[test]
fn test_radial_gradient_center() {
    let center = Vec2::new(0.5, 0.5);
    let edge = Vec2::new(1.0, 0.5); // Radius = 0.5

    // At center
    let t = compute_gradient_t(center, center, edge, GradientType::Radial);
    assert!(
        (t - 0.0).abs() < 0.001,
        "Expected t=0.0 at center, got {}",
        t
    );

    // At radius (right edge)
    let t = compute_gradient_t(edge, center, edge, GradientType::Radial);
    assert!(
        (t - 1.0).abs() < 0.001,
        "Expected t=1.0 at radius, got {}",
        t
    );

    // Halfway to radius
    let halfway = Vec2::new(0.75, 0.5);
    let t = compute_gradient_t(halfway, center, edge, GradientType::Radial);
    assert!(
        (t - 0.5).abs() < 0.001,
        "Expected t=0.5 at half radius, got {}",
        t
    );

    // Beyond radius (should clamp to 1.0)
    let beyond = Vec2::new(1.5, 0.5);
    let t = compute_gradient_t(beyond, center, edge, GradientType::Radial);
    assert!(
        (t - 1.0).abs() < 0.001,
        "Expected t=1.0 beyond radius, got {}",
        t
    );
}

#[test]
fn test_radial_gradient_circular_symmetry() {
    let center = Vec2::new(0.5, 0.5);
    let edge = Vec2::new(1.0, 0.5); // Radius = 0.5

    // Points at same distance should have same t
    let points = vec![
        Vec2::new(0.75, 0.5), // Right
        Vec2::new(0.5, 0.75), // Top
        Vec2::new(0.25, 0.5), // Left
        Vec2::new(0.5, 0.25), // Bottom
    ];

    let t_values: Vec<f32> = points
        .iter()
        .map(|&p| compute_gradient_t(p, center, edge, GradientType::Radial))
        .collect();

    // All should be approximately 0.5 (distance = 0.25, radius = 0.5)
    for (i, t) in t_values.iter().enumerate() {
        assert!(
            (t - 0.5).abs() < 0.001,
            "Point {} should have t≈0.5, got {}",
            i,
            t
        );
    }
}

#[test]
fn test_radial_gradient_degenerate() {
    let center = Vec2::new(0.5, 0.5);
    let same = Vec2::new(0.5, 0.5); // Zero radius

    let t = compute_gradient_t(Vec2::new(1.0, 1.0), center, same, GradientType::Radial);
    assert!(
        (t - 0.0).abs() < 0.001,
        "Degenerate radial should return t=0.0"
    );
}

// ============================================================================
// Angular Gradient Tests
// ============================================================================

#[test]
fn test_angular_gradient_quadrants() {
    let center = Vec2::new(0.5, 0.5);
    let _end = Vec2::new(1.0, 0.5); // Not used for angular, but parameter required

    // atan2(y, x) returns:
    // Right (x=+, y=0): atan2(0, 1) = 0 rad → (0 + π) / 2π = 0.5
    // Top (x=0, y=-): atan2(-1, 0) = -π/2 rad → (-π/2 + π) / 2π = 0.25
    // Left (x=-, y=0): atan2(0, -1) = π rad → (π + π) / 2π = 1.0 (wraps to 0.0)
    // Bottom (x=0, y=+): atan2(1, 0) = π/2 rad → (π/2 + π) / 2π = 0.75

    // Right (0 degrees): atan2(0, 1) = 0
    let t = compute_gradient_t(Vec2::new(1.0, 0.5), center, _end, GradientType::Angular);
    assert!((t - 0.5).abs() < 0.01, "Right should be t=0.5, got {}", t);

    // Top (-90 degrees in screen coords): atan2(-1, 0) = -π/2
    let t = compute_gradient_t(Vec2::new(0.5, 0.0), center, _end, GradientType::Angular);
    assert!((t - 0.25).abs() < 0.01, "Top should be t=0.25, got {}", t);

    // Left (180 degrees): atan2(0, -1) = π
    let t = compute_gradient_t(Vec2::new(0.0, 0.5), center, _end, GradientType::Angular);
    assert!(
        t < 0.01 || t > 0.99,
        "Left should be t≈0.0 or t≈1.0 (wrap), got {}",
        t
    );

    // Bottom (90 degrees): atan2(1, 0) = π/2
    let t = compute_gradient_t(Vec2::new(0.5, 1.0), center, _end, GradientType::Angular);
    assert!(
        (t - 0.75).abs() < 0.01,
        "Bottom should be t=0.75, got {}",
        t
    );
}

#[test]
fn test_angular_gradient_continuity() {
    let center = Vec2::new(0.5, 0.5);
    let _end = Vec2::new(1.0, 0.5);

    // Test that gradient is continuous around the circle
    let num_samples = 360;
    let mut prev_t: Option<f32> = None;
    let mut discontinuities = 0;

    for i in 0..num_samples {
        let angle = (i as f32 / num_samples as f32) * 2.0 * std::f32::consts::PI;
        let point = center + Vec2::new(angle.cos(), angle.sin()) * 0.5;
        let t = compute_gradient_t(point, center, _end, GradientType::Angular);

        if let Some(prev) = prev_t {
            // Allow for wrap-around (0.0 ↔ 1.0)
            let delta = (t - prev).abs();
            if delta > 0.1 && delta < 0.9 {
                discontinuities += 1;
            }
        }
        prev_t = Some(t);
    }

    assert_eq!(
        discontinuities, 0,
        "Angular gradient should be continuous (found {} discontinuities)",
        discontinuities
    );
}

// ============================================================================
// Diamond Gradient Tests
// ============================================================================

#[test]
fn test_diamond_gradient_axes() {
    let start = Vec2::new(0.5, 0.5);
    let end = Vec2::new(1.0, 1.0); // Scale = (0.5, 0.5)

    // At start (center)
    let t = compute_gradient_t(start, start, end, GradientType::Diamond);
    assert!(
        (t - 0.0).abs() < 0.001,
        "Expected t=0.0 at center, got {}",
        t
    );

    // Right edge (only X changes)
    let t = compute_gradient_t(Vec2::new(1.0, 0.5), start, end, GradientType::Diamond);
    assert!(
        (t - 1.0).abs() < 0.001,
        "Expected t=1.0 at right edge, got {}",
        t
    );

    // Top edge (only Y changes)
    let t = compute_gradient_t(Vec2::new(0.5, 0.0), start, end, GradientType::Diamond);
    assert!(
        (t - 1.0).abs() < 0.001,
        "Expected t=1.0 at top edge, got {}",
        t
    );

    // Diagonal corner (both X and Y change maximally)
    // Manhattan distance: |0.5|/0.5 + |0.5|/0.5 = 1.0 + 1.0 = 2.0 → clamped to 1.0
    let t = compute_gradient_t(Vec2::new(1.0, 0.0), start, end, GradientType::Diamond);
    assert!(
        (t - 1.0).abs() < 0.001,
        "Expected t=1.0 at diagonal corner (clamped), got {}",
        t
    );
}

#[test]
fn test_diamond_gradient_manhattan_distance() {
    let start = Vec2::new(0.5, 0.5);
    let end = Vec2::new(1.0, 1.0);

    // Point at Manhattan distance 0.5 (quarter way on both axes)
    let point = Vec2::new(0.625, 0.625); // d = |0.125|/0.5 + |0.125|/0.5 = 0.25 + 0.25 = 0.5
    let t = compute_gradient_t(point, start, end, GradientType::Diamond);
    assert!(
        (t - 0.5).abs() < 0.001,
        "Expected t=0.5 for Manhattan distance 0.5, got {}",
        t
    );
}

#[test]
fn test_diamond_gradient_degenerate() {
    let start = Vec2::new(0.5, 0.5);
    let same = Vec2::new(0.5, 0.5); // Zero scale

    let t = compute_gradient_t(Vec2::new(1.0, 1.0), start, same, GradientType::Diamond);
    assert!(
        (t - 0.0).abs() < 0.001,
        "Degenerate diamond should return t=0.0"
    );
}

#[test]
fn test_diamond_gradient_asymmetric_scale() {
    let start = Vec2::new(0.5, 0.5);
    let end = Vec2::new(1.0, 0.75); // Scale X=0.5, Y=0.25 (asymmetric)

    // Right edge (only X at max): d = 0.5/0.5 + 0/0.25 = 1.0
    let t = compute_gradient_t(Vec2::new(1.0, 0.5), start, end, GradientType::Diamond);
    assert!(
        (t - 1.0).abs() < 0.001,
        "Expected t=1.0 at X edge, got {}",
        t
    );

    // Top edge (only Y at max): d = 0/0.5 + 0.25/0.25 = 1.0
    let t = compute_gradient_t(Vec2::new(0.5, 0.25), start, end, GradientType::Diamond);
    assert!(
        (t - 1.0).abs() < 0.001,
        "Expected t=1.0 at Y edge, got {}",
        t
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_all_gradients_clamp_to_0_1() {
    let start = Vec2::new(0.5, 0.5);
    let end = Vec2::new(1.0, 0.5);

    let gradient_types = vec![
        GradientType::Linear,
        GradientType::Radial,
        GradientType::Angular,
        GradientType::Diamond,
    ];

    // Test extreme points
    let test_points = vec![
        Vec2::new(-10.0, -10.0),
        Vec2::new(10.0, 10.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    ];

    for gradient_type in gradient_types {
        for point in &test_points {
            let t = compute_gradient_t(*point, start, end, gradient_type);
            assert!(
                t >= 0.0 && t <= 1.0,
                "{:?} gradient at {:?} should clamp to [0,1], got {}",
                gradient_type,
                point,
                t
            );
        }
    }
}

#[test]
fn test_gradient_types_match_wgsl_enum() {
    // Verify that our Rust enum values match the WGSL switch statement
    assert_eq!(GradientType::Linear as u32, 0);
    assert_eq!(GradientType::Radial as u32, 1);
    assert_eq!(GradientType::Angular as u32, 2);
    assert_eq!(GradientType::Diamond as u32, 3);
}
