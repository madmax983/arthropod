//! TextInput widget - single-line text input field

use crate::{validation::Validator, Text, Widget, WidgetContext};
use flux_state::{ReadSignal, Signal, WriteSignal};
use glam::Vec4;
use layout_engine::FlexDirection;
use render_engine::{Color, NodeContent, NodeId};
use theme_engine::DesignTokens;

/// TextInput widget with cursor, selection, and validation
///
/// Use the `input!` macro for declarative construction:
/// ```no_run
/// use widget_core::input;
/// use flux_state::{Runtime, Signal};
///
/// let runtime = Runtime::new();
///
/// // Basic input
/// let value1 = Signal::new(runtime.clone(), String::new());
/// let input1 = input!(value1);
///
/// // With placeholder
/// let value2 = Signal::new(runtime.clone(), String::new());
/// let input2 = input!(value2, placeholder: "Enter name...");
///
/// // With validator
/// let value3 = Signal::new(runtime.clone(), String::new());
/// let input3 = input!(value3, placeholder: "Email", validator: |s| {
///     if s.contains('@') { Ok(()) } else { Err("Invalid email".into()) }
/// });
///
/// // Multiple options
/// let value4 = Signal::new(runtime, String::new());
/// let input4 = input!(value4,
///     placeholder: "Password",
///     max_length: 32,
///     padding: 12.0
/// );
/// ```
///
/// Or use the builder pattern directly:
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
#[derive(Widget)]
#[widget(name = "input", positional_type = "Signal<String>")]
pub struct TextInput {
    // Internal: stores split signal (not exposed in macro)
    read_signal: ReadSignal<String>,
    write_signal: WriteSignal<String>,

    #[param]
    placeholder: Option<String>,
    #[callback]
    validator: Option<Validator>,
    #[flag]
    readonly: bool,
    #[param]
    max_length: Option<usize>,
    #[param]
    width: Option<f32>,
    #[param]
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
        // Get colors from design tokens (must be done before mutable ctx operations)
        // We extract colors first to avoid holding a reference across mutable borrows
        let (bg_color, text_color_primary, placeholder_color) =
            Self::get_colors_from_tokens(ctx.design_tokens());

        // Create input container with themed background
        let input_node = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::rgba(bg_color.x, bg_color.y, bg_color.z, bg_color.w),
            },
        );

        // Get current value and compute derived state without cloning the full string
        let (display_text, validation_result, is_empty) = self.read_signal.with_untracked(|value| {
            let is_empty = value.is_empty();

            let display = if is_empty && self.placeholder.is_some() {
                self.placeholder.as_ref().unwrap().clone()
            } else {
                value.clone()
            };

            let result = self.validate(value);

            (display, result, is_empty)
        });

        let text_color = if is_empty && self.placeholder.is_some() {
            placeholder_color // Use themed placeholder color
        } else {
            text_color_primary // Use themed text color
        };

        let text_widget = Text::new(display_text).color(text_color);
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
        ctx.add_text_input_state(
            input_node,
            self.read_signal.clone(),
            self.write_signal.clone(),
            self.readonly,
            self.max_length,
        );

        // Add validator if present
        if let Some(validator) = &self.validator {
            ctx.set_validator(input_node, validator.clone(), validation_result);
        }

        // Mark as having placeholder if empty and placeholder exists
        if is_empty && self.placeholder.is_some() {
            ctx.add_placeholder(input_node);
        }

        // Set background color for style checks (using themed color)
        ctx.set_background_color(input_node, bg_color);

        input_node
    }
}

impl TextInput {
    /// Get colors from design tokens or fall back to defaults
    ///
    /// Returns (background_color, text_color, placeholder_color)
    ///
    /// If design tokens are provided, uses themed colors:
    /// - Background: surface_primary (for input background)
    /// - Text: text_primary
    /// - Placeholder: text_secondary (dimmer color for placeholder text)
    fn get_colors_from_tokens(tokens: Option<&DesignTokens>) -> (Vec4, Vec4, Vec4) {
        match tokens {
            Some(t) => (
                t.surface_primary.as_color(),
                t.text_primary,
                t.text_secondary, // Placeholder uses secondary text color
            ),
            // Fallback to hardcoded values if no tokens (backwards compatibility)
            None => (
                Vec4::new(1.0, 1.0, 1.0, 1.0), // White background
                Vec4::new(0.0, 0.0, 0.0, 1.0), // Black text
                Vec4::new(0.5, 0.5, 0.5, 1.0), // Gray placeholder
            ),
        }
    }
}
