//! ECS integration layer for Arthropod GUI framework
//!
//! This crate provides Entity-Component-System (ECS) integration for Arthropod using [`bevy_ecs`].
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
//! This example demonstrates how to set up the ECS environment, spawn an entity,
//! and link it to a scene node with reactive properties.
//!
//! ```
//! use arthropod_ecs::{FrameworkContext, Renderable, ReactiveColor};
//! use render_engine::{Scene, Color};
//! use flux_state::{Runtime, Signal};
//!
//! // 1. Create the runtime and context
//! let runtime = Runtime::new();
//! let mut context = FrameworkContext::new();
//!
//! // 2. Access the Scene resource to get the root node
//! let root_id = {
//!     let scene = context.world().resource::<Scene>();
//!     scene.root()
//! };
//!
//! // 3. Create a reactive signal for color
//! let color_signal = Signal::new(runtime, Color::RED);
//! let (read_signal, _write_signal) = color_signal.split();
//!
//! // 4. Spawn an ECS entity linked to the scene node
//! // We attach `Renderable` (tag) and `ReactiveColor` (component)
//! context.spawn(root_id)
//!     .insert(Renderable)
//!     .insert(ReactiveColor::new(read_signal));
//!
//! // 5. Run the update loop
//! // This triggers systems that propagate signal changes to the Scene
//! context.update();
//!
//! // 6. Render
//! // The renderer traverses the Scene, which now has the updated color
//! let _instances = context.render();
//! ```

pub mod components;
pub(crate) mod context;
pub mod systems;

// Re-export main types for convenience
pub use components::{
    Hoverable, ReactiveColor, ReactiveComputedText, ReactiveOpacity, ReactiveText,
    ReactiveTransform, Renderable, SceneNodeRef,
};
pub use context::FrameworkContext;
pub use systems::{
    apply_a11y_bounds_system, apply_reactive_changes_system, collect_renderables_system,
    gather_a11y_bounds_system, gather_reactive_changes_system, update_all_reactive_system,
    A11yBoundsBuffer, ReactiveChangeBuffer, RenderCommands, TimeResource, TimelineDriver,
};

// Re-export accessibility types
pub use a11y_engine::{
    A11yAction, A11yId, A11yNode, A11yRelations, A11yState, A11yTree, AccessibleName, CheckedState,
    Role,
};
