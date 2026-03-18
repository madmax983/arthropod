//! Text Engine - Font loading, shaping, and text layout
//!
//! Uses cosmic-text for integrated text shaping and rasterization.
//! Provides glyph runs for rendering.

mod web_loader;

use cosmic_text::{
    Attrs, Buffer, CacheKeyFlags, Family, FontSystem, Metrics, Shaping, Style as CosmicStyle,
    Weight,
};
use std::cell::RefCell;
use std::sync::{Mutex, OnceLock};
pub use web_loader::{
    load_font_source, load_font_url, versioned_cache_key, FontCache, FontSource, WebFontLoadError,
};

fn global_font_registry() -> &'static Mutex<Vec<Vec<u8>>> {
    static REGISTRY: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

fn register_global_font_bytes(bytes: Vec<u8>) -> bool {
    let mut registry = global_font_registry()
        .lock()
        .expect("font registry lock poisoned");
    if registry.iter().any(|existing| existing == &bytes) {
        return false;
    }
    registry.push(bytes);
    true
}

fn apply_global_fonts(font_system: &mut FontSystem, applied_count: &mut usize) -> usize {
    let registry = global_font_registry()
        .lock()
        .expect("font registry lock poisoned");
    let start = *applied_count;
    if start >= registry.len() {
        return 0;
    }

    for bytes in registry.iter().skip(start) {
        font_system.db_mut().load_font_data(bytes.clone());
    }

    let added = registry.len().saturating_sub(start);
    *applied_count = registry.len();
    added
}

// Thread-local storage for parallel text shaping
thread_local! {
    static FONT_SYSTEM_POOL: RefCell<(FontSystem, Buffer, usize)> = RefCell::new({
        let mut font_system = FontSystem::new();
        let mut applied_global_fonts = 0usize;
        let _ = apply_global_fonts(&mut font_system, &mut applied_global_fonts);
        let metrics = Metrics::new(16.0, 20.0);
        let buffer = Buffer::new(&mut font_system, metrics);
        (font_system, buffer, applied_global_fonts)
    });
}

fn font_system_has_faces(font_system: &FontSystem) -> bool {
    font_system.db().faces().next().is_some()
}

/// Text engine using cosmic-text
pub struct TextEngine {
    font_system: FontSystem,
    buffer: Buffer,
    applied_global_fonts: usize,
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

/// Font style override for text shaping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

/// Optional text shaping overrides used to select a specific font face.
#[derive(Debug, Clone, Copy, Default)]
pub struct TextShapeOptions<'a> {
    pub family: Option<&'a str>,
    pub weight: Option<u16>,
    pub style: TextFontStyle,
}

fn attrs_from_options(options: TextShapeOptions<'_>) -> Attrs<'_> {
    let mut attrs = Attrs::new();

    if let Some(family) = options
        .family
        .map(str::trim)
        .filter(|family| !family.is_empty())
    {
        attrs = attrs.family(Family::Name(family));
    }

    if let Some(weight) = options.weight {
        attrs = attrs.weight(Weight(weight.clamp(1, 1000)));
    }

    let style = match options.style {
        TextFontStyle::Normal => CosmicStyle::Normal,
        TextFontStyle::Italic => CosmicStyle::Italic,
        TextFontStyle::Oblique => CosmicStyle::Oblique,
    };
    attrs.style(style)
}

impl TextEngine {
    /// Create a new text engine with system fonts
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();
        let mut applied_global_fonts = 0usize;
        let _ = apply_global_fonts(&mut font_system, &mut applied_global_fonts);

        // Create a buffer for shaping text
        let metrics = Metrics::new(16.0, 20.0);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_size(&mut font_system, None, None);

        Self {
            font_system,
            buffer,
            applied_global_fonts,
        }
    }

    #[cfg(test)]
    fn new_with_font_system(mut font_system: FontSystem) -> Self {
        let mut applied_global_fonts = 0usize;
        let _ = apply_global_fonts(&mut font_system, &mut applied_global_fonts);
        let metrics = Metrics::new(16.0, 20.0);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_size(&mut font_system, None, None);
        Self {
            font_system,
            buffer,
            applied_global_fonts,
        }
    }

    fn sync_global_fonts(&mut self) {
        let _ = apply_global_fonts(&mut self.font_system, &mut self.applied_global_fonts);
    }

    /// Register font bytes for shaping in this engine and all thread-local shaping pools.
    ///
    /// Returns the number of newly visible font faces in this engine's font database.
    pub fn register_font_bytes(&mut self, bytes: Vec<u8>) -> usize {
        let before = self.font_system.db().faces().count();
        let _ = register_global_font_bytes(bytes);
        self.sync_global_fonts();
        let after = self.font_system.db().faces().count();
        after.saturating_sub(before)
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
        self.shape_text_with_options(text, font_size, TextShapeOptions::default())
    }

    /// Shape text while honoring optional font family/weight/style overrides.
    pub fn shape_text_with_options(
        &mut self,
        text: &str,
        font_size: f32,
        options: TextShapeOptions<'_>,
    ) -> ShapedText {
        self.sync_global_fonts();

        if text.is_empty() {
            return ShapedText {
                glyphs: Vec::new(),
                bounds: TextBounds::default(),
            };
        }

        // Web targets may run without discoverable system fonts.
        // Avoid panics in cosmic-text fallback resolution by returning an empty shape result.
        if !font_system_has_faces(&self.font_system) {
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
        let attrs = attrs_from_options(options);
        self.buffer
            .set_text(&mut self.font_system, text, attrs, Shaping::Advanced);

        // Extract glyphs from the shaped buffer
        extract_shaped_text_from_buffer(&self.buffer)
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

fn extract_shaped_text_from_buffer(buffer: &Buffer) -> ShapedText {
    let runs: Vec<_> = buffer.layout_runs().collect();
    let total_glyphs = runs.iter().map(|run| run.glyphs.len()).sum();
    let mut glyphs = Vec::with_capacity(total_glyphs);
    let mut max_width = 0.0f32;
    let mut max_height = 0.0f32;

    for run in runs {
        let run_height = run.line_height;

        for glyph in run.glyphs.iter() {
            let x_end = glyph.x + glyph.w;
            max_width = max_width.max(x_end);
            max_height = max_height.max(run_height);

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

/// Shape text using thread-local FontSystem (safe for parallel execution)
///
/// This function uses thread-local storage to maintain per-thread FontSystem and Buffer
/// instances, allowing safe parallel text shaping via rayon without data races.
///
/// # Arguments
///
/// * `text` - The text to shape
/// * `font_size` - Font size in pixels
///
/// # Returns
///
/// A `ShapedText` containing glyphs and bounds
///
/// # Example
///
/// ```
/// use text_engine::shape_text_parallel;
/// use rayon::prelude::*;
///
/// let texts = vec!["Hello", "World", "!"];
/// let shaped: Vec<_> = texts.par_iter()
///     .map(|text| shape_text_parallel(text, 16.0))
///     .collect();
/// ```
pub fn shape_text_parallel(text: &str, font_size: f32) -> ShapedText {
    shape_text_parallel_with_options(text, font_size, TextShapeOptions::default())
}

/// Shape text in parallel-safe context while honoring optional font overrides.
pub fn shape_text_parallel_with_options(
    text: &str,
    font_size: f32,
    options: TextShapeOptions<'_>,
) -> ShapedText {
    if text.is_empty() {
        return ShapedText {
            glyphs: Vec::new(),
            bounds: TextBounds::default(),
        };
    }

    FONT_SYSTEM_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        let (font_system, buffer, applied_global_fonts) = &mut *pool;

        let _ = apply_global_fonts(font_system, applied_global_fonts);

        if !font_system_has_faces(font_system) {
            return ShapedText {
                glyphs: Vec::new(),
                bounds: TextBounds::default(),
            };
        }

        // Update metrics for this font size
        let metrics = Metrics::new(font_size, font_size * 1.2);
        buffer.set_metrics(font_system, metrics);

        // Set text and shape
        let attrs = attrs_from_options(options);
        buffer.set_text(font_system, text, attrs, Shaping::Advanced);

        // Extract glyphs from the shaped buffer
        extract_shaped_text_from_buffer(buffer)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const INTER_REGULAR_TTF: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");

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
    fn test_attrs_from_options_maps_family_weight_and_style() {
        let attrs = attrs_from_options(TextShapeOptions {
            family: Some("Inter"),
            weight: Some(700),
            style: TextFontStyle::Italic,
        });
        assert_eq!(attrs.family, Family::Name("Inter"));
        assert_eq!(attrs.weight, Weight(700));
        assert_eq!(attrs.style, CosmicStyle::Italic);
    }

    #[test]
    fn test_shape_special_chars() {
        let mut engine = TextEngine::new();
        let shaped = engine.shape_text("test.com-123", 16.0);
        // Should have glyphs for period and hyphen
        assert!(shaped.glyphs.len() >= 12); // "test.com-123" = 12 characters
    }

    #[test]
    fn test_parallel_shaping_single_thread() {
        let shaped = shape_text_parallel("Hello", 16.0);
        assert!(!shaped.glyphs.is_empty());
        assert!(shaped.bounds.width > 0.0);
    }

    #[test]
    fn test_parallel_shaping_matches_sequential() {
        use rayon::prelude::*;

        let texts = vec!["Hello", "World", "Test", "Parallel"];

        // Shape sequentially
        let mut engine = TextEngine::new();
        let sequential: Vec<_> = texts
            .iter()
            .map(|text| engine.shape_text(text, 16.0))
            .collect();

        // Shape in parallel
        let parallel: Vec<_> = texts
            .par_iter()
            .map(|text| shape_text_parallel(text, 16.0))
            .collect();

        // Verify results match
        for (seq, par) in sequential.iter().zip(parallel.iter()) {
            assert_eq!(seq.glyphs.len(), par.glyphs.len());
            assert!((seq.bounds.width - par.bounds.width).abs() < 0.1);
            assert!((seq.bounds.height - par.bounds.height).abs() < 0.1);

            // Verify glyphs are equivalent (cache keys should match)
            for (seq_glyph, par_glyph) in seq.glyphs.iter().zip(par.glyphs.iter()) {
                assert_eq!(seq_glyph.cache_key, par_glyph.cache_key);
                assert_eq!(seq_glyph.glyph_id, par_glyph.glyph_id);
                assert!((seq_glyph.x_offset - par_glyph.x_offset).abs() < 0.1);
                assert!((seq_glyph.y_offset - par_glyph.y_offset).abs() < 0.1);
            }
        }
    }

    #[test]
    fn test_parallel_shaping_with_empty_text() {
        let shaped = shape_text_parallel("", 16.0);
        assert!(shaped.glyphs.is_empty());
        assert_eq!(shaped.bounds.width, 0.0);
    }

    #[test]
    // Skip on Linux/Windows where system fonts might leak or be unavoidable in CI environments
    #[cfg_attr(not(target_os = "macos"), ignore)]
    fn test_shape_text_without_available_fonts_returns_empty() {
        let db = cosmic_text::fontdb::Database::new();
        let font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), db);
        let mut engine = TextEngine::new_with_font_system(font_system);
        let shaped = engine.shape_text("Hello", 16.0);
        assert!(shaped.glyphs.is_empty());
        assert_eq!(shaped.bounds.width, 0.0);
        assert_eq!(shaped.bounds.height, 0.0);
    }

    #[test]
    fn test_register_font_bytes_unblocks_shaping_on_empty_db() {
        let db = cosmic_text::fontdb::Database::new();
        let font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), db);
        let mut engine = TextEngine::new_with_font_system(font_system);

        let before = engine.shape_text("Hello", 16.0);
        assert!(before.glyphs.is_empty());

        let added_faces = engine.register_font_bytes(INTER_REGULAR_TTF.to_vec());
        assert!(added_faces > 0);

        let after = engine.shape_text("Hello", 16.0);
        assert!(!after.glyphs.is_empty());
        assert!(after.bounds.width > 0.0);
    }

    #[test]
    fn test_shape_text_parallel_uses_runtime_registered_font() {
        let db = cosmic_text::fontdb::Database::new();
        let font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), db);
        let mut engine = TextEngine::new_with_font_system(font_system);
        let _ = engine.register_font_bytes(INTER_REGULAR_TTF.to_vec());

        let shaped = shape_text_parallel("Parallel Hello", 16.0);
        assert!(!shaped.glyphs.is_empty());
        assert!(shaped.bounds.width > 0.0);
    }
}
