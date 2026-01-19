//! Glyph rasterization and caching
//!
//! Placeholder for future glyph atlas implementation

/// Glyph cache (future implementation)
pub struct GlyphCache {
    // TODO: Implement texture atlas caching
}

impl GlyphCache {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self::new()
    }
}
