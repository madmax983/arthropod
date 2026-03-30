//! Row widget - horizontal layout container
//!
//! NOTE: Alignment and justification helpers (align_center, justify_end, etc.) will be added
//! once the layout_engine supports align_items and justify_content in FlexStyle.
//! For now, Row is a convenience wrapper that sets FlexDirection::Row.

use crate::{Widget, WidgetContext, WidgetTuple};
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{NodeContent, NodeId};

/// Row widget for horizontal layout
///
/// A convenience wrapper around flexbox row layout.
/// Children are laid out horizontally (left-to-right).
///
/// # Example
///
/// ```no_run
/// use widget_core::{Row, Text, Button};
///
/// // Simple row
/// let row = Row::new((
///     Text::new("Label:"),
///     Button::new("Action"),
/// )).gap(8.0);
///
/// // With padding
/// let padded = Row::new((
///     Text::new("Hello"),
///     Text::new("World"),
/// ))
/// .padding(20.0)
/// .gap(10.0);
/// ```
pub struct Row<C: WidgetTuple> {
    children: C,
    gap: f32,
    padding: f32,
}

impl<C: WidgetTuple> Row<C> {
    /// Create a new row with children
    pub fn new(children: C) -> Self {
        Self {
            children,
            gap: 0.0,
            padding: 0.0,
        }
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

impl<C: WidgetTuple> Widget for Row<C> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create empty container node
        let root_id = ctx.root();
        let node_id = ctx.create_node(root_id, NodeContent::Empty);

        // Build all children using WidgetTuple trait
        self.children.build_all(ctx, node_id);

        // Configure layout with row direction
        let style = FlexStyle {
            direction: FlexDirection::Row,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Text;

    #[test]
    fn test_row_creation() {
        let row = Row::new((Text::new("A"), Text::new("B")));
        assert_eq!(row.gap, 0.0);
        assert_eq!(row.padding, 0.0);
    }

    #[test]
    fn test_row_gap() {
        let row = Row::new((Text::new("A"), Text::new("B"))).gap(10.0);
        assert_eq!(row.gap, 10.0);
    }

    #[test]
    fn test_row_padding() {
        let row = Row::new((Text::new("A"), Text::new("B"))).padding(20.0);
        assert_eq!(row.padding, 20.0);
    }

    #[test]
    fn test_row_chained_builders() {
        let row = Row::new((Text::new("A"), Text::new("B")))
            .gap(8.0)
            .padding(16.0);

        assert_eq!(row.gap, 8.0);
        assert_eq!(row.padding, 16.0);
    }
}
