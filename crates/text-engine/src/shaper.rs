//! Text shaping using rustybuzz

use crate::{ShapedGlyph, ShapedText, TextBounds};
use rustybuzz as rb;

/// Text shaper
pub struct TextShaper {
    // Shaper is stateless, using rustybuzz directly
}

impl TextShaper {
    /// Create a new text shaper
    pub fn new() -> Self {
        Self {}
    }

    /// Shape text with a given font
    pub fn shape(&mut self, text: &str, font_data: &[u8], font_size: f32) -> ShapedText {
        // Parse font
        let face = match rb::Face::from_slice(font_data, 0) {
            Some(face) => face,
            None => {
                // Return empty shaped text if font can't be parsed
                return ShapedText {
                    glyphs: Vec::new(),
                    bounds: TextBounds::default(),
                };
            }
        };

        // Create unicode buffer
        let mut buffer = rb::UnicodeBuffer::new();
        buffer.push_str(text);

        // Shape the text
        let output = rb::shape(&face, &[], buffer);

        // Extract glyph info and positions
        let glyph_infos = output.glyph_infos();
        let glyph_positions = output.glyph_positions();

        let mut glyphs = Vec::with_capacity(glyph_infos.len());
        let mut x = 0.0;
        let scale = font_size / face.units_per_em() as f32;

        for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
            let x_advance = pos.x_advance as f32 * scale;
            let y_advance = pos.y_advance as f32 * scale;
            let x_offset = pos.x_offset as f32 * scale;
            let y_offset = pos.y_offset as f32 * scale;

            glyphs.push(ShapedGlyph {
                glyph_id: info.glyph_id as u16,
                x_offset: x + x_offset,
                y_offset,
                x_advance,
                y_advance,
                cluster: info.cluster,
            });

            x += x_advance;
        }

        // Calculate bounds
        let width = x;
        let height = font_size * 1.2; // Approximate line height

        let bounds = TextBounds {
            x: 0.0,
            y: 0.0,
            width,
            height,
        };

        ShapedText { glyphs, bounds }
    }
}

impl Default for TextShaper {
    fn default() -> Self {
        Self::new()
    }
}
