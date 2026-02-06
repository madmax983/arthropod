//! Center widget - centers a child widget
//!
//! NOTE: Full centering support (justify_content + align_items) will be available once
//! layout_engine FlexStyle supports these properties. For now, Center is a placeholder
//! wrapper that can be enhanced later.

use crate::{Widget, WidgetContext};
use layout_engine::FlexStyle;
use render_engine::{NodeContent, NodeId};

/// Center widget for centering content
///
/// Wraps a single child widget and centers it within available space.
///
/// # Example
///
/// ```no_run
/// use widget_core::{Center, Button};
///
/// // Center a button
/// let centered = Center::new(Button::new("Click Me"));
/// ```
///
/// # Future Enhancement
///
/// Once `layout_engine::FlexStyle` supports `align_items` and `justify_content`,
/// this widget will use flexbox centering for true bidirectional centering.
pub struct Center<W: Widget> {
    child: W,
}

impl<W: Widget> Center<W> {
    /// Create a centered wrapper around a child widget
    pub fn new(child: W) -> Self {
        Self { child }
    }
}

impl<W: Widget> Widget for Center<W> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create empty container node
        let root_id = ctx.root();
        let node_id = ctx.create_node(root_id, NodeContent::Empty);

        // Build child and reparent to container
        let child_id = self.child.build(ctx);
        ctx.reparent_to(child_id, node_id);

        // Configure layout
        // TODO: Once FlexStyle supports align_items and justify_content:
        // - Set justify_content = Center
        // - Set align_items = Center
        let style = FlexStyle {
            ..Default::default()
        };

        ctx.set_layout_style(node_id, style);

        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Text;

    #[test]
    fn test_center_creation() {
        let _center = Center::new(Text::new("Centered"));
        // Center widget construction succeeds
    }

    #[test]
    fn test_center_wraps_child() {
        let center = Center::new(Text::new("Test"));
        // Verify the child is stored
        let _ = &center.child;
    }
}
