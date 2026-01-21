//! Arthropod - Enterprise-grade cross-platform Rust GUI framework
//!
//! Arthropod is a high-performance GUI framework built on:
//! - **bevy_ecs**: Entity-Component-System for scalable architecture
//! - **wgpu**: Modern GPU-accelerated rendering
//! - **flux-state**: Fine-grained reactive state management
//!
//! # Quick Start
//!
//! ```
//! use arthropod::prelude::*;
//!
//! let config = WindowConfig {
//!     title: "My App".to_string(),
//!     size: Size { width: 800, height: 600 },
//!     ..Default::default()
//! };
//!
//! // For testing/headless mode
//! let mut app = AppBuilder::new()
//!     .with_window_config(config)
//!     .build_headless()
//!     .expect("Failed to create app");
//!
//! // Add your UI code here
//! // app.spawn(node_id).insert(Renderable);
//! ```
//!
//! # Architecture
//!
//! Arthropod uses a hybrid architecture:
//! - **Scene Graph**: Custom HashMap-based tree for hierarchical UI layout
//! - **ECS**: bevy_ecs for cross-cutting concerns (rendering, animation, reactive state)
//! - **Resources**: Scene, Runtime, WgpuBackend all live in ECS World
//!
//! All resources are managed automatically by the `App` - no manual lifetime management required.

pub mod app;
pub mod event_dispatcher;
pub mod layout;
pub mod prelude;

// Re-export main types
pub use app::{App, AppBuilder, AppError};
pub use event_dispatcher::{DispatchResult, EventDispatcher};
pub use layout::{auto_layout, layout_widget_tree};
