//! Arthropod - Enterprise-grade cross-platform Rust GUI framework
//!
//! Arthropod is a high-performance GUI framework built on:
//! - **bevy_ecs**: Entity-Component-System for scalable architecture
//! - **wgpu**: Modern GPU-accelerated rendering
//! - **flux-state**: Fine-grained reactive state management
//!
//! # Quick Start
//!
//! For widget-based applications, use the high-level `App::run` API:
//!
//! ```no_run
//! use arthropod::prelude::*;
//! use widget_core::{txt, btn, col};
//!
//! fn main() -> Result<(), AppError> {
//!     App::run("Hello Arthropod", 400, 300, |_ctx| {
//!         col!(
//!             [
//!                 txt!("Hello, World!", size: 24.0),
//!                 btn!("Click Me", primary, on_click: || println!("Button clicked!")),
//!             ],
//!             gap: 20.0,
//!             padding: 20.0
//!         )
//!     })
//! }
//! ```
//!
//! For low-level control or headless testing:
//!
//! ```
//! use arthropod::prelude::*;
//! use render_engine::Scene;
//!
//! // Create headless app (no window/GPU)
//! let mut app = App::new_headless()
//!     .expect("Failed to create app");
//!
//! // Access resources via ECS world
//! let scene = app.world().resource::<Scene>();
//! println!("Root node: {:?}", scene.root());
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
#[cfg(feature = "nova")]
pub mod experimental;
pub mod layout;
pub mod prelude;

// Re-export main types
pub use app::{App, AppError};
pub use event_dispatcher::{DispatchResult, EventDispatcher};
pub use layout::{auto_layout, layout_widget_tree};
