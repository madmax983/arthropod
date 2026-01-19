//! Container widget - flexbox layout container

use crate::{Widget, WidgetContext};
use render_engine::{NodeId, NodeContent};
use layout_engine::{FlexStyle, FlexDirection};

/// Container widget for layout
///
/// Implements flexbox layout with row/column direction.
///
/// # Example
///
/// ```no_run
/// use widget_core::{Container, Text};
///
/// let widget = Container::column()
///     .gap(10.0)
///     .padding(16.0)
///     .child(Text::new("Hello"))
///     .child(Text::new("World"));
/// ```
pub struct Container {
    direction: FlexDirection,
    children: Vec<Box<dyn Widget>>,
    gap: f32,
    padding: f32,
}

impl Container {
    /// Create a row container
    pub fn row() -> Self {
        Self {
            direction: FlexDirection::Row,
            children: Vec::new(),
            gap: 0.0,
            padding: 0.0,
        }
    }

    /// Create a column container
    pub fn column() -> Self {
        Self {
            direction: FlexDirection::Column,
            children: Vec::new(),
            gap: 0.0,
            padding: 0.0,
        }
    }

    /// Add a child widget
    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    /// Set gap between children
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set uniform padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl Widget for Container {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create empty container node
        let root_id = ctx.root();
        let node_id = ctx.create_node(root_id, NodeContent::Empty);

        // Build all children
        let mut child_ids = Vec::new();
        for child in &self.children {
            let child_id = child.build(ctx);
            child_ids.push(child_id);
        }

        // Re-parent all children to this container
        let scene = ctx.scene_mut();

        // Remove children from root
        if let Some(root_node) = scene.get_node_mut(root_id) {
            for &child_id in &child_ids {
                if let Some(pos) = root_node.children.iter().position(|&id| id == child_id) {
                    root_node.children.remove(pos);
                }
            }
        }

        // Add children to container
        if let Some(container_node) = scene.get_node_mut(node_id) {
            container_node.children.extend(child_ids);
        }

        // Configure layout
        let style = FlexStyle {
            direction: self.direction,
            gap: self.gap,
            padding_left: self.padding,
            padding_right: self.padding,
            padding_top: self.padding,
            padding_bottom: self.padding,
            ..Default::default()
        };

        ctx.set_layout_style(node_id, style);

        node_id
    }
}
