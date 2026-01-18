//! Arthropod prelude - common imports for Arthropod applications
//!
//! This module re-exports the most commonly used types and traits.
//! Import with `use arthropod::prelude::*;` to get started quickly.

// App builder
pub use crate::app::{App, AppBuilder, AppError};

// ECS integration
pub use arthropod_ecs::{
    FrameworkContext, MainThreadSignal, ReactiveColor, ReactiveOpacity, ReactiveTransform,
    Renderable, SceneNodeRef,
};

// Accessibility
pub use a11y_engine::{
    A11yAction, A11yId, A11yNode, A11yRelations, A11yState, A11yTree, AccessibleName,
    CheckedState, Role,
};

// Reactive state
pub use flux_state::{Effect, Runtime, Signal, WriteSignal};

// Rendering
pub use render_engine::{
    backend::{RenderBackend, WgpuBackend},
    Color, NodeContent, NodeId, Scene, SceneNode, Transform2D, Vec2,
};

// Platform
pub use plat_core::{
    Application, ControlFlow, Event, EventLoop, Rect, Size, Window, WindowConfig, WindowEvent,
    WindowId,
};

// bevy_ecs core types
pub use bevy_ecs::prelude::*;
