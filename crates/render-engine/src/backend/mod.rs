//! Rendering backends.

#[cfg(target_os = "windows")]
pub mod composition_swap_chain;
pub mod text;
pub mod wgpu;

pub use crate::primitives::PrimitiveInstance;
pub use text::{GlyphAtlas, TexCoords, TextRenderer};
pub use wgpu::{PathCacheWarmupReport, TessellationCacheStats, WgpuBackend};
