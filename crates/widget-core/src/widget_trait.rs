//! Core Widget trait

use crate::context::WidgetContext;
use render_engine::NodeId;

/// The Widget trait - implemented by all UI widgets
///
/// Widgets are declarative UI building blocks that:
/// 1. Build scene nodes (hierarchy)
/// 2. Spawn ECS entities (components)
/// 3. Configure layout (flexbox)
/// 4. Handle state updates (reactive)
pub trait Widget {
    /// Build this widget, returning the root scene node ID
    ///
    /// This creates:
    /// - Scene nodes for visual hierarchy
    /// - ECS entities with components
    /// - Layout nodes for flexbox
    fn build(&self, ctx: &mut WidgetContext) -> NodeId;
}

/// Trait for widgets that can contain children
pub trait ContainerWidget: Widget {
    /// Add a child widget
    fn add_child(&mut self, child: Box<dyn Widget>);
}
