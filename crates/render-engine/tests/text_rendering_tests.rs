//! Text rendering integration tests - Written FIRST following TDD
//!
//! Note: After cosmic-text migration, low-level glyph atlas caching tests were removed
//! since cosmic-text's SwashCache handles this internally with proven performance.
//! We now focus on testing the integration layer.

use render_engine::backend::text::TextRenderer;
use text_engine::TextEngine;

#[test]
fn test_text_renderer_instance_generation() {
    let mut renderer = TextRenderer::new();

    // Create shaped text
    let mut engine = TextEngine::new();
    let shaped = engine.shape_text("Hello", 16.0);

    // Generate instances
    let position = glam::Vec2::new(100.0, 100.0);
    let color = glam::Vec4::new(1.0, 1.0, 1.0, 1.0);
    let mut instances = Vec::new();
    renderer.generate_instances_into(&shaped, position, color, &mut instances);

    // Zero-size glyphs (spaces, missing) are skipped, so instance count <= glyph count.
    // "Hello" has no spaces so all visible glyphs should produce instances.
    assert!(
        !instances.is_empty(),
        "Should produce at least one instance for 'Hello'"
    );
    assert!(
        instances.len() <= shaped.glyphs.len(),
        "Instance count should not exceed glyph count"
    );

    // Each instance should have valid data
    for instance in &instances {
        // Position should be near the base position (offset by glyph + placement)
        assert!(
            instance.pos[0] >= 90.0,
            "X position should be near base (got {})",
            instance.pos[0]
        );
        assert!(
            instance.pos[1] >= 70.0,
            "Y position should be near base (got {})",
            instance.pos[1]
        );

        // Size should be positive (non-zero glyphs only)
        assert!(instance.size[0] > 0.0, "Glyph width should be positive");
        assert!(instance.size[1] > 0.0, "Glyph height should be positive");

        // Color should match
        assert_eq!(instance.color[0], 1.0);
        assert_eq!(instance.color[1], 1.0);
        assert_eq!(instance.color[2], 1.0);
        assert_eq!(instance.color[3], 1.0);

        // Texture coordinates should be valid
        assert!(instance.tex_coords[0] >= 0.0); // u0
        assert!(instance.tex_coords[1] >= 0.0); // v0
        assert!(instance.tex_coords[2] <= 1.0); // u1
        assert!(instance.tex_coords[3] <= 1.0); // v1
    }
}

#[test]
fn test_text_renderer_empty_text() {
    let mut renderer = TextRenderer::new();

    let mut engine = TextEngine::new();
    let shaped = engine.shape_text("", 16.0);

    let mut instances = Vec::new();
    renderer.generate_instances_into(&shaped, glam::Vec2::ZERO, glam::Vec4::ONE, &mut instances);

    // Empty text should produce no instances
    assert_eq!(instances.len(), 0);
}
