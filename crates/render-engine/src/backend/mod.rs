//! Rendering backends.

#[cfg(target_os = "windows")]
pub mod composition_swap_chain;
pub mod text;
pub mod wgpu;

pub use text::{GlyphAtlas, GlyphInstance, TexCoords, TextRenderer};
pub use wgpu::{PrimitiveInstance, WgpuBackend};

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
