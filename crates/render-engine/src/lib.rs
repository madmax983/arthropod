//! Rendering engine for Arthropod GUI framework.
//!
//! Provides a retained-mode scene graph with wgpu backend.

pub mod backend;
mod node;
mod scene;

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
