//! Text Engine - Font loading, shaping, and text layout
//!
//! Uses rustybuzz for text shaping and swash for font rasterization.
//! Provides glyph runs for rendering.

pub mod font_manager;
pub mod shaper;
pub mod glyph_cache;

pub use font_manager::FontManager;
pub use shaper::TextShaper;

/// Text engine combining font management and shaping
pub struct TextEngine {
    font_manager: FontManager,
    shaper: TextShaper,
}

/// A shaped glyph with position and metrics
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    pub glyph_id: u16,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
    pub y_advance: f32,
    pub cluster: u32,
}

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
    /// Create a new text engine
    pub fn new() -> Self {
        let font_manager = FontManager::new();
        let shaper = TextShaper::new();

        Self {
            font_manager,
            shaper,
        }
    }

    /// Shape text with the default font
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

        // Get default font
        let font_data = self.font_manager.default_font();

        // Shape text
        self.shaper.shape(text, font_data, font_size)
    }

    /// Get the font manager
    pub fn font_manager(&mut self) -> &mut FontManager {
        &mut self.font_manager
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
}
