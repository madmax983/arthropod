//! Spacer widget - flexible or fixed spacing between widgets

use crate::{Widget, WidgetContext};
use layout_engine::FlexStyle;
use render_engine::{NodeContent, NodeId};

/// Spacer widget for adding space between widgets
///
/// Can be either flexible (grows to fill available space) or fixed size.
/// Flexible spacers are useful for pushing widgets to edges or centering them.
///
/// # Example
///
/// ```no_run
/// use widget_core::{Spacer, Row, Button};
///
/// // Flexible spacer (grows to fill available space)
/// let row = Row::new((
///     Button::new("Left"),
///     Spacer::flex(),  // Pushes button to edges
///     Button::new("Right"),
/// ));
///
/// // Fixed-size spacer
/// let row = Row::new((
///     Button::new("A"),
///     Spacer::fixed(20.0),  // 20px gap
///     Button::new("B"),
/// ));
///
/// // Vertical spacer in column
/// use widget_core::Column;
/// let column = Column::new((
///     Button::new("Top"),
///     Spacer::flex(),  // Pushes to top/bottom
///     Button::new("Bottom"),
/// ));
/// ```
pub struct Spacer {
    flex_grow: f32,
    width: Option<f32>,
    height: Option<f32>,
}

impl Spacer {
    /// Create a flexible spacer that grows to fill available space
    pub fn flex() -> Self {
        Self {
            flex_grow: 1.0,
            width: None,
            height: None,
        }
    }

    /// Create a flexible spacer with custom flex_grow value
    pub fn flex_with(flex_grow: f32) -> Self {
        Self {
            flex_grow,
            width: None,
            height: None,
        }
    }

    /// Create a fixed-size spacer
    ///
    /// The size applies to the main axis direction (width in Row, height in Column).
    pub fn fixed(size: f32) -> Self {
        Self {
            flex_grow: 0.0,
            width: Some(size),
            height: Some(size),
        }
    }

    /// Create a fixed-width spacer (for horizontal spacing in Row)
    pub fn width(width: f32) -> Self {
        Self {
            flex_grow: 0.0,
            width: Some(width),
            height: None,
        }
    }

    /// Create a fixed-height spacer (for vertical spacing in Column)
    pub fn height(height: f32) -> Self {
        Self {
            flex_grow: 0.0,
            width: None,
            height: Some(height),
        }
    }
}

impl Widget for Spacer {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create empty node (invisible)
        let root_id = ctx.root();
        let node_id = ctx.create_node(root_id, NodeContent::Empty);

        // Configure layout with flex properties
        let style = FlexStyle {
            flex_grow: self.flex_grow,
            width: self.width,
            height: self.height,
            ..Default::default()
        };

        ctx.set_layout_style(node_id, style);

        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacer_flex() {
        let spacer = Spacer::flex();
        assert_eq!(spacer.flex_grow, 1.0);
        assert_eq!(spacer.width, None);
        assert_eq!(spacer.height, None);
    }

    #[test]
    fn test_spacer_flex_with() {
        let spacer = Spacer::flex_with(2.0);
        assert_eq!(spacer.flex_grow, 2.0);
        assert_eq!(spacer.width, None);
        assert_eq!(spacer.height, None);
    }

    #[test]
    fn test_spacer_fixed() {
        let spacer = Spacer::fixed(20.0);
        assert_eq!(spacer.flex_grow, 0.0);
        assert_eq!(spacer.width, Some(20.0));
        assert_eq!(spacer.height, Some(20.0));
    }

    #[test]
    fn test_spacer_width() {
        let spacer = Spacer::width(30.0);
        assert_eq!(spacer.flex_grow, 0.0);
        assert_eq!(spacer.width, Some(30.0));
        assert_eq!(spacer.height, None);
    }

    #[test]
    fn test_spacer_height() {
        let spacer = Spacer::height(40.0);
        assert_eq!(spacer.flex_grow, 0.0);
        assert_eq!(spacer.width, None);
        assert_eq!(spacer.height, Some(40.0));
    }
}
