//! Text shaping tests - Written FIRST following TDD

use text_engine::{TextEngine, ShapedText};

#[test]
fn test_simple_text_shaping() {
    let mut engine = TextEngine::new();

    let text = "Hello World";
    let font_size = 16.0;

    let shaped = engine.shape_text(text, font_size);

    // Should have glyphs for all characters (11 including space)
    assert!(shaped.glyphs.len() > 0, "Should have glyphs");

    // Glyphs should have positive advance widths
    for glyph in &shaped.glyphs {
        assert!(glyph.x_advance > 0.0, "Glyph should have positive advance");
    }

    // Bounds should be reasonable for 16px text
    assert!(shaped.bounds.width > 50.0, "Text should have reasonable width");
    assert!(shaped.bounds.height > 10.0, "Text should have reasonable height");
}

#[test]
fn test_empty_string_shaping() {
    let mut engine = TextEngine::new();

    let shaped = engine.shape_text("", 16.0);

    // Empty string should produce no glyphs
    assert_eq!(shaped.glyphs.len(), 0);
    assert_eq!(shaped.bounds.width, 0.0);
}

#[test]
fn test_font_size_affects_metrics() {
    let mut engine = TextEngine::new();

    let shaped_16 = engine.shape_text("Test", 16.0);
    let shaped_32 = engine.shape_text("Test", 32.0);

    // Larger font size should produce larger bounds
    assert!(shaped_32.bounds.width > shaped_16.bounds.width);
    assert!(shaped_32.bounds.height > shaped_16.bounds.height);
}

#[test]
fn test_unicode_text_shaping() {
    let mut engine = TextEngine::new();

    // Test with various Unicode characters
    let texts = vec![
        "Hello 世界",      // Latin + CJK
        "مرحبا",           // Arabic (RTL)
        "Привет",          // Cyrillic
        "Hello! 🌍",       // Emoji
    ];

    for text in texts {
        let shaped = engine.shape_text(text, 16.0);

        // Should successfully shape all text
        assert!(shaped.glyphs.len() > 0, "Should shape {}", text);
    }
}

#[test]
fn test_glyph_positions() {
    let mut engine = TextEngine::new();

    let shaped = engine.shape_text("abc", 16.0);

    // Glyphs should be positioned sequentially in LTR text
    assert!(shaped.glyphs.len() >= 3);

    // Each glyph should be to the right of the previous one
    for i in 1..shaped.glyphs.len() {
        assert!(
            shaped.glyphs[i].x_offset >= shaped.glyphs[i - 1].x_offset,
            "Glyphs should be in order"
        );
    }
}
