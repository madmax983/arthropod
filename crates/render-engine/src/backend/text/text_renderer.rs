//! Text renderer for generating GPU instances from shaped text

use super::glyph_atlas::GlyphAtlas;
use text_engine::ShapedText;

/// GPU glyph instance data
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlyphInstance {
    pub pos: [f32; 2],           // Position (x, y)
    pub size: [f32; 2],          // Size (width, height)
    pub color: [f32; 4],         // Color (r, g, b, a)
    pub tex_coords: [f32; 4],    // Texture coords (u0, v0, u1, v1)
}

/// Text renderer manages glyph atlas and instance generation
pub struct TextRenderer {
    atlas: GlyphAtlas,
    font_data: Vec<u8>,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new() -> Self {
        // Load default system font
        let font_data = Self::load_default_font();

        Self {
            atlas: GlyphAtlas::new(1024, 1024),
            font_data,
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
            // Get texture coordinates from atlas
            let coords = self.atlas.get_or_rasterize(
                glyph.glyph_id,
                16, // TODO: Use actual font size
                &self.font_data,
            );

            // Calculate glyph position
            let glyph_x = position.x + glyph.x_offset;
            let glyph_y = position.y + glyph.y_offset;

            // Estimate glyph size from advance (rough approximation)
            let glyph_width = glyph.x_advance;
            let glyph_height = 16.0; // Approximate from font size

            instances.push(GlyphInstance {
                pos: [glyph_x, glyph_y],
                size: [glyph_width, glyph_height],
                color: color.to_array(),
                tex_coords: [coords.u0, coords.v0, coords.u1, coords.v1],
            });
        }

        instances
    }

    /// Load default system font
    fn load_default_font() -> Vec<u8> {
        #[cfg(target_os = "windows")]
        {
            let font_paths = vec![
                r"C:\Windows\Fonts\segoeui.ttf",
                r"C:\Windows\Fonts\arial.ttf",
                r"C:\Windows\Fonts\verdana.ttf",
            ];

            for path in font_paths {
                if let Ok(data) = std::fs::read(path) {
                    return data;
                }
            }
        }

        // Return empty vec if no font found
        Vec::new()
    }

    /// Get the glyph atlas
    pub fn atlas(&self) -> &GlyphAtlas {
        &self.atlas
    }

    /// Get mutable reference to atlas
    pub fn atlas_mut(&mut self) -> &mut GlyphAtlas {
        &mut self.atlas
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
