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
use std::sync::{Arc, Mutex, OnceLock};
pub use web_loader::{
    load_font_source, load_font_url, versioned_cache_key, FontCache, FontSource, WebFontLoadError,
};

fn global_font_registry() -> &'static Mutex<Vec<Arc<Vec<u8>>>> {
    static REGISTRY: OnceLock<Mutex<Vec<Arc<Vec<u8>>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

fn register_global_font_bytes(bytes: Vec<u8>) -> bool {
    let mut registry = global_font_registry()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let arc_bytes = Arc::new(bytes);

    if registry.iter().any(|existing| **existing == *arc_bytes) {
        return false;
    }
    registry.push(arc_bytes);
    true
}

fn apply_global_fonts(font_system: &mut FontSystem, applied_count: &mut usize) -> usize {
    let registry = global_font_registry()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let start = *applied_count;
    if start >= registry.len() {
        return 0;
    }

    for bytes in registry.iter().skip(start) {
        font_system
            .db_mut()
            .load_font_source(cosmic_text::fontdb::Source::Binary(
                Arc::clone(bytes) as Arc<dyn AsRef<[u8]> + Sync + Send>
            ));
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

/// Represents a single character (or ligature) that has gone through HarfBuzz shaping.
///
/// The shaping process converts logical characters (like "fi") into physical raster
/// instructions (a single "fi" ligature glyph). This struct holds the offset instructions
/// so the GPU pipeline knows exactly where to place the texture quad.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ShapedGlyph {
    pub cache_key: CacheKey,
    pub glyph_id: u16,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
    pub y_advance: f32,
    pub cluster: u32,
}

// Re-export CacheKey for convenience
pub use cosmic_text::CacheKey;

/// The final artifact of the text shaping pipeline.
///
/// Contains an ordered list of `ShapedGlyph`s ready to be uploaded to the GPU,
/// alongside the computed `TextBounds` which determines the exact physical pixel
/// footprint of the text. This is critical for layout engines (like Yoga or Taffy)
/// to perform accurate text-wrapping.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ShapedText {
    pub glyphs: Vec<ShapedGlyph>,
    pub bounds: TextBounds,
}

/// The physical, pixel-aligned boundary of a block of text.
///
/// Unlike raw CSS layouts, text rendering often overhangs its logical bounding box
/// due to font ascenders/descenders (like the tail of a 'y') or italics leaning
/// outside the box. This struct represents the *visual* footprint.
#[derive(Debug, Clone, Copy, Default)]
#[allow(missing_docs)]
pub struct TextBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Typographic slant configurations.
///
/// Used to instruct the underlying font matcher (e.g., Fontconfig) to prefer
/// specific font faces within a font family. Note that if a font does not
/// contain a true `Italic` face, the system may synthesize an `Oblique` slant automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFontStyle {
    /// Standard upright letterforms.
    #[default]
    Normal,
    /// Cursive or stylized letterforms designed explicitly by the typographer.
    Italic,
    /// Mechanically slanted letterforms.
    Oblique,
}

/// Overrides for font face selection during the shaping process.
///
/// By default, the `TextEngine` will use the system's default UI font.
/// Passing this struct allows you to request specific weights or families.
/// If the requested family cannot be found, it gracefully falls back to the system default.
#[derive(Debug, Clone, Copy, Default)]
#[allow(missing_docs)]
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
    // ⚡ Bolt: Eliminate intermediate `Vec` allocation for layout runs.
    // Iterating twice over `buffer.layout_runs()` is cheap, allowing us to
    // compute total glyphs and reserve exact capacity without an extra heap allocation.
    let total_glyphs = buffer.layout_runs().map(|run| run.glyphs.len()).sum();
    let mut glyphs = Vec::with_capacity(total_glyphs);
    let mut max_width = 0.0f32;
    let mut max_height = 0.0f32;

    for run in buffer.layout_runs() {
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
    fn test_font_registry_poison_recovery() {
        use std::thread;

        // Clone the registry Arc so we can poison it in a thread
        let registry_arc = global_font_registry();

        // Spawn a thread to deliberately poison the lock
        let handle = thread::spawn(move || {
            let _guard = registry_arc.lock().unwrap();
            panic!("Intentional panic to poison the Mutex");
        });

        // Wait for the thread to finish and ignore the panic
        let _ = handle.join();

        // Verify the lock is indeed poisoned
        assert!(registry_arc.lock().is_err());

        // Call our public-facing method that touches the poisoned lock.
        // If it handles the poison gracefully, this will not panic.
        let bytes = vec![0x00, 0x01, 0x02];
        let added = register_global_font_bytes(bytes);

        // Should return true since the mock data is new and not in the registry yet
        assert!(added);

        // Let's also verify apply_global_fonts handles it correctly without panic.
        // Note: Creating a new TextEngine triggers apply_global_fonts
        let engine = TextEngine::new();
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

#[cfg(test)]
mod elenchus_extract_tests {
    use super::*;

    #[test]
    fn test_extract_shaped_text_accumulates_max_width() {
        let mut engine = TextEngine::new();
        // A string with multiple characters, each adding width
        let shaped = engine.shape_text("WW", 16.0);

        // A single character will have a certain width. "WW" should have ~twice the width.
        // We want to ensure that max_width = max_width.max(x_end) works correctly,
        // which means the total width is significantly greater than just the advance of one glyph,
        // catching the replace + with - or * in `glyph.x + glyph.w`.
        assert!(shaped.glyphs.len() == 2, "Should have 2 glyphs");
        let w0 = shaped.glyphs[0].x_advance;
        let w1 = shaped.glyphs[1].x_advance;
        // Total expected width is roughly x of second + w of second
        let x1 = shaped.glyphs[1].x_offset;
        let expected_total_width = x1 + w1;

        // If glyph.x + glyph.w was replaced by glyph.x - glyph.w, the max_width would be < x1.
        assert_eq!(shaped.bounds.width, expected_total_width);

        // Ensure the width is reasonably positive and the second glyph is placed after the first
        assert!(
            shaped.bounds.width > w0 * 1.5,
            "Width should be roughly 2 characters wide"
        );
    }
}
