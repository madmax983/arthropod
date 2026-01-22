//! Text Engine - Font loading, shaping, and text layout
//!
//! Uses cosmic-text for integrated text shaping and rasterization.
//! Provides glyph runs for rendering.

use cosmic_text::{Attrs, Buffer, CacheKeyFlags, FontSystem, Metrics, Shaping};

/// Text engine using cosmic-text
pub struct TextEngine {
    font_system: FontSystem,
    buffer: Buffer,
}

/// A shaped glyph with position and metrics
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    pub cache_key: CacheKey, // cosmic-text cache key for rasterization
    pub glyph_id: u16,       // kept for compatibility
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
    pub y_advance: f32,
    pub cluster: u32,
}

// Re-export CacheKey for convenience
pub use cosmic_text::CacheKey;

/// Shaped text result
#[derive(Debug, Clone)]
pub struct ShapedText {
    pub glyphs: Vec<ShapedGlyph>,
    pub bounds: TextBounds,
}

/// Text bounding box
#[derive(Debug, Clone, Copy, Default)]
pub struct TextBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl TextEngine {
    /// Create a new text engine with system fonts
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();

        // Create a buffer for shaping text
        let metrics = Metrics::new(16.0, 20.0);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_size(&mut font_system, None, None);

        Self {
            font_system,
            buffer,
        }
    }

    /// Shape text with the specified font size
    ///
    /// # Example
    ///
    /// ```no_run
    /// use text_engine::TextEngine;
    ///
    /// let mut engine = TextEngine::new();
    /// let shaped = engine.shape_text("Hello World", 16.0);
    ///
    /// println!("Glyphs: {}", shaped.glyphs.len());
    /// ```
    pub fn shape_text(&mut self, text: &str, font_size: f32) -> ShapedText {
        if text.is_empty() {
            return ShapedText {
                glyphs: Vec::new(),
                bounds: TextBounds::default(),
            };
        }

        // Update metrics for this font size
        // Line height uses font metrics: cosmic-text calculates line height from
        // the font's ascender + descender + line gap. We pass a slightly larger
        // value to ensure proper spacing, then use the actual run.line_height below.
        // Note: Metrics::new takes (font_size, line_height) where line_height is
        // the desired line spacing for multi-line text.
        let metrics = Metrics::new(font_size, font_size * 1.2);
        self.buffer.set_metrics(&mut self.font_system, metrics);

        // Set text and shape
        self.buffer
            .set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);

        // Extract glyphs from the shaped buffer
        let mut glyphs = Vec::new();
        let mut max_width = 0.0f32;
        let mut max_height = 0.0f32;

        for run in self.buffer.layout_runs() {
            // Use the actual line height from font metrics
            let run_height = run.line_height;

            for glyph in run.glyphs.iter() {
                let x_end = glyph.x + glyph.w;
                max_width = max_width.max(x_end);
                max_height = max_height.max(run_height);

                // Construct CacheKey from glyph properties
                let (cache_key, _x_bin, _y_bin) = CacheKey::new(
                    glyph.font_id,
                    glyph.glyph_id,
                    glyph.font_size,
                    (glyph.x_offset, glyph.y_offset),
                    CacheKeyFlags::empty(),
                );

                glyphs.push(ShapedGlyph {
                    cache_key,
                    glyph_id: glyph.glyph_id,
                    x_offset: glyph.x,
                    y_offset: glyph.y,
                    x_advance: glyph.w,
                    y_advance: 0.0,
                    cluster: glyph.start as u32,
                });
            }
        }

        ShapedText {
            glyphs,
            bounds: TextBounds {
                x: 0.0,
                y: 0.0,
                width: max_width,
                height: max_height,
            },
        }
    }

    /// Get access to the font system for advanced operations
    pub fn font_system(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_engine() {
        let engine = TextEngine::new();
        // Just verify it can be created
        drop(engine);
    }

    #[test]
    fn test_shape_text() {
        let mut engine = TextEngine::new();
        let shaped = engine.shape_text("Hello", 16.0);
        assert!(!shaped.glyphs.is_empty());
        assert!(shaped.bounds.width > 0.0);
    }

    #[test]
    fn test_shape_special_chars() {
        let mut engine = TextEngine::new();
        let shaped = engine.shape_text("test.com-123", 16.0);
        // Should have glyphs for period and hyphen
        assert!(shaped.glyphs.len() >= 12); // "test.com-123" = 12 characters
    }
}
