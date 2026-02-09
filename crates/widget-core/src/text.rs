//! Text widget - displays text

use crate::{Widget, WidgetContext};
use flux_state::{Computed, ReadSignal};
use layout_engine;
use render_engine::{Color, NodeContent, NodeId};

/// Text widget
///
/// Displays text with optional reactive updates.
///
/// # Example
///
/// ```no_run
/// use widget_core::Text;
/// use render_engine::Color;
///
/// // Static text
/// let text = Text::new("Hello World")
///     .size(20.0)
///     .color(Color::rgba(1.0, 1.0, 1.0, 1.0));
///
/// // With generated macro:
/// // txt!("Hello World")
/// // txt!("Title", size: 20.0)
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
    color: Option<Color>,
}

enum TextContent {
    Static(String),
    Reactive(ReadSignal<String>),
    Computed(Computed<String>),
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

    /// Create a text widget with computed content
    ///
    /// Computed values automatically update when their dependencies change.
    /// Use this when deriving text from other signals.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use arthropod::prelude::*;
    /// # use flux_state::Computed;
    /// # use widget_core::Text;
    /// # App::run("Test", 400, 300, |ctx| {
    /// let counter = ctx.signal(0);
    /// let (read, _write) = counter.split();
    ///
    /// // Text automatically updates when counter changes
    /// let text = Text::computed(Computed::new(ctx.runtime().clone(), move || {
    ///     format!("Count: {}", read.get())
    /// }));
    /// # text
    /// # });
    /// ```
    /// Create a text widget with computed reactive content
    ///
    /// Use this for derived text that automatically updates when dependencies change.
    /// Computed values are lazy (only recompute when read) and memoized (cache results).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use flux_state::{Runtime, Signal, Computed};
    /// use widget_core::Text;
    ///
    /// let runtime = Runtime::new();
    /// let counter = Signal::new(runtime.clone(), 0);
    /// let (read, _write) = counter.split();
    ///
    /// // Text automatically updates when counter changes
    /// let text = Computed::new(runtime.clone(), move || {
    ///     format!("Count: {}", read.get())
    /// });
    ///
    /// Text::computed(text);
    /// ```
    pub fn computed(computed: Computed<String>) -> Self {
        Self {
            content: TextContent::Computed(computed),
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
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    // Size presets

    /// Large heading text (32px)
    pub fn heading1(mut self) -> Self {
        self.font_size = 32.0;
        self
    }

    /// Medium heading text (24px)
    pub fn heading2(mut self) -> Self {
        self.font_size = 24.0;
        self
    }

    /// Small heading text (20px)
    pub fn heading3(mut self) -> Self {
        self.font_size = 20.0;
        self
    }

    /// Body text (16px, default)
    pub fn body(mut self) -> Self {
        self.font_size = 16.0;
        self
    }

    /// Caption text (12px)
    pub fn caption(mut self) -> Self {
        self.font_size = 12.0;
        self
    }
}

impl Widget for Text {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Resolve color: explicit > theme > hardcoded fallback
        let resolved_color = match self.color {
            Some(c) => c,
            None => {
                let theme_color = ctx
                    .design_tokens()
                    .map(|t| t.text_primary)
                    .unwrap_or(glam::Vec4::new(0.0, 0.0, 0.0, 1.0));
                Color::rgba(theme_color.x, theme_color.y, theme_color.z, theme_color.w)
            }
        };

        // Get text content
        let text_string = match &self.content {
            TextContent::Static(s) => s.clone(),
            TextContent::Reactive(signal) => {
                // Get untracked value for initial build
                signal.get_untracked()
            }
            TextContent::Computed(computed) => {
                // Get current computed value for initial build
                computed.get()
            }
        };

        // Create text node with raw text (backend will shape it during rendering)
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(resolved_color.as_vec4())
                        .text(render_engine::TextContent::new(text_string, self.font_size)),
                ),
            },
        );

        // Register reactive or computed text for ECS updates
        match &self.content {
            TextContent::Reactive(signal) => {
                ctx.add_reactive_text_state(node_id, signal.clone());
            }
            TextContent::Computed(computed) => {
                ctx.add_computed_text_state(node_id, computed.clone());
            }
            TextContent::Static(_) => {}
        }

        // Set layout style with estimated dimensions
        // Height is approximately font_size * 1.4 (accounts for line height)
        // Width is auto (will grow to fit content)
        let style = layout_engine::FlexStyle {
            height: Some(self.font_size * 1.4),
            ..Default::default()
        };
        ctx.set_layout_style(node_id, style);

        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_new() {
        let text = Text::new("Hello");
        assert_eq!(text.font_size, 16.0); // Default body size
    }

    #[test]
    fn test_text_size() {
        let text = Text::new("Hello").size(20.0);
        assert_eq!(text.font_size, 20.0);
    }

    #[test]
    fn test_text_heading1() {
        let text = Text::new("Heading").heading1();
        assert_eq!(text.font_size, 32.0);
    }

    #[test]
    fn test_text_heading2() {
        let text = Text::new("Heading").heading2();
        assert_eq!(text.font_size, 24.0);
    }

    #[test]
    fn test_text_heading3() {
        let text = Text::new("Heading").heading3();
        assert_eq!(text.font_size, 20.0);
    }

    #[test]
    fn test_text_body() {
        let text = Text::new("Body").body();
        assert_eq!(text.font_size, 16.0);
    }

    #[test]
    fn test_text_caption() {
        let text = Text::new("Caption").caption();
        assert_eq!(text.font_size, 12.0);
    }

    #[test]
    fn test_text_color() {
        let color = Color::rgba(1.0, 0.0, 0.0, 1.0);
        let text = Text::new("Colored").color(color);
        assert_eq!(text.color, Some(color));
    }

    #[test]
    fn test_text_chained_builders() {
        let color = Color::rgba(0.5, 0.5, 0.5, 1.0);
        let text = Text::new("Test").heading2().color(color);
        assert_eq!(text.font_size, 24.0);
        assert_eq!(text.color, Some(color));
    }
}
