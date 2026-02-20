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
//!
//! fn main() -> Result<(), AppError> {
//!     App::run("Hello Arthropod", 400, 300, |ctx| {
//!         // Create reactive state
//!         let count = ctx.signal(0);
//!         let (read_count, write_count) = count.split();
//!
//!         // Create derived state (auto-updates when count changes)
//!         let count_text = Computed::new(ctx.runtime().clone(), move || {
//!             format!("Count: {}", read_count.get())
//!         });
//!
//!         col!(
//!             [
//!                 txt!("Hello, World!", size: 24.0),
//!                 // Note: txt! macro currently supports static strings and signals.
//!                 // For computed values, use the builder pattern:
//!                 Text::computed(count_text).size(18.0),
//!                 btn!("Increment", primary, on_click: move || {
//!                     write_count.update(|c| *c += 1);
//!                 }),
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
//! Arthropod uses a hybrid architecture combining a retained-mode scene graph with an ECS runtime:
//!
//! - **Scene Graph**: A [`render_engine::Scene`] manages the visual hierarchy (nodes, transforms, bounds) optimized for rendering.
//! - **ECS Runtime**: [`bevy_ecs`] manages cross-cutting concerns like input handling, animation, and reactive state updates.
//! - **Widget Layer**: The `widget-core` crate provides a high-level, declarative API that builds the scene graph.
//!
//! ## The Integration Loop
//!
//! 1. **Build Phase**: Widgets build a temporary scene graph in a [`WidgetContext`] (re-exported in prelude).
//! 2. **Integration**: [`App::integrate_widgets`] bridges this state into the ECS, creating entities with components like `Clickable` or `ReactiveColor`.
//! 3. **Update Loop**: The application loop processes events, updates signals via `flux-state`, and triggers re-renders.
//!
//! All resources are managed automatically by the [`App`] struct.

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
