//! Text widget - displays shaped text

use crate::{Widget, WidgetContext};
use render_engine::{NodeId, NodeContent, node::{ShapedTextData, ShapedGlyphData}, Color};
use flux_state::ReadSignal;
use text_engine::TextEngine;
use glam::Vec4;

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
/// ```
pub struct Text {
    content: TextContent,
    font_size: f32,
    color: Vec4,
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
            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    /// Create a text widget with reactive content
    pub fn reactive(signal: ReadSignal<String>) -> Self {
        Self {
            content: TextContent::Reactive(signal),
            font_size: 16.0,
            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    /// Set font size
    pub fn size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set text color
    pub fn color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }
}

impl Widget for Text {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Get text content
        let text_string = match &self.content {
            TextContent::Static(s) => s.clone(),
            TextContent::Reactive(signal) => {
                // Get untracked value for initial build
                signal.get_untracked()
            }
        };

        // Shape the text
        let mut text_engine = TextEngine::new();
        let shaped = text_engine.shape_text(&text_string, self.font_size);

        // Convert to serializable format
        let shaped_data = ShapedTextData {
            glyphs: shaped.glyphs.iter().map(|g| ShapedGlyphData {
                glyph_id: g.glyph_id,
                x_offset: g.x_offset,
                y_offset: g.y_offset,
                x_advance: g.x_advance,
                y_advance: g.y_advance,
            }).collect(),
            bounds_width: shaped.bounds.width,
            bounds_height: shaped.bounds.height,
        };

        // Create text node
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Text {
                shaped_text: shaped_data,
                color: Color::rgba(self.color.x, self.color.y, self.color.z, self.color.w),
            },
        );

        // If reactive, store signal for updates
        // (This would be done via ECS components in full implementation)

        node_id
    }
}
