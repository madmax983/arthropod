//! Text rendering integration tests - Written FIRST following TDD

use render_engine::backend::text::{GlyphAtlas, TextRenderer};
use text_engine::TextEngine;
use std::time::Instant;

#[test]
fn test_glyph_atlas_caching() {
    let mut atlas = GlyphAtlas::new(512, 512);

    // Create a fake glyph
    let glyph_id = 65; // 'A'
    let font_size = 16;

    // First rasterization should be slow
    let start = Instant::now();
    let coords1 = atlas.get_or_rasterize(glyph_id, font_size, &test_font_data());
    let first_time = start.elapsed();

    // Second time should hit cache (much faster)
    let start = Instant::now();
    let coords2 = atlas.get_or_rasterize(glyph_id, font_size, &test_font_data());
    let second_time = start.elapsed();

    // Cache hit should be significantly faster (at least 3x)
    assert!(
        second_time < first_time / 3,
        "Cache hit should be >3x faster (first: {:?}, second: {:?})",
        first_time,
        second_time
    );

    // Should return same texture coordinates
    assert_eq!(coords1.u0, coords2.u0);
    assert_eq!(coords1.v0, coords2.v0);
    assert_eq!(coords1.u1, coords2.u1);
    assert_eq!(coords1.v1, coords2.v1);
}

#[test]
fn test_glyph_atlas_packing() {
    let mut atlas = GlyphAtlas::new(128, 128);

    // Add multiple glyphs - should pack them efficiently
    let font_data = test_font_data();
    if font_data.is_empty() {
        // Skip test if font not available
        println!("Test skipped: No font data available");
        return;
    }

    for glyph_id in 0..10 {
        let coords = atlas.get_or_rasterize(glyph_id, 16, &font_data);

        // All texture coordinates should be in [0, 1] range
        assert!(coords.u0 >= 0.0 && coords.u0 <= 1.0, "u0 out of range: {}", coords.u0);
        assert!(coords.v0 >= 0.0 && coords.v0 <= 1.0, "v0 out of range: {}", coords.v0);
        assert!(coords.u1 >= 0.0 && coords.u1 <= 1.0, "u1 out of range: {}", coords.u1);
        assert!(coords.v1 >= 0.0 && coords.v1 <= 1.0, "v1 out of range: {}", coords.v1);

        // u1 should be >= u0, v1 should be >= v0 (allow equality for small glyphs)
        assert!(coords.u1 >= coords.u0, "u1 ({}) should be >= u0 ({})", coords.u1, coords.u0);
        assert!(coords.v1 >= coords.v0, "v1 ({}) should be >= v0 ({})", coords.v1, coords.v0);
    }
}

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

    let instances = renderer.generate_instances(
        &shaped,
        glam::Vec2::ZERO,
        glam::Vec4::ONE,
    );

    // Empty text should produce no instances
    assert_eq!(instances.len(), 0);
}

// Helper function to get test font data
fn test_font_data() -> Vec<u8> {
    // Load a system font for testing
    #[cfg(target_os = "windows")]
    {
        if let Ok(data) = std::fs::read(r"C:\Windows\Fonts\arial.ttf") {
            return data;
        }
    }

    // Return empty vec if font not found (test will skip)
    Vec::new()
}
