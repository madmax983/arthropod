//! Column widget - vertical layout container
//!
//! NOTE: Alignment and justification helpers (align_center, justify_end, etc.) will be added
//! once the layout_engine supports align_items and justify_content in FlexStyle.
//! For now, Column is a convenience wrapper that sets FlexDirection::Column.

use crate::{Widget, WidgetContext, WidgetTuple};
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{NodeContent, NodeId};

/// Column widget for vertical layout.
///
/// A convenience wrapper around flexbox column layout.
/// Children are laid out vertically (top-to-bottom).
///
/// The generic type `C` represents the children, which must implement [`WidgetTuple`].
/// This trait is implemented for tuples of widgets up to a certain size (e.g., `(W1, W2, W3)`).
///
/// # Example
///
/// ```
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
    style: Option<crate::Style>,
}

impl<C: WidgetTuple> Column<C> {
    /// Create a new column with the given children.
    ///
    /// Children are provided as a tuple of widgets.
    ///
    /// # Example
    ///
    /// ```
    /// use widget_core::{Column, Text};
    ///
    /// let col = Column::new((
    ///     Text::new("First"),
    ///     Text::new("Second"),
    /// ));
    /// ```
    pub fn new(children: C) -> Self {
        Self {
            children,
            gap: 0.0,
            padding: 0.0,
            style: None,
        }
    }

    /// Set the gap between children (in logical pixels).
    ///
    /// # Example
    ///
    /// ```
    /// use widget_core::{Column, Text};
    ///
    /// let col = Column::new((Text::new("A"), Text::new("B")))
    ///     .gap(10.0);
    /// ```
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set uniform padding around the column content (in logical pixels).
    ///
    /// # Example
    ///
    /// ```
    /// use widget_core::{Column, Text};
    ///
    /// let col = Column::new((Text::new("A"), Text::new("B")))
    ///     .padding(20.0);
    /// ```
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Set a high-level widget style
    pub fn style(mut self, style: crate::Style) -> Self {
        self.style = Some(style);
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
        let layout = FlexStyle {
            direction: FlexDirection::Column,
            gap: self.gap,
            padding_left: self.padding,
            padding_right: self.padding,
            padding_top: self.padding,
            padding_bottom: self.padding,
            ..Default::default()
        };

        ctx.set_layout_style(node_id, layout);

        // Apply high-level style if present
        if let Some(style) = &self.style {
            ctx.set_widget_style(node_id, style.clone());

            // Apply initial resolved style
            let resolved = style.resolve(false, false, false, false);
            ctx.apply_style(node_id, &resolved);

            if style.hover.is_some() || style.active.is_some() || style.focus.is_some() {
                ctx.add_hover_state(node_id);
            }
        }

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
