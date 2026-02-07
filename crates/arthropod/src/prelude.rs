//! Arthropod prelude - common imports for Arthropod applications
//!
//! This module re-exports the most commonly used types and traits.
//! Import with `use arthropod::prelude::*;` to get started quickly.

// Standard library time
pub use std::time::{Duration, Instant};

// App builder and context
pub use crate::app::{App, AppContext, AppError};
pub use crate::event_dispatcher::{DispatchResult, EventDispatcher};

// ECS integration
pub use arthropod_ecs::{
    FrameworkContext, MainThreadSignal, ReactiveColor, ReactiveOpacity, ReactiveText,
    ReactiveTransform, Renderable, SceneNodeRef,
};

// Accessibility
pub use a11y_engine::{
    A11yAction, A11yId, A11yNode, A11yRelations, A11yState, A11yTree, AccessibleName, CheckedState,
    Role,
};

// Reactive state
pub use flux_state::{Computed, Effect, Runtime, Signal, WriteSignal};

// Rendering
pub use render_engine::{
    Color, NodeContent, NodeId, Scene, SceneNode, Transform2D, Vec2,
    backend::{RenderBackend, WgpuBackend},
};

// Theme engine
pub use theme_engine::{DesignTokens, SystemTheme};

// Platform - Core types
pub use plat_core::{
    Application, ControlFlow, Event, EventLoop, Rect, Size, Window, WindowConfig, WindowEvent,
    WindowId,
};

// Platform - Input types
pub use plat_core::{ElementState, Key, MouseButton};

// Platform - Materials
pub use plat_core::{BackdropMaterial, HasBackdropMaterial};

// Widget system - Core widgets
pub use widget_core::{
    Button, Card, Center, Checkbox, Column, Container, Divider, Form, Grid, List, Padding, Row,
    Spacer, Stack, Text, TextInput, Widget, WidgetContext,
};

// Widget system - Macros
pub use widget_core::{btn, col, form, input, row, txt};

// bevy_ecs core types
pub use bevy_ecs::prelude::*;
