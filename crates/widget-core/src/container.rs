//! Container widget - flexbox layout container
//!
//! NOTE: Alignment and justification helpers (align_start, justify_center, etc.) will be added
//! once layout_engine FlexStyle supports align_items and justify_content properties.

use crate::{Widget, WidgetContext, WidgetTuple};
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::{NodeContent, NodeId};

/// Container widget for layout with type-safe children
///
/// Implements flexbox layout with row/column direction.
/// Uses the `WidgetTuple` trait for compile-time typed children (no vtable overhead).
///
/// # Example
///
/// ```no_run
/// use widget_core::{Container, Text};
///
/// // Tuple-based construction (compile-time typed, no vtable overhead)
/// let widget = Container::column((
///     Text::new("Hello"),
///     Text::new("World"),
/// )).gap(10.0).padding(16.0);
///
/// // With sizing
/// let sized = Container::row((
///     Text::new("A"),
///     Text::new("B"),
/// ))
/// .width(200.0)
/// .height(100.0);
///
/// // With generated macros:
/// // col!([Text::new("Hello"), Text::new("World")], gap: 10.0)
/// // row!([widget1, widget2], padding: 16.0)
/// ```
pub struct Container<C: WidgetTuple> {
    direction: FlexDirection,
    children: C,
    gap: f32,
    padding: f32,
    width: Option<f32>,
    height: Option<f32>,
    flex_grow: f32,
    style: Option<crate::Style>,
}

/// Create a column container with children
///
/// # Example
///
/// ```no_run
/// use widget_core::{col, Text};
///
/// let column = col!([
///     Text::new("Line 1"),
///     Text::new("Line 2"),
/// ], gap: 10.0, padding: 16.0);
/// ```
#[macro_export]
macro_rules! col {
    // Children only - wrap in tuple
    ([$($children:expr),* $(,)?]) => {{
        $crate::Container::column(($($children,)*))
    }};

    // Children + params
    ([$($children:expr),* $(,)?], $($rest:tt)*) => {{
        let widget = $crate::Container::column(($($children,)*));
        $crate::__col_apply!(widget, $($rest)*)
    }};
}

/// Helper macro for applying col! parameters
#[macro_export]
#[doc(hidden)]
macro_rules! __col_apply {
    ($w:expr,) => { $w };
    ($w:expr) => { $w };
    ($w:expr, , $($rest:tt)*) => { $crate::__col_apply!($w, $($rest)*) };

    // Named parameters
    ($w:expr, gap: $v:expr $(, $($rest:tt)*)?) => {
        $crate::__col_apply!($w.gap($v), $($($rest)*)?)
    };
    ($w:expr, padding: $v:expr $(, $($rest:tt)*)?) => {
        $crate::__col_apply!($w.padding($v), $($($rest)*)?)
    };
}

/// Create a row container with children
///
/// # Example
///
/// ```no_run
/// use widget_core::{row, Button};
///
/// let buttons = row!([
///     Button::new("OK"),
///     Button::new("Cancel"),
/// ], gap: 8.0);
/// ```
#[macro_export]
macro_rules! row {
    // Children only - wrap in tuple
    ([$($children:expr),* $(,)?]) => {{
        $crate::Container::row(($($children,)*))
    }};

    // Children + params
    ([$($children:expr),* $(,)?], $($rest:tt)*) => {{
        let widget = $crate::Container::row(($($children,)*));
        $crate::__row_apply!(widget, $($rest)*)
    }};
}

/// Helper macro for applying row! parameters
#[macro_export]
#[doc(hidden)]
macro_rules! __row_apply {
    ($w:expr,) => { $w };
    ($w:expr) => { $w };
    ($w:expr, , $($rest:tt)*) => { $crate::__row_apply!($w, $($rest)*) };

    // Named parameters
    ($w:expr, gap: $v:expr $(, $($rest:tt)*)?) => {
        $crate::__row_apply!($w.gap($v), $($($rest)*)?)
    };
    ($w:expr, padding: $v:expr $(, $($rest:tt)*)?) => {
        $crate::__row_apply!($w.padding($v), $($($rest)*)?)
    };
}

impl<C: WidgetTuple> Container<C> {
    /// Create a row container with children
    pub fn row(children: C) -> Self {
        Self {
            direction: FlexDirection::Row,
            children,
            gap: 0.0,
            padding: 0.0,
            width: None,
            height: None,
            flex_grow: 0.0,
            style: None,
        }
    }

    /// Create a column container with children
    pub fn column(children: C) -> Self {
        Self {
            direction: FlexDirection::Column,
            children,
            gap: 0.0,
            padding: 0.0,
            width: None,
            height: None,
            flex_grow: 0.0,
            style: None,
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

    /// Set fixed width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set fixed height
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Make container fill available space (flex_grow: 1)
    pub fn fill(mut self) -> Self {
        self.flex_grow = 1.0;
        self
    }

    /// Set a high-level widget style
    pub fn style(mut self, style: crate::Style) -> Self {
        self.style = Some(style);
        self
    }
}

impl<C: WidgetTuple> Widget for Container<C> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create empty container node
        let root_id = ctx.root();
        let node_id = ctx.create_node(root_id, NodeContent::Empty);

        // Build all children using WidgetTuple trait
        self.children.build_all(ctx, node_id);

        // Configure layout
        let layout = FlexStyle {
            direction: self.direction,
            gap: self.gap,
            padding_left: self.padding,
            padding_right: self.padding,
            padding_top: self.padding,
            padding_bottom: self.padding,
            width: self.width,
            height: self.height,
            flex_grow: self.flex_grow,
            ..Default::default()
        };

        ctx.set_layout_style(node_id, layout);

        // Apply high-level style if present
        if let Some(style) = &self.style {
            ctx.set_widget_style(node_id, style.clone());

            // Apply initial resolved style for backwards compatibility and immediate visual feedback
            let resolved = style.resolve(false, false, false, false);
            ctx.apply_style(node_id, &resolved);

            // If the style has pseudo-states, we need to track interactions
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
    fn test_container_row_creation() {
        let container = Container::row((Text::new("A"), Text::new("B")));
        assert_eq!(container.gap, 0.0);
        assert_eq!(container.padding, 0.0);
        assert_eq!(container.width, None);
        assert_eq!(container.height, None);
        assert_eq!(container.flex_grow, 0.0);
    }

    #[test]
    fn test_container_column_creation() {
        let container = Container::column((Text::new("A"), Text::new("B")));
        assert_eq!(container.gap, 0.0);
        assert_eq!(container.padding, 0.0);
    }

    #[test]
    fn test_container_width() {
        let container = Container::row((Text::new("A"),)).width(200.0);
        assert_eq!(container.width, Some(200.0));
    }

    #[test]
    fn test_container_height() {
        let container = Container::row((Text::new("A"),)).height(100.0);
        assert_eq!(container.height, Some(100.0));
    }

    #[test]
    fn test_container_fill() {
        let container = Container::row((Text::new("A"),)).fill();
        assert_eq!(container.flex_grow, 1.0);
    }

    #[test]
    fn test_container_chained_builders() {
        let container = Container::column((Text::new("A"), Text::new("B")))
            .gap(10.0)
            .padding(20.0)
            .width(300.0)
            .height(150.0)
            .fill();

        assert_eq!(container.gap, 10.0);
        assert_eq!(container.padding, 20.0);
        assert_eq!(container.width, Some(300.0));
        assert_eq!(container.height, Some(150.0));
        assert_eq!(container.flex_grow, 1.0);
    }
}
