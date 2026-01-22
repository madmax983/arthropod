//! Text widget - displays text

use crate::{Widget, WidgetContext};
use flux_state::ReadSignal;
use glam::Vec4;
use render_engine::{Color, NodeContent, NodeId};

/// Text widget
///
/// Displays text with optional reactive updates.
///
/// # Example
///
/// ```no_run
/// use widget_core::Text;
/// use glam::Vec4;
///
/// // Static text
/// let text = Text::new("Hello World")
///     .size(20.0)
///     .color(Vec4::ONE);
///
/// // With generated macro:
/// // txt!("Hello World")
/// // txt!("Title", size: 20.0, color: Vec4::ONE)
/// // txt!(@signal, size: 20.0)  // Reactive
/// ```
#[derive(Widget)]
#[widget(name = "txt")]
pub struct Text {
    #[positional(reactive)]
    content: TextContent,

    #[param(default = 16.0, setter = "size")]
    font_size: f32,

    #[param]
    color: Option<Vec4>,
}

enum TextContent {
    Static(String),
    Reactive(ReadSignal<String>),
}

impl Text {
    /// Create a new text widget with static content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: TextContent::Static(content.into()),
            font_size: 16.0,
            color: None, // Use theme default
        }
    }

    /// Create a text widget with reactive content
    pub fn reactive(signal: ReadSignal<String>) -> Self {
        Self {
            content: TextContent::Reactive(signal),
            font_size: 16.0,
            color: None, // Use theme default
        }
    }

    /// Set font size
    pub fn size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set text color
    pub fn color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }
}

impl Widget for Text {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Resolve color: explicit > theme > hardcoded fallback
        let resolved_color = match self.color {
            Some(c) => c,
            None => ctx
                .design_tokens()
                .map(|t| t.text_primary)
                .unwrap_or(Vec4::new(0.0, 0.0, 0.0, 1.0)),
        };

        // Get text content
        let text_string = match &self.content {
            TextContent::Static(s) => s.clone(),
            TextContent::Reactive(signal) => {
                // Get untracked value for initial build
                signal.get_untracked()
            }
        };

        // Create text node with raw text (backend will shape it during rendering)
        ctx.create_node(
            ctx.root(),
            NodeContent::Text {
                text: text_string,
                font_size: self.font_size,
                color: Color::rgba(
                    resolved_color.x,
                    resolved_color.y,
                    resolved_color.z,
                    resolved_color.w,
                ),
            },
        )
    }
}
