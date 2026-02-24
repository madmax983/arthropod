//! Rendering engine for Arthropod GUI framework.
//!
//! Provides a retained-mode scene graph with wgpu backend.
//!
//! # Architecture
//!
//! The rendering engine is built around a **Scene Graph** ([`Scene`]) which manages the visual
//! hierarchy of the application.
//!
//! - **Retained Mode**: The scene graph persists across frames. You modify it (add/remove/update nodes),
//!   and the engine handles rendering the current state.
//! - **Flattened Hierarchy**: Nodes are stored in a flat `HashMap`, with relationships managed via IDs.
//!   This enables O(1) lookups and updates.
//! - **Z-Ordering**: The engine enforces a deterministic Z-order based on the order of children in the
//!   parent's list. The last child added is drawn on top (Painter's Algorithm).
//! - **Coordinate System**: Uses a top-left origin (0,0), with Y increasing downwards.
//!
//! # Integration
//!
//! The [`Scene`] is designed to be used as a Resource in an ECS (Entity Component System) environment.
//!
//! ```
//! use render_engine::{Scene, SceneNode, NodeContent, Color, Transform2D};
//! use style_engine::VisualStyle;
//! use plat_core::Rect;
//!
//! // Create a scene
//! let mut scene = Scene::new();
//! let root = scene.root();
//!
//! // Create a styled node (Red Rectangle)
//! let style = VisualStyle::new().solid_fill(Color::RED.as_vec4());
//! let mut node = SceneNode::new(NodeContent::Styled {
//!     style: Box::new(style),
//! });
//!
//! // Set transform and bounds
//! node.transform = Transform2D::translate(100.0, 50.0);
//! node.bounds = Rect::new(0.0, 0.0, 200.0, 100.0);
//!
//! // Add to scene
//! scene.add_node(root, node);
//! ```

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

// Re-export style-engine types for convenient API
pub use style_engine::{
    BlendMode, ColorStop, CornerRadii, Effect, Paint, StrokeStyle, TextContent, VisualStyle,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("No suitable graphics adapter found")]
    NoAdapter,

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("wgpu error: {0}")]
    Wgpu(#[from] wgpu::RequestDeviceError),

    #[error("Surface error: {0}")]
    Surface(#[from] wgpu::SurfaceError),

    #[error("Surface creation error: {0}")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),

    #[error("Window handle error: {0}")]
    WindowHandle(#[from] raw_window_handle::HandleError),
}
