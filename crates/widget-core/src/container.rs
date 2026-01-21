//! Container widget - flexbox layout container

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
/// // With generated macros:
/// // col!([Text::new("Hello"), Text::new("World")], gap: 10.0)
/// // row!([widget1, widget2], padding: 16.0)
/// ```
pub struct Container<C: WidgetTuple> {
    direction: FlexDirection,
    children: C,
    gap: f32,
    padding: f32,
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
        }
    }

    /// Create a column container with children
    pub fn column(children: C) -> Self {
        Self {
            direction: FlexDirection::Column,
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

impl<C: WidgetTuple> Widget for Container<C> {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create empty container node
        let root_id = ctx.root();
        let node_id = ctx.create_node(root_id, NodeContent::Empty);

        // Build all children using WidgetTuple trait
        self.children.build_all(ctx, node_id);

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
