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
    let instances = renderer.generate_instances(&shaped, position, color);

    // Should have one instance per glyph
    assert_eq!(instances.len(), shaped.glyphs.len());

    // Each instance should have valid data
    for (i, instance) in instances.iter().enumerate() {
        let glyph = &shaped.glyphs[i];

        // Position should be offset by glyph position
        assert_eq!(instance.pos[0], 100.0 + glyph.x_offset);
        assert_eq!(instance.pos[1], 100.0 + glyph.y_offset);

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

    let instances = renderer.generate_instances(&shaped, glam::Vec2::ZERO, glam::Vec4::ONE);

    // Empty text should produce no instances
    assert_eq!(instances.len(), 0);
}
