//! Padding widget - adds space around a child widget

use crate::{Widget, WidgetContext};
use layout_engine::FlexStyle;
use render_engine::{NodeContent, NodeId};

/// Padding widget for adding space around content
///
/// Wraps a single child widget and adds padding on all sides.
/// Supports uniform, horizontal, vertical, or custom padding per side.
///
/// # Example
///
/// ```no_run
/// use widget_core::{Padding, Button};
///
/// // Uniform padding (all sides)
/// let padded = Padding::all(20.0, Button::new("Click Me"));
///
/// // Horizontal and vertical
/// let padded = Padding::symmetric(40.0, 20.0, Button::new("Click Me")); // h: 40, v: 20
///
/// // Custom per side
/// let padded = Padding::new(Button::new("Click Me"))
///     .left(10.0)
///     .right(10.0)
///     .top(5.0)
///     .bottom(5.0);
/// ```
pub struct Padding<W: Widget> {
    child: W,
    padding_left: f32,
    padding_right: f32,
    padding_top: f32,
    padding_bottom: f32,
}

impl<W: Widget> Padding<W> {
    /// Create padding around a child widget (starts with 0 padding)
    pub fn new(child: W) -> Self {
        Self {
            child,
            padding_left: 0.0,
            padding_right: 0.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
        }
    }

    /// Create uniform padding on all sides
    pub fn all(padding: f32, child: W) -> Self {
        Self {
            child,
            padding_left: padding,
            padding_right: padding,
            padding_top: padding,
            padding_bottom: padding,
        }
    }

    /// Create symmetric padding (horizontal, vertical)
    pub fn symmetric(horizontal: f32, vertical: f32, child: W) -> Self {
        Self {
            child,
            padding_left: horizontal,
            padding_right: horizontal,
            padding_top: vertical,
            padding_bottom: vertical,
        }
    }

    /// Set left padding
    pub fn left(mut self, padding: f32) -> Self {
        self.padding_left = padding;
        self
    }

    /// Set right padding
    pub fn right(mut self, padding: f32) -> Self {
        self.padding_right = padding;
        self
    }

    /// Set top padding
    pub fn top(mut self, padding: f32) -> Self {
        self.padding_top = padding;
        self
    }

    /// Set bottom padding
    pub fn bottom(mut self, padding: f32) -> Self {
        self.padding_bottom = padding;
        self
    }

    /// Set horizontal padding (left and right)
    pub fn horizontal(mut self, padding: f32) -> Self {
        self.padding_left = padding;
        self.padding_right = padding;
        self
    }

    /// Set vertical padding (top and bottom)
    pub fn vertical(mut self, padding: f32) -> Self {
        self.padding_top = padding;
        self.padding_bottom = padding;
        self
    }
}

impl<W: Widget> Widget for Padding<W> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create empty container node
        let root_id = ctx.root();
        let node_id = ctx.create_node(root_id, NodeContent::Empty);

        // Build child and reparent to container
        let child_id = self.child.build(ctx);
        ctx.reparent_to(child_id, node_id);

        // Configure layout with padding
        let style = FlexStyle {
            padding_left: self.padding_left,
            padding_right: self.padding_right,
            padding_top: self.padding_top,
            padding_bottom: self.padding_bottom,
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
    fn test_padding_new() {
        let padding = Padding::new(Text::new("Test"));
        assert_eq!(padding.padding_left, 0.0);
        assert_eq!(padding.padding_right, 0.0);
        assert_eq!(padding.padding_top, 0.0);
        assert_eq!(padding.padding_bottom, 0.0);
    }

    #[test]
    fn test_padding_all() {
        let padding = Padding::all(20.0, Text::new("Test"));
        assert_eq!(padding.padding_left, 20.0);
        assert_eq!(padding.padding_right, 20.0);
        assert_eq!(padding.padding_top, 20.0);
        assert_eq!(padding.padding_bottom, 20.0);
    }

    #[test]
    fn test_padding_symmetric() {
        let padding = Padding::symmetric(30.0, 10.0, Text::new("Test"));
        assert_eq!(padding.padding_left, 30.0);
        assert_eq!(padding.padding_right, 30.0);
        assert_eq!(padding.padding_top, 10.0);
        assert_eq!(padding.padding_bottom, 10.0);
    }

    #[test]
    fn test_padding_left() {
        let padding = Padding::new(Text::new("Test")).left(15.0);
        assert_eq!(padding.padding_left, 15.0);
        assert_eq!(padding.padding_right, 0.0);
    }

    #[test]
    fn test_padding_right() {
        let padding = Padding::new(Text::new("Test")).right(25.0);
        assert_eq!(padding.padding_right, 25.0);
        assert_eq!(padding.padding_left, 0.0);
    }

    #[test]
    fn test_padding_top() {
        let padding = Padding::new(Text::new("Test")).top(5.0);
        assert_eq!(padding.padding_top, 5.0);
        assert_eq!(padding.padding_bottom, 0.0);
    }

    #[test]
    fn test_padding_bottom() {
        let padding = Padding::new(Text::new("Test")).bottom(35.0);
        assert_eq!(padding.padding_bottom, 35.0);
        assert_eq!(padding.padding_top, 0.0);
    }

    #[test]
    fn test_padding_horizontal() {
        let padding = Padding::new(Text::new("Test")).horizontal(40.0);
        assert_eq!(padding.padding_left, 40.0);
        assert_eq!(padding.padding_right, 40.0);
        assert_eq!(padding.padding_top, 0.0);
        assert_eq!(padding.padding_bottom, 0.0);
    }

    #[test]
    fn test_padding_vertical() {
        let padding = Padding::new(Text::new("Test")).vertical(50.0);
        assert_eq!(padding.padding_top, 50.0);
        assert_eq!(padding.padding_bottom, 50.0);
        assert_eq!(padding.padding_left, 0.0);
        assert_eq!(padding.padding_right, 0.0);
    }

    #[test]
    fn test_padding_chained_builders() {
        let padding = Padding::new(Text::new("Test"))
            .left(10.0)
            .right(20.0)
            .top(5.0)
            .bottom(15.0);

        assert_eq!(padding.padding_left, 10.0);
        assert_eq!(padding.padding_right, 20.0);
        assert_eq!(padding.padding_top, 5.0);
        assert_eq!(padding.padding_bottom, 15.0);
    }
}
