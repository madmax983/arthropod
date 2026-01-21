//! Widget Core - Widget trait and primitive widgets
//!
//! Provides the foundation for building UI with a declarative widget API.
//! Integrates with Scene (hierarchy), ECS (components), Layout, and Text engines.
//!
//! # Widget Macros
//!
//! This crate provides declarative macros for widget creation:
//!
//! - `txt!` / `text!` - Text widget
//! - `btn!` / `button!` - Button widget
//! - `col!` / `column!` - Column container
//! - `row!` - Row container
//!
//! # Examples
//!
//! ```no_run
//! use widget_core::{txt, btn, col, row, Text, Button, Container};
//!
//! // Text widget
//! let title = txt!("Hello World");
//! let styled = txt!("Styled", size: 24.0);
//!
//! // Button widget
//! let button = btn!("Click Me");
//! let primary_btn = btn!("Save", primary, on_click: || println!("Saved!"));
//!
//! // Layout containers
//! let layout = col!([
//!     txt!("Header"),
//!     row!([
//!         btn!("OK"),
//!         btn!("Cancel"),
//!     ], gap: 8.0),
//! ], gap: 16.0, padding: 20.0);
//! ```

pub mod button;
pub mod card;
pub mod container;
pub mod context;
pub mod form;
pub mod text;
pub mod text_input;
pub mod validation;
pub mod widget_trait;

pub use button::{Button, ButtonStyle};
pub use card::Card;
pub use container::Container;
pub use context::WidgetContext;
pub use form::Form;
pub use text::Text;
pub use text_input::TextInput;
pub use widget_trait::{NamedWidgetTuple, Widget, WidgetTuple};

// Re-export layout types for advanced use cases (manual container construction)
pub use layout_engine::{FlexDirection, FlexStyle};
// Re-export node content for advanced use cases
pub use render_engine::NodeContent;

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
