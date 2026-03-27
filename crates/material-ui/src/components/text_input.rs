//! MD3 MaterialTextInput -- text field with Filled/Outlined variants.

use crate::theme::MaterialTheme;
use flux_state::Signal;
use glam::Vec4;
use layout_engine::{FlexDirection, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, Paint, StrokeStyle, TextContent, VisualStyle};
use style_engine::StrokeAlign;
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4)
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 on_surface (#1D1B20)
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// MD3 surface_variant (#E7E0EC)
const FALLBACK_SURFACE_VARIANT: Vec4 = Vec4::new(0.906, 0.878, 0.925, 1.0);

/// MD3 outline (#79747E)
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// MD3 on_surface_variant (#49454F)
const FALLBACK_ON_SURFACE_VARIANT: Vec4 = Vec4::new(0.286, 0.271, 0.310, 1.0);

/// MD3 error (#B3261E)
const FALLBACK_ERROR: Vec4 = Vec4::new(0.702, 0.149, 0.118, 1.0);

/// Transparent fill
const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

/// Disabled content opacity multiplier.
const DISABLED_ALPHA: f32 = 0.38;

/// Input container height in dp (MD3).
const INPUT_HEIGHT: f32 = 56.0;

/// Default input width in dp.
const DEFAULT_WIDTH: f32 = 280.0;

/// Outlined variant corner radius.
const OUTLINED_CORNER_RADIUS: f32 = 4.0;

/// Body-large font size (MD3).
const BODY_LARGE_SIZE: f32 = 16.0;

/// Body-small font size for supporting text (MD3).
const BODY_SMALL_SIZE: f32 = 12.0;

/// Horizontal padding inside the input container.
const INPUT_PADDING_H: f32 = 16.0;

// ---------------------------------------------------------------------------
// TextInputVariant
// ---------------------------------------------------------------------------

/// MD3 text field variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextInputVariant {
    /// Filled background, no side border, bottom indicator line.
    Filled,
    /// Transparent background with 1dp outline border all around.
    #[default]
    Outlined,
}

// ---------------------------------------------------------------------------
// MaterialTextInput
// ---------------------------------------------------------------------------

/// MD3 Text Input with Filled or Outlined variant.
///
/// Builds a column containing an optional label, a 56dp input container,
/// and optional helper/error text below. Colors are resolved from
/// [`MaterialTheme`] with sensible MD3 fallbacks.
///
/// # Example
///
/// ```rust,no_run
/// use flux_state::{Runtime, Signal};
/// use material_ui::components::MaterialTextInput;
///
/// let runtime = Runtime::new();
/// let email = Signal::new(runtime, String::new());
///
/// let input = MaterialTextInput::new(email)
///     .label("Email")
///     .placeholder("you@example.com")
///     .helper_text("We'll never share your email");
/// ```
pub struct MaterialTextInput {
    signal: Signal<String>,
    variant: TextInputVariant,
    label: Option<String>,
    placeholder: Option<String>,
    helper_text: Option<String>,
    error_text: Option<String>,
    disabled: bool,
    width: f32,
}

impl MaterialTextInput {
    /// Create a new outlined text input wired to a reactive string signal.
    pub fn new(signal: Signal<String>) -> Self {
        Self {
            signal,
            variant: TextInputVariant::Outlined,
            label: None,
            placeholder: None,
            helper_text: None,
            error_text: None,
            disabled: false,
            width: DEFAULT_WIDTH,
        }
    }

    /// Switch to the Filled variant.
    pub fn filled(mut self) -> Self {
        self.variant = TextInputVariant::Filled;
        self
    }

    /// Set a floating label text.
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Set placeholder text shown when the input is empty.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set helper text displayed below the input.
    pub fn helper_text(mut self, text: impl Into<String>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// Set error text displayed below the input (replaces helper text).
    pub fn error_text(mut self, text: impl Into<String>) -> Self {
        self.error_text = Some(text.into());
        self
    }

    /// Disable interaction.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the overall width in dp (default: 280.0).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Widget for MaterialTextInput {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let on_surface = theme
            .as_ref()
            .map(|t| t.color.on_surface)
            .unwrap_or(FALLBACK_ON_SURFACE);
        let surface_variant = theme
            .as_ref()
            .map(|t| t.color.surface_variant)
            .unwrap_or(FALLBACK_SURFACE_VARIANT);
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);
        let on_surface_variant = theme
            .as_ref()
            .map(|t| t.color.on_surface_variant)
            .unwrap_or(FALLBACK_ON_SURFACE_VARIANT);
        let error = theme
            .as_ref()
            .map(|t| t.color.error)
            .unwrap_or(FALLBACK_ERROR);

        let has_error = self.error_text.is_some();

        // -- Determine input border/outline color --
        let border_color = if has_error {
            error
        } else if self.disabled {
            Vec4::new(outline.x, outline.y, outline.z, DISABLED_ALPHA)
        } else {
            outline
        };

        // -- Root column container --
        let root = ctx.create_node(ctx.root(), NodeContent::Empty);
        ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(self.width),
                gap: 4.0,
                ..Default::default()
            },
        );

        // -- Optional label --
        if let Some(ref label_text) = self.label {
            let label_color = if has_error { error } else { primary };
            let label_style = VisualStyle::new()
                .solid_fill(label_color)
                .text(TextContent::new(label_text.clone(), BODY_SMALL_SIZE));
            let label_node = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(label_style),
                },
            );
            let _ = label_node;
        }

        // -- Input container --
        let input_style = match self.variant {
            TextInputVariant::Filled => VisualStyle::new()
                .solid_fill(surface_variant)
                .corner_radius(OUTLINED_CORNER_RADIUS),
            TextInputVariant::Outlined => VisualStyle::new()
                .solid_fill(TRANSPARENT)
                .corner_radius(OUTLINED_CORNER_RADIUS)
                .stroke(StrokeStyle::solid(
                    Paint::solid(border_color),
                    1.0,
                    StrokeAlign::Inside,
                )),
        };

        let input_container = ctx.create_node(
            root,
            NodeContent::Styled {
                style: Box::new(input_style),
            },
        );
        ctx.set_layout_style(
            input_container,
            FlexStyle {
                height: Some(INPUT_HEIGHT),
                padding_left: INPUT_PADDING_H,
                padding_right: INPUT_PADDING_H,
                align_items: layout_engine::FlexAlign::Center,
                ..Default::default()
            },
        );

        // -- Input text (current value or placeholder) --
        let (read, _write) = self.signal.clone().split();
        let current_value = read.get_untracked();
        let is_empty = current_value.is_empty();
        let display_text = if is_empty {
            self.placeholder.clone().unwrap_or_default()
        } else {
            current_value
        };
        let text_color = if is_empty {
            on_surface_variant
        } else {
            on_surface
        };
        let text_color = if self.disabled {
            Vec4::new(text_color.x, text_color.y, text_color.z, DISABLED_ALPHA)
        } else {
            text_color
        };
        let text_style = VisualStyle::new()
            .solid_fill(text_color)
            .text(TextContent::new(display_text, BODY_LARGE_SIZE));
        let text_node = ctx.create_node(
            input_container,
            NodeContent::Styled {
                style: Box::new(text_style),
            },
        );
        let _ = text_node;

        // -- Supporting text (error takes priority over helper) --
        if let Some(ref error_txt) = self.error_text {
            let error_style = VisualStyle::new()
                .solid_fill(error)
                .text(TextContent::new(error_txt.clone(), BODY_SMALL_SIZE));
            let error_node = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(error_style),
                },
            );
            let _ = error_node;
        } else if let Some(ref helper_txt) = self.helper_text {
            let helper_style = VisualStyle::new()
                .solid_fill(on_surface_variant)
                .text(TextContent::new(helper_txt.clone(), BODY_SMALL_SIZE));
            let helper_node = ctx.create_node(
                root,
                NodeContent::Styled {
                    style: Box::new(helper_style),
                },
            );
            let _ = helper_node;
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;
    use render_engine::Paint;

    #[test]
    fn test_outlined_default() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, String::new());

        let mut ctx = WidgetContext::new_test();
        let input = MaterialTextInput::new(signal);
        let root_id = input.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "MaterialTextInput root should exist"
        );

        // Root is a column container
        let root_node = ctx.scene().get_node(root_id).unwrap();
        // Should have at least the input container child
        assert!(
            !root_node.children.is_empty(),
            "MaterialTextInput should have children"
        );

        // The first child is the input container (no label)
        let input_container_id = root_node.children[0];
        let input_node = ctx.scene().get_node(input_container_id).unwrap();
        if let NodeContent::Styled { ref style } = input_node.content {
            // Outlined variant should have a stroke
            assert!(
                style.stroke.is_some(),
                "Outlined text input should have a stroke border"
            );
        } else {
            panic!("Input container should be Styled content");
        }
    }

    #[test]
    fn test_filled_variant() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, String::new());

        let mut ctx = WidgetContext::new_test();
        let input = MaterialTextInput::new(signal).filled();
        let root_id = input.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        let input_container_id = root_node.children[0];
        let input_node = ctx.scene().get_node(input_container_id).unwrap();
        if let NodeContent::Styled { ref style } = input_node.content {
            // Filled variant should NOT have a stroke
            assert!(
                style.stroke.is_none(),
                "Filled text input should not have a stroke"
            );
            // Should have surface_variant fill
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE_VARIANT.x).abs() < 0.01,
                    "Filled input should use surface_variant fill, got {color:?}"
                );
            }
        } else {
            panic!("Input container should be Styled content");
        }
    }

    #[test]
    fn test_with_label() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, String::new());

        let mut ctx = WidgetContext::new_test();
        let input = MaterialTextInput::new(signal).label("Email");
        let root_id = input.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // With label: children are [label, input_container]
        assert!(
            root_node.children.len() >= 2,
            "Text input with label should have at least 2 children (label + input), got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_with_error() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, String::new());

        let mut ctx = WidgetContext::new_test();
        let input = MaterialTextInput::new(signal).error_text("Invalid email");
        let root_id = input.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // Without label: children are [input_container, error_text]
        assert!(
            root_node.children.len() >= 2,
            "Text input with error should have at least 2 children (input + error), got {}",
            root_node.children.len()
        );

        // The error text node should use error color
        let error_id = root_node.children[root_node.children.len() - 1];
        let error_node = ctx.scene().get_node(error_id).unwrap();
        if let NodeContent::Styled { ref style } = error_node.content
            && let Paint::Solid(color) = &style.fills[0]
        {
            assert!(
                (color.x - FALLBACK_ERROR.x).abs() < 0.01,
                "Error text should use error color, got {color:?}"
            );
        }
    }
}
