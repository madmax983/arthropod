//! Widget Core - Comprehensive widget library for Arthropod GUI framework
//!
//! Provides a declarative widget API for building cross-platform user interfaces.
//! Integrates with Scene (hierarchy), ECS (components), Layout, and Text engines.
//!
//! # Widget Categories
//!
//! ## Primitive Widgets
//! - [`Text`] - Static or reactive text display
//! - [`Button`] - Clickable button with styles (primary, secondary, disabled)
//! - [`TextInput`] - Single-line text input with validation
//! - [`Checkbox`] - Boolean toggle with optional label
//!
//! ## Layout Widgets
//! - [`Row`] - Horizontal layout with alignment helpers
//! - [`Column`] - Vertical layout with alignment helpers
//! - [`Spacer`] - Flexible or fixed spacing between widgets
//! - [`Divider`] - Visual separator line (horizontal or vertical)
//! - [`Padding`] - Adds padding around a single child
//! - [`Center`] - Centers a child widget horizontally and/or vertically
//!
//! ## Advanced Layouts
//! - [`Stack`] - Overlay widgets with z-ordering
//! - [`Grid`] - 2D grid layout with fixed columns
//! - [`List`] - Dynamic list for runtime-generated content
//!
//! ## Container Widgets
//! - [`Container`] - Generic container with flexbox layout (legacy, prefer Row/Column)
//! - [`Card`] - Themed card with padding and background
//!
//! ## Form Widgets
//! - [`Form`] - Form with validation and submission
//!
//! # Widget Macros
//!
//! This crate provides declarative macros for common widgets:
//!
//! - `txt!` - Text widget shorthand
//! - `btn!` / `button!` - Button widget shorthand
//! - `col!` / `column!` - Column layout shorthand
//! - `row!` - Row layout shorthand
//! - `form!` - Form widget shorthand
//! - `input!` - TextInput widget shorthand
//!
//! # Examples
//!
//! ## Using Macros (Recommended)
//!
//! ```no_run
//! use widget_core::{txt, btn, col, row, Spacer};
//!
//! // Text widget
//! let title = txt!("Hello World", size: 24.0);
//!
//! // Justified toolbar with spacer
//! let toolbar = row!([
//!     btn!("Back"),
//!     Spacer::flex(),  // Pushes remaining items to right
//!     btn!("Save", primary),
//!     btn!("Share", primary),
//! ], gap: 8.0);
//!
//! // Nested layouts
//! let layout = col!([
//!     txt!("Settings", size: 20.0),
//!     row!([btn!("OK"), btn!("Cancel")], gap: 8.0),
//! ], gap: 16.0, padding: 20.0);
//! ```
//!
//! ## Using Builder Pattern
//!
//! ```no_run
//! use widget_core::{Text, Button, Row, Column, Center, Grid};
//!
//! // Text widget
//! let title = Text::new("Hello World").size(24.0);
//!
//! // Row layout with gap
//! let row = Row::new((
//!     Button::new("Left"),
//!     Button::new("Center"),
//!     Button::new("Right"),
//! ))
//! .gap(10.0)
//! .padding(8.0);
//!
//! // Centered content
//! let centered = Center::new(Text::new("Centered!"));
//!
//! // Grid layout
//! let grid = Grid::new(
//!     (
//!         Text::new("A"), Text::new("B"),
//!         Text::new("C"), Text::new("D"),
//!     ),
//!     2, // columns
//! )
//! .gap(10.0);
//! ```
//!
//! # Custom Widgets
//!
//! To create your own widgets, implement the [`Widget`] trait:
//!
//! ```no_run
//! use widget_core::{Widget, WidgetContext};
//! use render_engine::{NodeId, NodeContent};
//!
//! struct MyWidget {
//!     label: String,
//! }
//!
//! impl Widget for MyWidget {
//!     fn build(&self, ctx: &mut WidgetContext) -> NodeId {
//!         // Create scene node, configure layout, register ECS components
//!         todo!()
//!     }
//! }
//! ```
//!
//! See [`trait@Widget`] for a detailed guide and complete examples.

pub mod button;
pub mod card;
pub mod center;
pub mod checkbox;
pub mod column;
pub mod container;
pub mod context;
pub mod divider;
pub mod form;
pub mod form_state;
pub mod grid;
pub mod input_state;
pub mod list;
pub mod padding;
pub mod row;
pub mod spacer;
pub mod stack;
pub mod text;
pub mod text_input;
pub mod validation;
pub mod validation_state;
pub mod widget_trait;

pub use button::{Button, ButtonStyle};
pub use card::Card;
pub use center::Center;
pub use checkbox::Checkbox;
pub use column::Column;
pub use container::Container;
pub use context::WidgetContext;
pub use divider::Divider;
pub use form::Form;
pub use grid::Grid;
pub use list::{list_from, List, WidgetBoxed};
pub use padding::Padding;
pub use row::Row;
pub use spacer::Spacer;
pub use stack::Stack;
pub use text::Text;
pub use text_input::TextInput;
pub use widget_trait::{NamedWidgetTuple, Widget, WidgetTuple};

// Re-export layout types for advanced use cases (manual container construction)
pub use layout_engine::{FlexDirection, FlexStyle};
// Re-export node content for advanced use cases
pub use render_engine::NodeContent;
// Re-export theme types for theming support
pub use theme_engine::{DesignTokens, SystemTheme};

// Re-export derive macros - Widget derive coexists with Widget trait (different namespaces)
pub use widget_macros::Widget;
pub use widget_macros::WidgetEnum;

// Re-export generated macros from this crate
// (txt!, btn!, col!, row! are defined via #[derive] and #[macro_export])

#[cfg(test)]
mod tests {
    #[test]
    fn test_widget_core_compiles() {
        // Sanity check that the crate compiles
    }
}
