//! Rendering backends.

mod wgpu_backend;
pub mod text;

pub use wgpu_backend::{RectInstance, WgpuBackend};
pub use text::{GlyphAtlas, TextRenderer, GlyphInstance, TexCoords};

use crate::Scene;

/// Trait for rendering backends.
pub trait RenderBackend {
    /// Render a scene.
    fn render(&mut self, scene: &Scene) -> Result<(), crate::RendererError>;

    /// Resize the rendering surface.
    fn resize(&mut self, width: u32, height: u32);

    /// Set the clear color.
    fn set_clear_color(&mut self, color: crate::Color);
}
