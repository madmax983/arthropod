//! Arthropod - Enterprise-grade cross-platform Rust GUI framework
//!
//! Arthropod is a high-performance GUI framework built on:
//! - **bevy_ecs**: Entity-Component-System for scalable architecture
//! - **wgpu**: Modern GPU-accelerated rendering
//! - **flux-state**: Fine-grained reactive state management
//!
//! # Overview
//!
//! Arthropod combines the ergonomics of modern declarative UI frameworks (like React or SwiftUI)
//! with the power of an Entity-Component-System (ECS) architecture. This "Hybrid ECS" approach
//! allows for:
//!
//! - **Zero-cost Abstractions**: Complex widgets compile down to simple scene nodes.
//! - **High Performance**: Layout and rendering are batched and optimized.
//! - **Extensibility**: You can hook into the ECS to add custom behaviors or systems.
//!
//! # Quick Start
//!
//! For widget-based applications, use the high-level [`App::run`] API:
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
//!                 // Text::computed allows using derived state
//!                 Text::computed(count_text).size(18.0),
//!                 // btn! macro with 'primary' style flag
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
//! Arthropod separates concerns into three distinct layers:
//!
//! 1.  **Widget Layer (`widget-core`)**: The high-level API where you define your UI. It uses the Facade pattern to hide complexity.
//! 2.  **Scene Graph (`render-engine`)**: A retained-mode tree structure that manages layout and rendering primitives.
//! 3.  **ECS Runtime (`arthropod-ecs`)**: The backbone that orchestrates the application lifecycle, handling events, updates, and resource management.
//!
//! ## The Integration Loop
//!
//! 1. **Build Phase**: Widgets build a temporary scene graph in a [`prelude::WidgetContext`] (re-exported in prelude).
//! 2. **Integration**: [`App::integrate_widgets`] bridges this state into the ECS, creating entities with components like `Clickable` or `ReactiveColor`.
//! 3. **Update Loop**: The application loop processes events, updates signals via `flux-state`, and triggers re-renders.
//!
//! All resources are managed automatically by the [`App`] struct.
//!
//! # Feature Flags
//!
//! - `nova`: Enables experimental features (Story Engine, Particles, Flux Radar).
//!   See [`experimental`] module for details.

pub mod app;
pub mod event_dispatcher;
/// Experimental features, including advanced rendering and new architectures.
pub mod experimental;
/// Core types and schemas for importing and representing Figma documents.
pub mod figma;
/// Code generation tools for baking imported Figma documents directly into Rust source files.
pub mod figma_codegen;
/// Execution bridge connecting imported Figma layouts to the `render_engine`.
pub mod figma_runtime;
pub mod layout;
pub mod prelude;
/// Interactive execution engine for evaluating Figma prototype triggers and transitions.
pub mod prototype_runtime;

// Re-export main types
pub use app::{App, AppError};
pub use event_dispatcher::{DispatchResult, EventDispatcher};
pub use figma::import_figma_document;
pub use figma_codegen::{generate_rust_module_from_json, write_rust_module_from_json};
pub use figma_runtime::FigmaRuntime;
pub use layout::{auto_layout, layout_widget_tree};

// Re-export core crates for convenience
pub use arthropod_ecs;
pub use flux_state;
pub use input_engine;
pub use render_engine;
pub use widget_core;
