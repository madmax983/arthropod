//! Button widget - interactive clickable button

use crate::WidgetEnum;
use crate::{style, DesignTokens, Style};
use crate::{Text, Widget, WidgetContext};
use glam::Vec4;
use render_engine::{Color, NodeContent, NodeId};
use std::sync::Arc;

/// Button widget with hover and click interactions.
///
/// Styles are defined by the [`ButtonStyle`] enum and can be applied via helper methods.
/// Uses the unified [`crate::Style`] system for automatic hover effects.
///
/// # Example
///
/// ```no_run
/// use widget_core::{Button, btn, ButtonStyle};
///
/// // Builder pattern
/// let button = Button::new("Click Me")
///     .primary()
///     .on_click(|| println!("Clicked!"));
///
/// // Macro usage
/// let b1 = btn!("Click Me"); // Default style
/// let b2 = btn!("Save", primary, on_click: || println!("Saved")); // Primary style
/// let b3 = btn!("Cancel", secondary, disabled, padding: 20.0); // Secondary style
/// ```
#[derive(Widget)]
#[widget(name = "btn", alias = "button")]
pub struct Button {
    #[positional]
    text: String,

    #[callback]
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,

    #[method_flag(primary, secondary)]
    style_variant: ButtonStyle,

    #[param(default = 12.0)]
    padding: f32,

    #[flag]
    disabled: bool,

    #[param]
    style: Option<crate::Style>,
}

/// Visual styling tier for a `Button`.
///
/// Use styles to enforce visual hierarchy and guide users to the most important actions.
///
/// ## Examples
///
/// ```rust,no_run
/// # use widget_core::{btn, ButtonStyle};
/// // The primary action on a screen (e.g., "Submit" or "Save").
/// let submit = btn!("Save", primary);
///
/// // Alternative actions (e.g., "Cancel" or "Back").
/// let cancel = btn!("Cancel", secondary);
/// ```
#[derive(Clone, Copy, Default, WidgetEnum)]
pub enum ButtonStyle {
    /// The highest emphasis style, rendered with the active system accent color.
    /// Use this for the main intended action on a screen.
    #[flag]
    Primary,
    /// A lower emphasis style, rendered with the secondary surface color.
    /// Use this for alternative or optional actions.
    #[flag]
    Secondary,
    /// The base, neutral style, rendered with a light surface color.
    #[default]
    Default,
}

impl Button {
    /// Create a new button with text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            on_click: None,
            style_variant: ButtonStyle::Default,
            padding: 12.0,
            disabled: false,
            style: None,
        }
    }

    /// Set click callback
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(callback));
        self
    }

    /// Use primary style (themed accent color)
    pub fn primary(mut self) -> Self {
        self.style_variant = ButtonStyle::Primary;
        self
    }

    /// Use secondary style
    pub fn secondary(mut self) -> Self {
        self.style_variant = ButtonStyle::Secondary;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set a high-level style override
    pub fn style(mut self, style: crate::Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Create high-level style for the button
    fn create_style(&self, tokens: Option<&DesignTokens>) -> Style {
        if let Some(style) = &self.style {
            return style.clone();
        }

        match tokens {
            Some(t) => {
                let (bg, hover_bg, pressed_bg, text_color) = match self.style_variant {
                    ButtonStyle::Primary => (
                        t.accent,
                        t.accent_hover,
                        t.accent_pressed,
                        Vec4::new(1.0, 1.0, 1.0, 1.0),
                    ),
                    ButtonStyle::Secondary => (
                        t.surface_secondary.as_color(),
                        t.surface_elevated.as_color(),
                        t.surface_secondary.as_color(),
                        t.text_primary,
                    ),
                    ButtonStyle::Default => (
                        t.surface_secondary.as_color(),
                        t.surface_elevated.as_color(),
                        t.surface_secondary.as_color(),
                        t.text_primary,
                    ),
                };

                style! {
                    background: bg;
                    color: text_color;
                    border_radius: t.radius_md;
                    padding: crate::StylePadding::symmetric(self.padding / 2.0, self.padding);
                    direction: layout_engine::FlexDirection::Row;
                    justify_content: layout_engine::FlexJustifyContent::Center;
                    align_items: layout_engine::FlexAlign::Center;

                    &:hover {
                        background: hover_bg;
                    }

                    &:active {
                        background: pressed_bg;
                    }

                    &:disabled {
                        opacity: 0.5;
                    }
                }
            }
            None => {
                // Fallback style without tokens
                let bg = match self.style_variant {
                    ButtonStyle::Primary => Vec4::new(0.0, 0.47, 0.84, 1.0),
                    ButtonStyle::Secondary => Vec4::new(0.5, 0.5, 0.5, 1.0),
                    ButtonStyle::Default => Vec4::new(0.9, 0.9, 0.9, 1.0),
                };

                style! {
                    background: bg;
                    border_radius: 6.0;
                    padding: self.padding;
                    direction: layout_engine::FlexDirection::Row;

                    &:hover {
                        opacity: 0.9;
                    }

                    &:disabled {
                        opacity: 0.5;
                    }
                }
            }
        }
    }

    /// Get text color for current style (used for child text widget)
    fn get_text_color(&self, tokens: Option<&DesignTokens>) -> Vec4 {
        if let Some(style) = &self.style {
            if let Some(color) = style.color {
                return color;
            }
        }

        match tokens {
            Some(t) => match self.style_variant {
                ButtonStyle::Primary => Vec4::new(1.0, 1.0, 1.0, 1.0),
                ButtonStyle::Secondary => t.text_primary,
                ButtonStyle::Default => t.text_primary,
            },
            None => match self.style_variant {
                ButtonStyle::Primary => Vec4::new(1.0, 1.0, 1.0, 1.0),
                ButtonStyle::Secondary => Vec4::new(1.0, 1.0, 1.0, 1.0),
                ButtonStyle::Default => Vec4::new(0.0, 0.0, 0.0, 1.0),
            },
        }
    }
}

impl Widget for Button {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let tokens = ctx.design_tokens();
        let button_style = self.create_style(tokens);
        let text_color = self.get_text_color(tokens);

        // 1. Create button container node (initially empty content, will be styled by system)
        let button_node = ctx.create_node(ctx.root(), NodeContent::Empty);

        // 2. Apply Unified Style
        ctx.set_widget_style(button_node, button_style.clone());
        let resolved = button_style.resolve(false, false, false, false);
        ctx.apply_style(button_node, &resolved);
        ctx.add_hover_state(button_node);

        // 3. Create and re-parent text child
        let text_widget = Text::new(self.text.clone()).color(Color::rgba(
            text_color.x,
            text_color.y,
            text_color.z,
            text_color.w,
        ));
        let text_id = text_widget.build(ctx);
        ctx.reparent_to(text_id, button_node);

        // 4. Add interaction components
        if let Some(callback) = &self.on_click {
            if !self.disabled {
                ctx.add_clickable(button_node, callback.clone());
            }
        }

        button_node
    }
}
