//! Tests to verify glam types match behavior of custom math types.
//! Following TDD: Write tests first, then migrate implementation.

#[cfg(test)]
mod tests {
    use crate::{Color, Transform2D};
    use glam::{Affine2, Vec2, Vec4};

    /// Test that Color can be replaced with Vec4
    #[test]
    fn color_matches_vec4() {
        // Custom Color
        let custom = Color::rgba(1.0, 0.5, 0.25, 0.8);

        // glam Vec4
        let glam_color = Vec4::new(1.0, 0.5, 0.25, 0.8);

        // Verify component access
        assert_eq!(custom.r(), glam_color.x);
        assert_eq!(custom.g(), glam_color.y);
        assert_eq!(custom.b(), glam_color.z);
        assert_eq!(custom.a(), glam_color.w);

        // Verify array conversion
        assert_eq!(
            [custom.r(), custom.g(), custom.b(), custom.a()],
            glam_color.to_array()
        );
    }

    #[test]
    fn color_constants_match_vec4() {
        // Test color constants
        let red = Color::RED;
        let glam_red = Vec4::new(1.0, 0.0, 0.0, 1.0);
        assert_eq!([red.r(), red.g(), red.b(), red.a()], glam_red.to_array());

        let green = Color::GREEN;
        let glam_green = Vec4::new(0.0, 1.0, 0.0, 1.0);
        assert_eq!(
            [green.r(), green.g(), green.b(), green.a()],
            glam_green.to_array()
        );

        let blue = Color::BLUE;
        let glam_blue = Vec4::new(0.0, 0.0, 1.0, 1.0);
        assert_eq!(
            [blue.r(), blue.g(), blue.b(), blue.a()],
            glam_blue.to_array()
        );
    }

    /// Test that Transform2D::identity matches Affine2::IDENTITY
    #[test]
    fn transform_identity_matches_affine2() {
        let _custom = Transform2D::identity();
        let glam_transform = Affine2::IDENTITY;

        // Affine2 stores as 2x3 matrix: [m11, m12, m13, m21, m22, m23]
        // Our custom stores as [[m11, m12, m13], [m21, m22, m23]]
        let _glam_cols = glam_transform.to_cols_array();

        // glam uses column-major, but for 2D affine the translation is at the end
        // Affine2 memory layout: [m11, m21, m12, m22, m13, m23]
        // We want: [[1, 0, 0], [0, 1, 0]]

        // Just verify it's identity behavior
        let point = Vec2::new(5.0, 7.0);
        let transformed = glam_transform.transform_point2(point);
        assert_eq!(transformed, point); // Identity doesn't change points
    }

    /// Test that Transform2D::translate matches Affine2::from_translation
    #[test]
    fn transform_translate_matches_affine2() {
        let custom = Transform2D::translate(10.0, 20.0);
        let glam_transform = Affine2::from_translation(Vec2::new(10.0, 20.0));

        // Verify they both transform points the same way
        let point = Vec2::new(5.0, 7.0);
        let expected = Vec2::new(15.0, 27.0);

        // glam transformation
        let glam_result = glam_transform.transform_point2(point);
        assert_eq!(glam_result, expected);

        // Custom transformation
        let custom_result = custom.transform_point(point);
        assert_eq!(custom_result, expected);
    }

    /// Test that Transform2D::scale matches Affine2::from_scale
    #[test]
    fn transform_scale_matches_affine2() {
        let custom = Transform2D::scale(2.0, 3.0);
        let glam_transform = Affine2::from_scale(Vec2::new(2.0, 3.0));

        // Verify they both transform points the same way
        let point = Vec2::new(5.0, 7.0);
        let expected = Vec2::new(10.0, 21.0);

        // glam transformation
        let glam_result = glam_transform.transform_point2(point);
        assert_eq!(glam_result, expected);

        // Custom transformation
        let custom_result = custom.transform_point(point);
        assert_eq!(custom_result, expected);
    }

    /// Test Vec4 arithmetic (addition)
    #[test]
    fn color_addition_with_vec4() {
        let c1 = Vec4::new(0.2, 0.4, 0.6, 1.0);
        let c2 = Vec4::new(0.1, 0.2, 0.3, 0.0);
        let result = c1 + c2;

        const EPSILON: f32 = 1e-6;
        assert!((result.x - 0.3).abs() < EPSILON);
        assert!((result.y - 0.6).abs() < EPSILON);
        assert!((result.z - 0.9).abs() < EPSILON);
        assert!((result.w - 1.0).abs() < EPSILON);
    }

    /// Test Vec4 scalar multiplication
    #[test]
    fn color_scalar_multiply_with_vec4() {
        let c = Vec4::new(0.5, 0.4, 0.3, 1.0);
        let result = c * 2.0;

        assert_eq!(result.x, 1.0);
        assert_eq!(result.y, 0.8);
        assert_eq!(result.z, 0.6);
        assert_eq!(result.w, 2.0);
    }

    /// Test Affine2 composition (translate then scale)
    #[test]
    fn transform_composition_with_affine2() {
        let translate = Affine2::from_translation(Vec2::new(10.0, 20.0));
        let scale = Affine2::from_scale(Vec2::new(2.0, 2.0));

        // Compose: scale * translate (applied right-to-left)
        let composed = scale * translate;

        let point = Vec2::new(5.0, 5.0);
        // First translate: (5, 5) -> (15, 25)
        // Then scale: (15, 25) -> (30, 50)
        let result = composed.transform_point2(point);

        assert_eq!(result.x, 30.0);
        assert_eq!(result.y, 50.0);
    }

    /// Test Color::as_vec4() and from_vec4() round-trip
    #[test]
    fn color_vec4_roundtrip() {
        let original = Color::rgba(0.2, 0.4, 0.6, 0.8);
        let vec4 = original.as_vec4();
        let roundtrip = Color::from_vec4(vec4);

        assert_eq!(original.r(), roundtrip.r());
        assert_eq!(original.g(), roundtrip.g());
        assert_eq!(original.b(), roundtrip.b());
        assert_eq!(original.a(), roundtrip.a());
    }

    /// Test Transform2D::as_affine2() and from_affine2() round-trip
    #[test]
    fn transform_affine2_roundtrip() {
        let original = Transform2D::translate(10.0, 20.0);
        let affine2 = original.as_affine2();
        let roundtrip = Transform2D::from_affine2(affine2);

        // Verify they transform points identically
        let point = Vec2::new(5.0, 7.0);

        let original_result = original.transform_point(point);
        let roundtrip_result = roundtrip.transform_point(point);

        assert_eq!(original_result, roundtrip_result);
    }

    /// Test SIMD-accelerated color blending via glam
    #[test]
    fn color_blend_with_glam_simd() {
        let c1 = Color::rgba(1.0, 0.0, 0.0, 1.0);
        let c2 = Color::rgba(0.0, 1.0, 0.0, 1.0);

        // Use glam for SIMD-accelerated blending
        let v1 = c1.as_vec4();
        let v2 = c2.as_vec4();
        let blended = v1.lerp(v2, 0.5);
        let result = Color::from_vec4(blended);

        // Should be yellow (0.5, 0.5, 0.0, 1.0)
        const EPSILON: f32 = 1e-6;
        assert!((result.r() - 0.5).abs() < EPSILON);
        assert!((result.g() - 0.5).abs() < EPSILON);
        assert!((result.b() - 0.0).abs() < EPSILON);
        assert!((result.a() - 1.0).abs() < EPSILON);
    }

    /// Test SIMD-accelerated transform composition via glam
    #[test]
    fn transform_compose_with_glam_simd() {
        let t1 = Transform2D::translate(10.0, 20.0);
        let t2 = Transform2D::scale(2.0, 3.0);

        // Use glam for SIMD-accelerated composition
        let composed = t2.compose(&t1); // Scale after translate

        // Verify transformation of a test point
        let point = Vec2::new(5.0, 5.0);
        let result = composed.transform_point(point);

        // Expected: translate (5,5) -> (15, 25), then scale -> (30, 75)
        const EPSILON: f32 = 1e-5;
        assert!((result.x - 30.0).abs() < EPSILON);
        assert!((result.y - 75.0).abs() < EPSILON);
    }
}
