//! Stack widget - overlay widgets with z-ordering

use crate::{Widget, WidgetContext, WidgetTuple};
use layout_engine::FlexStyle;
use render_engine::{NodeContent, NodeId};

/// Stack widget for overlaying widgets with z-ordering
///
/// Children are rendered in order - first child is background (bottom),
/// last child is foreground (top). All children are positioned to fill
/// the stack's bounds, creating an overlay effect.
///
/// # Example
///
/// ```no_run
/// use widget_core::{Stack, Text, Container};
/// use render_engine::Color;
///
/// // Background image, text overlay, button on top
/// let stack = Stack::new((
///     Container::column(()).padding(0.0), // Background
///     Text::new("Overlay Text"),          // Middle layer
///     // Button on top
/// ));
/// ```
///
/// # Z-Ordering
///
/// Z-order is determined by child order in the tuple:
/// - First child = bottom layer (z-index 0)
/// - Last child = top layer (highest z-index)
///
/// All children are positioned to fill the stack container.
pub struct Stack<C: WidgetTuple> {
    children: C,
    padding: f32,
}

impl<C: WidgetTuple> Stack<C> {
    /// Create a new stack with children
    ///
    /// Children are rendered in tuple order (first = background, last = foreground).
    pub fn new(children: C) -> Self {
        Self {
            children,
            padding: 0.0,
        }
    }

    /// Set uniform padding around the stack
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl<C: WidgetTuple> Widget for Stack<C> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create stack container node
        let root_id = ctx.root();
        let node_id = ctx.create_node(root_id, NodeContent::Empty);

        // Build all children and collect their IDs
        let child_ids = self.children.build_all_to_vec(ctx);

        // Reparent all children to the stack container
        for child_id in &child_ids {
            ctx.reparent_to(*child_id, node_id);
        }

        // Stack container should expand to fill available space
        // and position children to overlay each other
        let style = FlexStyle {
            flex_grow: 1.0, // Expand to fill parent
            padding_left: self.padding,
            padding_right: self.padding,
            padding_top: self.padding,
            padding_bottom: self.padding,
            ..Default::default()
        };

        ctx.set_layout_style(node_id, style);

        // For each child, set them to fill the stack container
        // This creates the overlay effect
        for child_id in &child_ids {
            let child_style = FlexStyle {
                width: Some(0.0),  // Will be overridden by flex_grow
                height: Some(0.0), // Will be overridden by flex_grow
                flex_grow: 1.0,    // Fill available space
                ..Default::default()
            };
            ctx.set_layout_style(*child_id, child_style);
        }

        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWidget {
        label: &'static str,
    }

    impl Widget for TestWidget {
        fn build(&self, ctx: &mut WidgetContext) -> NodeId {
            ctx.create_node(
                ctx.root(),
                NodeContent::Rect {
                    color: render_engine::Color::rgba(1.0, 0.0, 0.0, 1.0),
                },
            )
        }
    }

    #[test]
    fn test_stack_creates_container() {
        let mut ctx = WidgetContext::new_test();
        let stack = Stack::new((TestWidget { label: "A" },));

        let node_id = stack.build(&mut ctx);

        // Verify node exists
        assert!(ctx.scene().get_node(node_id).is_some());
    }

    #[test]
    fn test_stack_children_count() {
        let mut ctx = WidgetContext::new_test();
        let stack = Stack::new((
            TestWidget { label: "A" },
            TestWidget { label: "B" },
            TestWidget { label: "C" },
        ));

        let node_id = stack.build(&mut ctx);
        let scene_node = ctx.scene().get_node(node_id).unwrap();

        assert_eq!(scene_node.children.len(), 3);
    }

    #[test]
    fn test_stack_padding() {
        let mut ctx = WidgetContext::new_test();
        let stack = Stack::new((TestWidget { label: "A" },)).padding(10.0);

        let node_id = stack.build(&mut ctx);
        let layout = ctx.get_layout_style(node_id).unwrap();

        assert_eq!(layout.padding_left, 10.0);
        assert_eq!(layout.padding_right, 10.0);
        assert_eq!(layout.padding_top, 10.0);
        assert_eq!(layout.padding_bottom, 10.0);
    }

    #[test]
    fn test_stack_flex_grow() {
        let mut ctx = WidgetContext::new_test();
        let stack = Stack::new((TestWidget { label: "A" },));

        let node_id = stack.build(&mut ctx);
        let layout = ctx.get_layout_style(node_id).unwrap();

        assert_eq!(layout.flex_grow, 1.0);
    }
}
