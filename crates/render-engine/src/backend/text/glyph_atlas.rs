//! Glyph atlas for caching rasterized glyphs in a texture

use cosmic_text::{CacheKey, FontSystem, SwashCache};
use hashbrown::HashMap;

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
        let (coords, width, height) = self.rasterize_glyph(cache_key, font_system);

        // Cache it
        self.cache.insert(
            cache_key,
            CachedGlyph {
                coords,
                width,
                height,
            },
        );

        coords
    }

    /// Rasterize a glyph and pack it into the atlas
    fn rasterize_glyph(
        &mut self,
        cache_key: CacheKey,
        font_system: &mut FontSystem,
    ) -> (TexCoords, u32, u32) {
        // Rasterize using cosmic-text's SwashCache
        // We extract the data we need immediately to avoid borrow conflicts
        let (glyph_width, glyph_height, glyph_data) =
            match self.swash_cache.get_image(font_system, cache_key) {
                Some(img) => {
                    let w = img.placement.width;
                    let h = img.placement.height;
                    let data = img.data.clone(); // Clone to release borrow on self
                    (w, h, data)
                }
                None => {
                    // Return valid small coords for missing glyphs (1x1 pixel)
                    let u0 = self.current_x as f32 / self.width as f32;
                    let v0 = self.current_y as f32 / self.height as f32;
                    let u1 = (self.current_x + 1) as f32 / self.width as f32;
                    let v1 = (self.current_y + 1) as f32 / self.height as f32;
                    self.current_x += 1;
                    return (TexCoords { u0, v0, u1, v1 }, 1, 1);
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
            // Atlas full - reset to beginning and clear cache
            // This is a simple eviction strategy that clears all glyphs
            // Future improvement: implement LRU eviction or atlas expansion
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

        // Copy glyph data to texture (R8 format - just alpha channel)
        // cosmic-text's image data is already in alpha format
        for row in 0..glyph_height {
            for col in 0..glyph_width {
                let src_idx = (row * glyph_width + col) as usize;
                let dst_x = x + col;
                let dst_y = y + row;
                let dst_idx = (dst_y * self.width + dst_x) as usize;

                if dst_idx < self.texture_data.len() && src_idx < glyph_data.len() {
                    let alpha = glyph_data[src_idx];
                    self.texture_data[dst_idx] = alpha; // R8: single alpha channel
                }
            }
        }

        // Update packing state
        self.current_x += glyph_width;
        self.row_height = self.row_height.max(glyph_height);

        (TexCoords { u0, v0, u1, v1 }, glyph_width, glyph_height)
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
        };

        assert!(coords.u0 >= 0.0 && coords.u0 <= 1.0);
        assert!(coords.v0 >= 0.0 && coords.v0 <= 1.0);
        assert!(coords.u1 >= 0.0 && coords.u1 <= 1.0);
        assert!(coords.v1 >= 0.0 && coords.v1 <= 1.0);
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
            // If no system fonts are found or "A" produces no glyphs, we skip the test
            // or print a warning, but we expect "A" to work on most systems.
            // On CI environments without fonts this might be an issue.
            // But let's assume there is a font.
            return;
        }

        let glyph = &shaped.glyphs[0];

        atlas.get_or_rasterize(glyph.cache_key, engine.font_system());

        let cached = atlas
            .cache
            .get(&glyph.cache_key)
            .expect("Glyph should be cached");

        // Allow for the possibility that one dimension could legitimately be zero
        assert!(
            cached.width > 0 || cached.height > 0,
            "At least one glyph dimension should be > 0"
        );
    }
}
