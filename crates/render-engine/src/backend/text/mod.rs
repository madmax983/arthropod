//! GPU text rendering backend
//!
//! Implements glyph atlas texture packing and GPU text rendering.

pub mod glyph_atlas;
pub mod text_renderer;

pub use glyph_atlas::{GlyphAtlas, TexCoords};
pub use text_renderer::TextRenderer;
