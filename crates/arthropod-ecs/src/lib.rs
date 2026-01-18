//! ECS integration layer for Arthropod GUI framework
//!
//! This crate provides Entity-Component-System (ECS) integration for Arthropod using bevy_ecs.
//! It implements a hybrid architecture where:
//!
//! - **Scene hierarchy** uses a custom tree (Arena/HashMap) for O(1) access and cache-friendly traversal
//! - **Cross-cutting systems** (rendering, animation, reactive updates) use ECS for bulk operations
//!
//! # Architecture
//!
//! ```text
//! Scene (HashMap<NodeId, Node>)  ←→  FrameworkContext (bevy_ecs::World)
//!        ↑                                    ↑
//!        └──── NodeId references ─────────────┘
//!
//! Systems query ECS → read/write Scene nodes via NodeId
//! ```
//!
//! # Performance Characteristics
//!
//! ## Tree Operations (Custom Scene)
//! - Layout: O(n) depth-first traversal
//! - Event bubbling: O(depth) parent traversal
//! - Find by ID: O(1) HashMap lookup
//!
//! ## Bulk Operations (ECS)
//! - Render all visible: O(n) archetype iteration
//! - Animate all: O(n) parallel component updates
//! - Query by component: O(n) cache-friendly
//! - Reactive updates: O(changed) only dirty entities
//!
//! # Usage
//!
//! ```no_run
//! use arthropod_ecs::{FrameworkContext, Renderable, ReactiveColor};
//! use render_engine::Scene;
//! use flux_state::Signal;
//!
//! # let runtime = flux_state::Runtime::new();
//! let mut scene = Scene::new();
//! let mut context = FrameworkContext::new();
//!
//! // Create scene node
//! let node_id = scene.root();
//!
//! // Spawn ECS entity linked to scene node
//! let color_signal = Signal::new(runtime, render_engine::Color::RED);
//! let (read_signal, _write_signal) = color_signal.split();
//! context.spawn(node_id)
//!     .insert(Renderable)
//!     .insert(ReactiveColor::new(read_signal));
//!
//! // Update reactive systems
//! context.update(&mut scene);
//!
//! // Render and get GPU instances
//! let instances = context.render(&scene);
//! ```

pub mod components;
pub mod context;
pub mod systems;

// Re-export main types for convenience
pub use components::{
    Hoverable, MainThreadSignal, ReactiveColor, ReactiveOpacity, ReactiveTransform, Renderable,
    SceneNodeRef,
};
pub use context::FrameworkContext;
pub use systems::{
    collect_renderables_system, update_reactive_colors_system, update_reactive_opacity_system,
    update_reactive_transforms_system, RenderCommands, SceneReadResource, SceneResource,
};
