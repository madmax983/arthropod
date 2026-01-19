//! Rendering engine for Arthropod GUI framework.
//!
//! Provides a retained-mode scene graph with wgpu backend.

pub mod backend;
pub mod node;
mod scene;

#[cfg(test)]
mod math_migration_tests;

// Re-export glam types for math operations
pub use glam::{Affine2, Vec2, Vec3, Vec4};

// Type aliases for transitional API (will become newtype wrappers later)
/// RGBA color represented as a 4-component vector (r, g, b, a).
/// Currently uses glam::Vec4 for SIMD performance.
pub type GlamColor = Vec4;

/// 2D affine transformation matrix.
/// Currently uses glam::Affine2 for SIMD performance.
pub type GlamTransform = Affine2;

pub use node::*;
pub use scene::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("No suitable graphics adapter found")]
    NoAdapter,

    #[error("wgpu error: {0}")]
    Wgpu(#[from] wgpu::RequestDeviceError),

    #[error("Surface error: {0}")]
    Surface(#[from] wgpu::SurfaceError),

    #[error("Surface creation error: {0}")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),

    #[error("Window handle error: {0}")]
    WindowHandle(#[from] raw_window_handle::HandleError),
}
