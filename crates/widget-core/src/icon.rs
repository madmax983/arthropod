//! Icon widget - displays a vector or font-based icon

use crate::{Widget, WidgetContext};
use layout_engine::FlexStyle;
use render_engine::{Color, NodeContent, NodeId};

/// Icon widget
///
/// Displays an icon from a font (like Material Symbols) or a static character.
///
/// # Example
///
/// ```no_run
/// use widget_core::Icon;
///
/// let widget = Icon::new("search").size(24.0);
/// ```
#[derive(crate::Widget)]
#[widget(name = "icon", skip_impl)]
pub struct Icon {
    #[positional]
    icon: String,

    #[param(default = 20.0)]
    size: f32,

    #[param]
    color: Option<Color>,
}

impl Icon {
    /// Create a new Icon with the given symbol or name
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            size: 20.0,
            color: None,
        }
    }

    /// Set the display size of the icon
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the color of the icon
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    // Common Material Symbols character mappings

    /// Create a "Search" icon (`\u{e8b6}`)
    pub fn search() -> Self {
        Self::new("\u{e8b6}")
    }

    /// Create a "Home" icon (`\u{e88a}`)
    pub fn home() -> Self {
        Self::new("\u{e88a}")
    }

    /// Create a "Settings" gear icon (`\u{e8b8}`)
    pub fn settings() -> Self {
        Self::new("\u{e8b8}")
    }

    /// Create a "Check" mark icon (`\u{e5ca}`)
    pub fn check() -> Self {
        Self::new("\u{e5ca}")
    }

    /// Create a "Close" (X) icon (`\u{e5cd}`)
    pub fn close() -> Self {
        Self::new("\u{e5cd}")
    }

    /// Create a hamburger "Menu" icon (`\u{e5d2}`)
    pub fn menu() -> Self {
        Self::new("\u{e5d2}")
    }
}

impl Widget for Icon {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        let color = self.color.unwrap_or(Color::BLACK).as_vec4();

        // Create text node for the icon symbol
        // NOTE: In a real app, we'd ensure the "Material Symbols" font is active.
        // For now, we'll use whatever font the backend defaults to.
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(render_engine::VisualStyle::new().solid_fill(color).text(
                    render_engine::TextContent::new(self.icon.clone(), self.size),
                )),
            },
        );

        // Fixed size for icons to ensure consistent layout
        ctx.set_layout_style(
            node_id,
            FlexStyle {
                width: Some(self.size),
                height: Some(self.size),
                ..Default::default()
            },
        );

        node_id
    }
}
