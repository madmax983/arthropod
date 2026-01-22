//! Button widget - interactive clickable button

use crate::WidgetEnum;
use crate::{Text, Widget, WidgetContext};
use glam::Vec4;
use layout_engine::FlexDirection;
use render_engine::{Color, NodeContent, NodeId};
use std::sync::Arc;
use theme_engine::DesignTokens;

/// Button widget with hover and click interactions
///
/// # Example
///
/// ```no_run
/// use widget_core::Button;
///
/// let button = Button::new("Click Me")
///     .primary()
///     .on_click(|| println!("Clicked!"));
///
/// // With generated macro:
/// // btn!("Click Me")
/// // btn!("Save", primary, on_click: || save())
/// // btn!("Cancel", disabled, padding: 20.0)
/// ```
#[derive(Widget)]
#[widget(name = "btn", alias = "button")]
pub struct Button {
    #[positional]
    text: String,

    #[callback]
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,

    #[method_flag(primary, secondary)]
    style: ButtonStyle,

    #[param(default = 12.0)]
    padding: f32,

    #[flag]
    disabled: bool,
}

/// Button visual style
#[derive(Clone, Copy, Default, WidgetEnum)]
pub enum ButtonStyle {
    #[flag]
    Primary,
    #[flag]
    Secondary,
    #[default]
    Default,
}

impl Button {
    /// Create a new button with text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            on_click: None,
            style: ButtonStyle::Default,
            padding: 12.0,
            disabled: false,
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
        self.style = ButtonStyle::Primary;
        self
    }

    /// Use secondary style
    pub fn secondary(mut self) -> Self {
        self.style = ButtonStyle::Secondary;
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

    /// Get background color for current style
    ///
    /// If design tokens are provided, uses themed colors:
    /// - Primary: System accent color
    /// - Secondary: Surface secondary color
    /// - Default: Lighter surface color
    fn get_background_color(&self, tokens: Option<&DesignTokens>) -> Vec4 {
        match tokens {
            Some(t) => match self.style {
                ButtonStyle::Primary => t.accent,
                ButtonStyle::Secondary => {
                    // Use a slightly darker surface for secondary
                    let surface = t.surface_secondary.as_color();
                    Vec4::new(
                        surface.x * 0.85,
                        surface.y * 0.85,
                        surface.z * 0.85,
                        surface.w,
                    )
                }
                ButtonStyle::Default => t.surface_secondary.as_color(),
            },
            // Fallback to hardcoded values if no tokens
            None => match self.style {
                ButtonStyle::Primary => Vec4::new(0.0, 0.47, 0.84, 1.0), // Blue
                ButtonStyle::Secondary => Vec4::new(0.5, 0.5, 0.5, 1.0), // Gray
                ButtonStyle::Default => Vec4::new(0.9, 0.9, 0.9, 1.0),   // Light gray
            },
        }
    }

    /// Get text color for current style
    ///
    /// If design tokens are provided, uses themed colors:
    /// - Primary: White (high contrast on accent)
    /// - Secondary/Default: text_primary from tokens
    fn get_text_color(&self, tokens: Option<&DesignTokens>) -> Vec4 {
        match tokens {
            Some(t) => match self.style {
                ButtonStyle::Primary => Vec4::new(1.0, 1.0, 1.0, 1.0), // White on accent
                ButtonStyle::Secondary => t.text_primary,
                ButtonStyle::Default => t.text_primary,
            },
            // Fallback to hardcoded values if no tokens
            None => match self.style {
                ButtonStyle::Primary => Vec4::new(1.0, 1.0, 1.0, 1.0), // White on primary
                ButtonStyle::Secondary => Vec4::new(1.0, 1.0, 1.0, 1.0), // White on secondary
                ButtonStyle::Default => Vec4::new(0.0, 0.0, 0.0, 1.0), // Black on default
            },
        }
    }
}

impl Widget for Button {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Get colors from design tokens (must be done before mutable ctx operations)
        // We extract colors first to avoid holding a reference across mutable borrows
        let (bg_color, text_color) = {
            let tokens = ctx.design_tokens();
            (
                self.get_background_color(tokens),
                self.get_text_color(tokens),
            )
        };

        // Create button container with background
        let button_node = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::rgba(bg_color.x, bg_color.y, bg_color.z, bg_color.w),
            },
        );

        // Create text child
        let text_widget = Text::new(self.text.clone()).color(text_color);
        let text_id = text_widget.build(ctx);

        // Re-parent text to button
        let root_id = ctx.root();
        ctx.reparent_node(text_id, root_id, button_node);

        // Configure layout
        let layout_style = layout_engine::FlexStyle {
            direction: FlexDirection::Row,
            padding_left: self.padding,
            padding_right: self.padding,
            padding_top: self.padding / 2.0,
            padding_bottom: self.padding / 2.0,
            ..Default::default()
        };

        ctx.set_layout_style(button_node, layout_style);

        // Add interaction components
        ctx.add_hover_state(button_node);

        if let Some(callback) = &self.on_click {
            if !self.disabled {
                ctx.add_clickable(button_node, callback.clone());
            }
        }

        // Store background color for style checks
        ctx.set_background_color(button_node, bg_color);

        button_node
    }
}
