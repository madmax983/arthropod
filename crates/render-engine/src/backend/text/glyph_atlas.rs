//! Glyph atlas for caching rasterized glyphs in a texture

use cosmic_text::{CacheKey, FontSystem, SwashCache};
use hashbrown::HashMap;

/// Texture coordinates in atlas (0.0-1.0 normalized) plus pixel dimensions and placement
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TexCoords {
    /// Left texture coordinate (0.0 to 1.0).
    pub u0: f32,
    /// Top texture coordinate (0.0 to 1.0).
    pub v0: f32,
    /// Right texture coordinate (0.0 to 1.0).
    pub u1: f32,
    /// Bottom texture coordinate (0.0 to 1.0).
    pub v1: f32,
    /// Rasterized glyph width in pixels
    pub pixel_width: u32,
    /// Rasterized glyph height in pixels
    pub pixel_height: u32,
    /// Horizontal offset from glyph origin to left edge of rasterized image
    pub placement_left: i32,
    /// Vertical offset from glyph origin to top edge of rasterized image
    pub placement_top: i32,
}

/// Cached glyph entry
struct CachedGlyph {
    coords: TexCoords,
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
        let size = (width as usize)
            .checked_mul(height as usize)
            .expect("Glyph atlas dimensions too large");
        let texture_data = vec![0u8; size];

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
        self.cache.insert(cache_key, CachedGlyph { coords });

        coords
    }

    /// Rasterize a glyph and pack it into the atlas
    fn rasterize_glyph(&mut self, cache_key: CacheKey, font_system: &mut FontSystem) -> TexCoords {
        use cosmic_text::SwashContent;

        // Rasterize using cosmic-text's SwashCache
        // We extract the data we need immediately to avoid borrow conflicts
        let (glyph_width, glyph_height, place_left, place_top, alpha_data) =
            match self.swash_cache.get_image(font_system, cache_key) {
                Some(img) => {
                    let w = img.placement.width;
                    let h = img.placement.height;
                    let pl = img.placement.left;
                    let pt = img.placement.top;
                    // Extract alpha channel based on content format
                    let alpha = match img.content {
                        SwashContent::Mask => {
                            // Already 1 byte per pixel (alpha mask)
                            img.data.clone()
                        }
                        SwashContent::SubpixelMask => {
                            // 4 bytes per pixel — R/G/B hold per-channel coverage.
                            // Average the RGB channels for a grayscale alpha mask.
                            img.data
                                .chunks_exact(4)
                                .map(|rgba| {
                                    let avg =
                                        (rgba[0] as u16 + rgba[1] as u16 + rgba[2] as u16) / 3;
                                    avg as u8
                                })
                                .collect()
                        }
                        SwashContent::Color => {
                            // 4 bytes per pixel (RGBA bitmap) — extract alpha channel
                            img.data.chunks_exact(4).map(|rgba| rgba[3]).collect()
                        }
                    };
                    (w, h, pl, pt, alpha)
                }
                None => {
                    // Return valid small coords for missing glyphs (1x1 pixel)
                    let u0 = self.current_x as f32 / self.width as f32;
                    let v0 = self.current_y as f32 / self.height as f32;
                    let u1 = (self.current_x + 1) as f32 / self.width as f32;
                    let v1 = (self.current_y + 1) as f32 / self.height as f32;
                    self.current_x += 1;
                    return TexCoords {
                        u0,
                        v0,
                        u1,
                        v1,
                        pixel_width: 1,
                        pixel_height: 1,
                        placement_left: 0,
                        placement_top: 0,
                    };
                }
            };

        // Check if we need to move to next row
        if self.current_x + glyph_width > self.width {
            self.current_x = 0;
            self.current_y += self.row_height;
            self.row_height = 0;
        }

        // Check if atlas is full
        if self.current_y + glyph_height > self.height {
            tracing::warn!(
                width = self.width,
                height = self.height,
                glyph_count = self.cache.len(),
                "Glyph atlas full, resetting cache"
            );
            self.reset();
        }

        // Calculate texture coordinates
        let x = self.current_x;
        let y = self.current_y;

        let u0 = x as f32 / self.width as f32;
        let v0 = y as f32 / self.height as f32;
        let u1 = (x + glyph_width) as f32 / self.width as f32;
        let v1 = (y + glyph_height) as f32 / self.height as f32;

        // Copy alpha data to texture (R8 format)
        for row in 0..glyph_height {
            for col in 0..glyph_width {
                let src_idx = (row * glyph_width + col) as usize;
                let dst_x = x + col;
                let dst_y = y + row;
                let dst_idx = (dst_y * self.width + dst_x) as usize;

                if dst_idx < self.texture_data.len() && src_idx < alpha_data.len() {
                    self.texture_data[dst_idx] = alpha_data[src_idx];
                }
            }
        }

        // Update packing state
        self.current_x += glyph_width;
        self.row_height = self.row_height.max(glyph_height);

        TexCoords {
            u0,
            v0,
            u1,
            v1,
            pixel_width: glyph_width,
            pixel_height: glyph_height,
            placement_left: place_left,
            placement_top: place_top,
        }
    }

    /// Get the texture data
    pub fn texture_data(&self) -> &[u8] {
        &self.texture_data
    }

    /// Reset the atlas, clearing all cached glyphs
    ///
    /// This is called when the atlas is full and we need to start fresh.
    /// All previously rasterized glyphs will need to be re-rasterized.
    pub fn reset(&mut self) {
        self.cache.clear();
        self.current_x = 0;
        self.current_y = 0;
        self.row_height = 0;
        // Clear texture data to avoid visual artifacts
        self.texture_data.fill(0);
    }

    /// Get the number of cached glyphs
    pub fn glyph_count(&self) -> usize {
        self.cache.len()
    }

    /// Check if the atlas is getting full (>80% used in Y direction)
    pub fn is_near_full(&self) -> bool {
        self.current_y > (self.height * 80 / 100)
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
            pixel_width: 10,
            pixel_height: 16,
            placement_left: 1,
            placement_top: 14,
        };

        assert!(coords.u0 >= 0.0 && coords.u0 <= 1.0);
        assert!(coords.v0 >= 0.0 && coords.v0 <= 1.0);
        assert!(coords.u1 >= 0.0 && coords.u1 <= 1.0);
        assert!(coords.v1 >= 0.0 && coords.v1 <= 1.0);
        assert_eq!(coords.pixel_width, 10);
        assert_eq!(coords.pixel_height, 16);
    }

    #[test]
    fn test_atlas_reset() {
        let mut atlas = GlyphAtlas::new(256, 256);

        // Simulate some usage by setting state
        atlas.current_x = 100;
        atlas.current_y = 50;
        atlas.row_height = 20;
        atlas.texture_data[0] = 255;
        atlas.texture_data[1000] = 128;

        // Reset should clear everything
        atlas.reset();

        assert_eq!(atlas.current_x, 0);
        assert_eq!(atlas.current_y, 0);
        assert_eq!(atlas.row_height, 0);
        assert_eq!(atlas.cache.len(), 0);
        assert_eq!(atlas.texture_data[0], 0);
        assert_eq!(atlas.texture_data[1000], 0);
    }

    #[test]
    fn test_glyph_count() {
        let atlas = GlyphAtlas::new(256, 256);
        assert_eq!(atlas.glyph_count(), 0);
    }

    #[test]
    fn test_is_near_full() {
        let mut atlas = GlyphAtlas::new(100, 100);

        // At 0%, not near full
        assert!(!atlas.is_near_full());

        // At 50%, not near full
        atlas.current_y = 50;
        assert!(!atlas.is_near_full());

        // At 80%, still not near full (boundary)
        atlas.current_y = 80;
        assert!(!atlas.is_near_full());

        // At 81%, near full
        atlas.current_y = 81;
        assert!(atlas.is_near_full());
    }

    #[test]
    fn test_cached_glyph_dimensions() {
        use text_engine::TextEngine;
        let mut atlas = GlyphAtlas::new(512, 512);
        let mut engine = TextEngine::new();

        // Shape some text to get a valid cache key
        let shaped = engine.shape_text("A", 16.0);

        // Ensure we have at least one glyph
        if shaped.glyphs.is_empty() {
            return;
        }

        let glyph = &shaped.glyphs[0];

        let coords = atlas.get_or_rasterize(glyph.cache_key, engine.font_system());

        // Pixel dimensions should be available via TexCoords
        assert!(
            coords.pixel_width > 0 || coords.pixel_height > 0,
            "At least one glyph dimension should be > 0"
        );
    }
}
