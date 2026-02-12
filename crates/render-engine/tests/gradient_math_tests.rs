//! CPU Reference Tests for Gradient Math
//!
//! These tests provide CPU-side reference implementations of gradient coordinate
//! calculations that can be compared against WGSL shader implementations.
//!
//! The shader engineer can use these tests to validate their WGSL gradient logic
//! by ensuring the shader produces the same `t` values for the same inputs.

use glam::{Vec2, Vec4};

/// CPU reference: Linear gradient coordinate calculation
///
/// Given a point (uv) in normalized rect space [0,1] x [0,1],
/// compute the gradient parameter t [0,1] along the gradient axis.
///
/// # Arguments
/// * `uv` - Point in normalized rect coordinates [0,1] x [0,1]
/// * `start` - Gradient start point (normalized)
/// * `end` - Gradient end point (normalized)
///
/// # Returns
/// Gradient parameter t, clamped to [0,1]
///
/// # WGSL Equivalent
/// ```wgsl
/// fn linear_gradient_t(uv: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
///     let dir = end - start;
///     return clamp(dot(uv - start, dir) / dot(dir, dir), 0.0, 1.0);
/// }
/// ```
pub fn linear_gradient_coord_cpu(uv: Vec2, start: Vec2, end: Vec2) -> f32 {
    let dir = end - start;
    let proj = (uv - start).dot(dir);
    let len_sq = dir.dot(dir);

    if len_sq < 1e-6 {
        return 0.0; // Degenerate gradient
    }

    let t = proj / len_sq;
    t.clamp(0.0, 1.0)
}

/// CPU reference: Radial gradient coordinate calculation
///
/// Computes distance from center, normalized by radius.
///
/// # WGSL Equivalent
/// ```wgsl
/// fn radial_gradient_t(uv: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
///     return clamp(length(uv - center) / radius, 0.0, 1.0);
/// }
/// ```
pub fn radial_gradient_coord_cpu(uv: Vec2, center: Vec2, radius: f32) -> f32 {
    if radius < 1e-6 {
        return 0.0; // Degenerate gradient
    }

    let dist = (uv - center).length();
    let t = dist / radius;
    t.clamp(0.0, 1.0)
}

/// CPU reference: Angular/conic gradient coordinate calculation
///
/// Computes angle from center point, normalized to [0,1].
///
/// # WGSL Equivalent
/// ```wgsl
/// fn angular_gradient_t(uv: vec2<f32>, center: vec2<f32>) -> f32 {
///     let d = uv - center;
///     return (atan2(d.y, d.x) + 3.14159265) / (2.0 * 3.14159265);
/// }
/// ```
pub fn angular_gradient_coord_cpu(uv: Vec2, center: Vec2) -> f32 {
    let d = uv - center;
    let angle = d.y.atan2(d.x); // Range: [-π, π]

    // Normalize to [0, 1]
    let t = (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
    t.clamp(0.0, 1.0)
}

/// CPU reference: Diamond gradient coordinate calculation
///
/// Computes diamond-shaped distance metric (Manhattan distance normalized by scale).
///
/// # WGSL Equivalent
/// ```wgsl
/// fn diamond_gradient_t(uv: vec2<f32>, center: vec2<f32>, scale: vec2<f32>) -> f32 {
///     let d = abs(uv - center);
///     return clamp((d.x / scale.x + d.y / scale.y), 0.0, 1.0);
/// }
/// ```
pub fn diamond_gradient_coord_cpu(uv: Vec2, center: Vec2, scale: Vec2) -> f32 {
    if scale.x < 1e-6 || scale.y < 1e-6 {
        return 0.0; // Degenerate gradient
    }

    let d = (uv - center).abs();
    let t = (d.x / scale.x) + (d.y / scale.y);
    t.clamp(0.0, 1.0)
}

/// Linear interpolation between two colors
pub fn lerp_color(a: Vec4, b: Vec4, t: f32) -> Vec4 {
    a + (b - a) * t
}

/// Interpolate gradient stops (CPU reference for gradient atlas rasterization)
///
/// This is the CPU-side logic used to rasterize gradient LUTs.
/// The shader samples the pre-rasterized texture instead of computing this.
pub fn interpolate_gradient_stops(t: f32, stops: &[(f32, Vec4)]) -> Vec4 {
    if stops.is_empty() {
        return Vec4::ZERO;
    }
    if stops.len() == 1 {
        return stops[0].1;
    }

    let t = t.clamp(0.0, 1.0);

    // Clamp to first/last stop if outside their bounds
    if t < stops[0].0 {
        return stops[0].1;
    }
    if t > stops[stops.len() - 1].0 {
        return stops[stops.len() - 1].1;
    }

    // Find surrounding stops
    let mut before = &stops[0];
    let mut after = &stops[stops.len() - 1];

    for i in 0..stops.len() - 1 {
        if stops[i].0 <= t && t <= stops[i + 1].0 {
            before = &stops[i];
            after = &stops[i + 1];
            break;
        }
    }

    // Handle same position
    if (after.0 - before.0).abs() < 1e-6 {
        return before.1;
    }

    // Linear interpolation
    let local_t = (t - before.0) / (after.0 - before.0);
    lerp_color(before.1, after.1, local_t)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn assert_near(a: f32, b: f32, msg: &str) {
        assert!((a - b).abs() < EPSILON, "{}: {} vs {}", msg, a, b);
    }

    #[test]
    fn test_linear_gradient_horizontal() {
        // Horizontal gradient from left (0,0.5) to right (1,0.5)
        let start = Vec2::new(0.0, 0.5);
        let end = Vec2::new(1.0, 0.5);

        // Left edge
        assert_near(
            linear_gradient_coord_cpu(Vec2::new(0.0, 0.5), start, end),
            0.0,
            "left edge",
        );

        // Middle
        assert_near(
            linear_gradient_coord_cpu(Vec2::new(0.5, 0.5), start, end),
            0.5,
            "middle",
        );

        // Right edge
        assert_near(
            linear_gradient_coord_cpu(Vec2::new(1.0, 0.5), start, end),
            1.0,
            "right edge",
        );

        // Off-axis (should project onto axis)
        let t = linear_gradient_coord_cpu(Vec2::new(0.25, 0.8), start, end);
        assert_near(t, 0.25, "off-axis projects correctly");
    }

    #[test]
    fn test_linear_gradient_vertical() {
        // Vertical gradient from top (0.5,0) to bottom (0.5,1)
        let start = Vec2::new(0.5, 0.0);
        let end = Vec2::new(0.5, 1.0);

        assert_near(
            linear_gradient_coord_cpu(Vec2::new(0.5, 0.0), start, end),
            0.0,
            "top",
        );
        assert_near(
            linear_gradient_coord_cpu(Vec2::new(0.5, 0.5), start, end),
            0.5,
            "middle",
        );
        assert_near(
            linear_gradient_coord_cpu(Vec2::new(0.5, 1.0), start, end),
            1.0,
            "bottom",
        );
    }

    #[test]
    fn test_linear_gradient_diagonal() {
        // Diagonal from (0,0) to (1,1)
        let start = Vec2::ZERO;
        let end = Vec2::ONE;

        assert_near(
            linear_gradient_coord_cpu(Vec2::ZERO, start, end),
            0.0,
            "origin",
        );
        assert_near(
            linear_gradient_coord_cpu(Vec2::new(0.5, 0.5), start, end),
            0.5,
            "center",
        );
        assert_near(
            linear_gradient_coord_cpu(Vec2::ONE, start, end),
            1.0,
            "opposite corner",
        );
    }

    #[test]
    fn test_linear_gradient_clamping() {
        let start = Vec2::new(0.25, 0.5);
        let end = Vec2::new(0.75, 0.5);

        // Before start
        let t = linear_gradient_coord_cpu(Vec2::new(0.0, 0.5), start, end);
        assert_near(t, 0.0, "clamps to 0 before start");

        // After end
        let t = linear_gradient_coord_cpu(Vec2::new(1.0, 0.5), start, end);
        assert_near(t, 1.0, "clamps to 1 after end");
    }

    #[test]
    fn test_radial_gradient_center() {
        let center = Vec2::new(0.5, 0.5);
        let radius = 0.5;

        // At center
        assert_near(
            radial_gradient_coord_cpu(center, center, radius),
            0.0,
            "at center",
        );

        // At radius distance
        assert_near(
            radial_gradient_coord_cpu(Vec2::new(1.0, 0.5), center, radius),
            1.0,
            "at radius (right)",
        );
        assert_near(
            radial_gradient_coord_cpu(Vec2::new(0.0, 0.5), center, radius),
            1.0,
            "at radius (left)",
        );
        assert_near(
            radial_gradient_coord_cpu(Vec2::new(0.5, 0.0), center, radius),
            1.0,
            "at radius (top)",
        );

        // Halfway
        let t = radial_gradient_coord_cpu(Vec2::new(0.75, 0.5), center, radius);
        assert_near(t, 0.5, "halfway to edge");
    }

    #[test]
    fn test_radial_gradient_clamping() {
        let center = Vec2::new(0.5, 0.5);
        let radius = 0.2;

        // Beyond radius
        let t = radial_gradient_coord_cpu(Vec2::new(1.0, 0.5), center, radius);
        assert_near(t, 1.0, "clamps to 1 beyond radius");
    }

    #[test]
    fn test_angular_gradient_quadrants() {
        let center = Vec2::new(0.5, 0.5);

        // Right (0°): atan2(0, 0.5) = 0, normalized = (0 + π)/(2π) = 0.5
        let t = angular_gradient_coord_cpu(Vec2::new(1.0, 0.5), center);
        assert_near(t, 0.5, "right is 0° -> t=0.5");

        // Top (90°): atan2(-0.5, 0) = -π/2, normalized = (-π/2 + π)/(2π) = 0.25
        let t = angular_gradient_coord_cpu(Vec2::new(0.5, 0.0), center);
        assert_near(t, 0.25, "top is -90° -> t=0.25");

        // Left (180°): atan2(0, -0.5) = π, normalized = (π + π)/(2π) = 1.0
        // Note: Angular gradients wrap, so t=1.0 is same as t=0.0
        let t = angular_gradient_coord_cpu(Vec2::new(0.0, 0.5), center);
        assert!(t > 0.99, "left is 180° -> t≈1.0 (wraps to 0)");

        // Bottom (270°): atan2(0.5, 0) = π/2, normalized = (π/2 + π)/(2π) = 0.75
        let t = angular_gradient_coord_cpu(Vec2::new(0.5, 1.0), center);
        assert_near(t, 0.75, "bottom is 90° -> t=0.75");
    }

    #[test]
    fn test_diamond_gradient_axes() {
        let center = Vec2::new(0.5, 0.5);
        let scale = Vec2::new(0.5, 0.5);

        // Center
        assert_near(
            diamond_gradient_coord_cpu(center, center, scale),
            0.0,
            "at center",
        );

        // Along X axis
        let t = diamond_gradient_coord_cpu(Vec2::new(1.0, 0.5), center, scale);
        assert_near(t, 1.0, "right edge");

        // Along Y axis
        let t = diamond_gradient_coord_cpu(Vec2::new(0.5, 1.0), center, scale);
        assert_near(t, 1.0, "bottom edge");

        // Diagonal (45°)
        let t = diamond_gradient_coord_cpu(Vec2::new(0.75, 0.75), center, scale);
        assert_near(t, 1.0, "diagonal at scale distance");
    }

    #[test]
    fn test_color_interpolation() {
        let red = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let blue = Vec4::new(0.0, 0.0, 1.0, 1.0);

        // Start
        let color = lerp_color(red, blue, 0.0);
        assert_eq!(color, red, "t=0 returns start color");

        // End
        let color = lerp_color(red, blue, 1.0);
        assert_eq!(color, blue, "t=1 returns end color");

        // Middle (should be purple-ish)
        let color = lerp_color(red, blue, 0.5);
        assert_near(color.x, 0.5, "red component");
        assert_near(color.y, 0.0, "green component");
        assert_near(color.z, 0.5, "blue component");
        assert_near(color.w, 1.0, "alpha component");
    }

    #[test]
    fn test_gradient_stops_interpolation() {
        let stops = vec![
            (0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)), // Red
            (0.5, Vec4::new(0.0, 1.0, 0.0, 1.0)), // Green
            (1.0, Vec4::new(0.0, 0.0, 1.0, 1.0)), // Blue
        ];

        // At stops
        let color = interpolate_gradient_stops(0.0, &stops);
        assert_eq!(color, stops[0].1, "t=0 returns first stop");

        let color = interpolate_gradient_stops(0.5, &stops);
        assert_eq!(color, stops[1].1, "t=0.5 returns middle stop");

        let color = interpolate_gradient_stops(1.0, &stops);
        assert_eq!(color, stops[2].1, "t=1 returns last stop");

        // Between stops (0.25 is halfway between red and green)
        let color = interpolate_gradient_stops(0.25, &stops);
        assert_near(color.x, 0.5, "red channel");
        assert_near(color.y, 0.5, "green channel");
        assert_near(color.z, 0.0, "blue channel");
    }

    #[test]
    fn test_gradient_stops_clamping() {
        let stops = vec![
            (0.2, Vec4::new(1.0, 0.0, 0.0, 1.0)),
            (0.8, Vec4::new(0.0, 0.0, 1.0, 1.0)),
        ];

        // Before first stop
        let color = interpolate_gradient_stops(0.0, &stops);
        assert_eq!(color, stops[0].1, "clamps to first stop");

        // After last stop
        let color = interpolate_gradient_stops(1.0, &stops);
        assert_eq!(color, stops[1].1, "clamps to last stop");
    }

    #[test]
    fn test_degenerate_linear_gradient() {
        // Same start and end
        let start = Vec2::new(0.5, 0.5);
        let end = Vec2::new(0.5, 0.5);

        let t = linear_gradient_coord_cpu(Vec2::new(0.3, 0.7), start, end);
        assert_near(t, 0.0, "degenerate gradient returns 0");
    }

    #[test]
    fn test_degenerate_radial_gradient() {
        // Zero radius
        let t = radial_gradient_coord_cpu(Vec2::ONE, Vec2::ZERO, 0.0);
        assert_near(t, 0.0, "zero radius returns 0");
    }

    #[test]
    fn test_degenerate_diamond_gradient() {
        // Zero scale
        let t = diamond_gradient_coord_cpu(Vec2::ONE, Vec2::ZERO, Vec2::ZERO);
        assert_near(t, 0.0, "zero scale returns 0");
    }
}
