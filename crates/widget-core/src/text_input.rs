//! TextInput widget - single-line text input field

use crate::{Widget, WidgetContext, Text, validation::Validator};
use render_engine::{NodeId, NodeContent, Color};
use layout_engine::FlexDirection;
use flux_state::{Signal, ReadSignal, WriteSignal};
use glam::Vec4;

/// TextInput widget with cursor, selection, and validation
///
/// # Example
///
/// ```no_run
/// use widget_core::TextInput;
/// use flux_state::{Runtime, Signal};
///
/// let runtime = Runtime::new();
/// let value = Signal::new(runtime, String::new());
///
/// let input = TextInput::new(value)
///     .placeholder("Enter text...")
///     .validator(|s| {
///         if s.is_empty() {
///             Err("Required".to_string())
///         } else {
///             Ok(())
///         }
///     });
/// ```
pub struct TextInput {
    read_signal: ReadSignal<String>,
    write_signal: WriteSignal<String>,
    placeholder: Option<String>,
    validator: Option<Validator>,
    readonly: bool,
    max_length: Option<usize>,
    width: Option<f32>,
    padding: f32,
}

impl TextInput {
    /// Create a new text input with a value signal
    pub fn new(value: Signal<String>) -> Self {
        let (read_signal, write_signal) = value.split();
        Self {
            read_signal,
            write_signal,
            placeholder: None,
            validator: None,
            readonly: false,
            max_length: None,
            width: None,
            padding: 8.0,
        }
    }

    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set validation function
    pub fn validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.validator = Some(std::sync::Arc::new(validator));
        self
    }

    /// Set readonly state
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Validate the current value
    fn validate(&self, value: &str) -> Result<(), String> {
        if let Some(validator) = &self.validator {
            validator(value)
        } else {
            Ok(())
        }
    }
}

impl Widget for TextInput {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Create input container with border
        let input_node = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::rgba(1.0, 1.0, 1.0, 1.0), // White background
            },
        );

        // Get current value
        let current_value = self.read_signal.get_untracked();

        // Create text display (or placeholder)
        let display_text = if current_value.is_empty() && self.placeholder.is_some() {
            self.placeholder.as_ref().unwrap().clone()
        } else {
            current_value.clone()
        };

        let text_color = if current_value.is_empty() && self.placeholder.is_some() {
            Vec4::new(0.5, 0.5, 0.5, 1.0) // Gray for placeholder
        } else {
            Vec4::new(0.0, 0.0, 0.0, 1.0) // Black for value
        };

        let text_widget = Text::new(display_text)
            .color(text_color);
        let text_id = text_widget.build(ctx);

        // Re-parent text to input
        let root_id = ctx.root();
        ctx.reparent_node(text_id, root_id, input_node);

        // Configure layout
        let layout_style = layout_engine::FlexStyle {
            direction: FlexDirection::Row,
            padding_left: self.padding,
            padding_right: self.padding,
            padding_top: self.padding,
            padding_bottom: self.padding,
            ..Default::default()
        };

        ctx.set_layout_style(input_node, layout_style);

        // Add input state tracking
        ctx.add_text_input_state(input_node, self.read_signal.clone(), self.write_signal.clone(), self.readonly, self.max_length);

        // Add validator if present
        if let Some(validator) = &self.validator {
            let validation_result = self.validate(&current_value);
            ctx.set_validator(input_node, validator.clone(), validation_result);
        }

        // Mark as having placeholder if empty and placeholder exists
        if current_value.is_empty() && self.placeholder.is_some() {
            ctx.add_placeholder(input_node);
        }

        // Set background color for style checks
        ctx.set_background_color(input_node, Vec4::new(1.0, 1.0, 1.0, 1.0));

        input_node
    }
}
