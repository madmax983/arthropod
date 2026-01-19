//! Widget Core - Widget trait and primitive widgets
//!
//! Provides the foundation for building UI with a declarative widget API.
//! Integrates with Scene (hierarchy), ECS (components), Layout, and Text engines.

pub mod widget_trait;
pub mod container;
pub mod text;
pub mod button;
pub mod text_input;
pub mod validation;
pub mod context;

pub use widget_trait::Widget;
pub use container::Container;
pub use text::Text;
pub use button::Button;
pub use text_input::TextInput;
pub use context::WidgetContext;

#[cfg(test)]
mod tests {
    #[test]
    fn test_widget_core_compiles() {
        // Sanity check that the crate compiles
    }
}
