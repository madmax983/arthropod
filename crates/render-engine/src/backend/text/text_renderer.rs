//! Text renderer for generating GPU instances from shaped text

use super::glyph_atlas::GlyphAtlas;
use text_engine::{ShapedText, TextEngine};

/// GPU glyph instance data
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlyphInstance {
    pub pos: [f32; 2],        // Position (x, y)
    pub size: [f32; 2],       // Size (width, height)
    pub color: [f32; 4],      // Color (r, g, b, a)
    pub tex_coords: [f32; 4], // Texture coords (u0, v0, u1, v1)
}

/// Text renderer manages glyph atlas and instance generation
pub struct TextRenderer {
    atlas: GlyphAtlas,
    text_engine: TextEngine,
}

impl TextRenderer {
    /// Create a new text renderer with system fonts
    pub fn new() -> Self {
        Self {
            atlas: GlyphAtlas::new(1024, 1024),
            text_engine: TextEngine::new(),
        }
    }

    /// Generate glyph instances for shaped text
    pub fn generate_instances(
        &mut self,
        shaped: &ShapedText,
        position: glam::Vec2,
        color: glam::Vec4,
    ) -> Vec<GlyphInstance> {
        let mut instances = Vec::with_capacity(shaped.glyphs.len());

        for glyph in &shaped.glyphs {
            // Get texture coordinates from atlas using cosmic-text cache key
            let coords = self.atlas.get_or_rasterize(
                glyph.cache_key,
                self.text_engine.font_system(),
            );

            // Calculate glyph position
            let glyph_x = position.x + glyph.x_offset;
            let glyph_y = position.y + glyph.y_offset;

            // Use glyph metrics for size
            let glyph_width = glyph.x_advance;
            let glyph_height = 16.0; // Approximate from font size (TODO: get from cache_key)

            instances.push(GlyphInstance {
                pos: [glyph_x, glyph_y],
                size: [glyph_width, glyph_height],
                color: color.to_array(),
                tex_coords: [coords.u0, coords.v0, coords.u1, coords.v1],
            });
        }

        instances
    }

    /// Get the glyph atlas
    pub fn atlas(&self) -> &GlyphAtlas {
        &self.atlas
    }

    /// Get mutable reference to atlas
    pub fn atlas_mut(&mut self) -> &mut GlyphAtlas {
        &mut self.atlas
    }

    /// Get mutable reference to the text engine for shaping
    pub fn text_engine_mut(&mut self) -> &mut TextEngine {
        &mut self.text_engine
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_renderer_creation() {
        let renderer = TextRenderer::new();
        assert!(renderer.atlas().texture_data().len() > 0);
    }
}
