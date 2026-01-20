//! Glyph atlas for caching rasterized glyphs in a texture

use hashbrown::HashMap;
use cosmic_text::{CacheKey, FontSystem, SwashCache};

/// Texture coordinates in atlas (0.0-1.0 normalized)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TexCoords {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

/// Cached glyph entry
struct CachedGlyph {
    coords: TexCoords,
    #[allow(dead_code)]
    width: u32,
    #[allow(dead_code)]
    height: u32,
}

/// Glyph atlas for texture packing and caching
pub struct GlyphAtlas {
    width: u32,
    height: u32,
    cache: HashMap<CacheKey, CachedGlyph>,
    // Simple row packer
    current_x: u32,
    current_y: u32,
    row_height: u32,
    // Texture data (R8 - single alpha channel)
    texture_data: Vec<u8>,
    // cosmic-text's swash cache for rasterization
    swash_cache: SwashCache,
}

impl GlyphAtlas {
    /// Create a new glyph atlas
    pub fn new(width: u32, height: u32) -> Self {
        let texture_data = vec![0u8; (width * height) as usize];

        Self {
            width,
            height,
            cache: HashMap::new(),
            current_x: 0,
            current_y: 0,
            row_height: 0,
            texture_data,
            swash_cache: SwashCache::new(),
        }
    }

    /// Get texture coordinates for a glyph, rasterizing if not cached
    pub fn get_or_rasterize(
        &mut self,
        cache_key: CacheKey,
        font_system: &mut FontSystem,
    ) -> TexCoords {
        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.coords;
        }

        // Rasterize glyph using cosmic-text
        let coords = self.rasterize_glyph(cache_key, font_system);

        // Cache it
        self.cache.insert(
            cache_key,
            CachedGlyph {
                coords,
                width: 0, // TODO: store actual dimensions
                height: 0,
            },
        );

        coords
    }

    /// Rasterize a glyph and pack it into the atlas
    fn rasterize_glyph(&mut self, cache_key: CacheKey, font_system: &mut FontSystem) -> TexCoords {
        // Rasterize using cosmic-text's SwashCache
        let image = match self.swash_cache.get_image(font_system, cache_key) {
            Some(img) => img,
            None => {
                // Return valid small coords for missing glyphs (1x1 pixel)
                let u0 = self.current_x as f32 / self.width as f32;
                let v0 = self.current_y as f32 / self.height as f32;
                let u1 = (self.current_x + 1) as f32 / self.width as f32;
                let v1 = (self.current_y + 1) as f32 / self.height as f32;
                self.current_x += 1;
                return TexCoords { u0, v0, u1, v1 };
            }
        };

        let glyph_width = image.placement.width as u32;
        let glyph_height = image.placement.height as u32;

        // Check if we need to move to next row
        if self.current_x + glyph_width > self.width {
            self.current_x = 0;
            self.current_y += self.row_height;
            self.row_height = 0;
        }

        // Check if atlas is full
        if self.current_y + glyph_height > self.height {
            // Atlas full - return last position (will overlap)
            // TODO: Implement atlas expansion or eviction
        }

        // Calculate texture coordinates
        let x = self.current_x;
        let y = self.current_y;

        let u0 = x as f32 / self.width as f32;
        let v0 = y as f32 / self.height as f32;
        let u1 = (x + glyph_width) as f32 / self.width as f32;
        let v1 = (y + glyph_height) as f32 / self.height as f32;

        // Copy glyph data to texture (R8 format - just alpha channel)
        // cosmic-text's image data is already in alpha format
        for row in 0..glyph_height {
            for col in 0..glyph_width {
                let src_idx = (row * glyph_width + col) as usize;
                let dst_x = x + col;
                let dst_y = y + row;
                let dst_idx = (dst_y * self.width + dst_x) as usize;

                if dst_idx < self.texture_data.len() && src_idx < image.data.len() {
                    let alpha = image.data[src_idx];
                    self.texture_data[dst_idx] = alpha; // R8: single alpha channel
                }
            }
        }

        // Update packing state
        self.current_x += glyph_width;
        self.row_height = self.row_height.max(glyph_height);

        TexCoords { u0, v0, u1, v1 }
    }

    /// Get the texture data
    pub fn texture_data(&self) -> &[u8] {
        &self.texture_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_creation() {
        let atlas = GlyphAtlas::new(512, 512);
        assert_eq!(atlas.width, 512);
        assert_eq!(atlas.height, 512);
    }

    #[test]
    fn test_tex_coords_range() {
        let coords = TexCoords {
            u0: 0.0,
            v0: 0.0,
            u1: 0.5,
            v1: 0.5,
        };

        assert!(coords.u0 >= 0.0 && coords.u0 <= 1.0);
        assert!(coords.v0 >= 0.0 && coords.v0 <= 1.0);
        assert!(coords.u1 >= 0.0 && coords.u1 <= 1.0);
        assert!(coords.v1 >= 0.0 && coords.v1 <= 1.0);
    }
}
