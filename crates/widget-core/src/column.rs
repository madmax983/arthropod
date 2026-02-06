//! Column widget - vertical layout container
//!
//! NOTE: Alignment and justification helpers (align_center, justify_end, etc.) will be added
//! once the layout_engine supports align_items and justify_content in FlexStyle.
//! For now, Column is a convenience wrapper that sets FlexDirection::Column.

use crate::{Widget, WidgetContext, WidgetTuple};
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{NodeContent, NodeId};

/// Column widget for vertical layout
///
/// A convenience wrapper around flexbox column layout.
/// Children are laid out vertically (top-to-bottom).
///
/// # Example
///
/// ```no_run
/// use widget_core::{Column, Text, Button};
///
/// // Simple column
/// let column = Column::new((
///     Text::new("Title"),
///     Text::new("Subtitle"),
///     Button::new("Action"),
/// )).gap(8.0);
///
/// // With padding
/// let padded = Column::new((
///     Text::new("Line 1"),
///     Text::new("Line 2"),
/// ))
/// .padding(20.0)
/// .gap(10.0);
/// ```
pub struct Column<C: WidgetTuple> {
    children: C,
    gap: f32,
    padding: f32,
}

impl<C: WidgetTuple> Column<C> {
    /// Create a new column with children
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

impl<C: WidgetTuple> Widget for Column<C> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create empty container node
        let root_id = ctx.root();
        let node_id = ctx.create_node(root_id, NodeContent::Empty);

        // Build all children using WidgetTuple trait
        self.children.build_all(ctx, node_id);

        // Configure layout with column direction
        let style = FlexStyle {
            direction: FlexDirection::Column,
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
    fn test_column_creation() {
        let column = Column::new((Text::new("A"), Text::new("B")));
        assert_eq!(column.gap, 0.0);
        assert_eq!(column.padding, 0.0);
    }

    #[test]
    fn test_column_gap() {
        let column = Column::new((Text::new("A"), Text::new("B"))).gap(10.0);
        assert_eq!(column.gap, 10.0);
    }

    #[test]
    fn test_column_padding() {
        let column = Column::new((Text::new("A"), Text::new("B"))).padding(20.0);
        assert_eq!(column.padding, 20.0);
    }

    #[test]
    fn test_column_chained_builders() {
        let column = Column::new((Text::new("A"), Text::new("B"), Text::new("C")))
            .gap(8.0)
            .padding(16.0);

        assert_eq!(column.gap, 8.0);
        assert_eq!(column.padding, 16.0);
    }
}
