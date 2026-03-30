//! MD3 MaterialIcon -- icon rendered as styled text with optional color.

use crate::theme::MaterialTheme;
use glam::Vec4;
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 on_surface_variant (#49454F)
const FALLBACK_ON_SURFACE_VARIANT: Vec4 = Vec4::new(0.286, 0.271, 0.310, 1.0);

/// Default icon size in dp (MD3).
const DEFAULT_ICON_SIZE: f32 = 24.0;

// ---------------------------------------------------------------------------
// MaterialIcon
// ---------------------------------------------------------------------------

/// MD3 Icon rendered as styled text.
///
/// Displays a single text glyph at 24dp by default with on_surface_variant
/// color. Supports custom size and color override.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::components::MaterialIcon;
///
/// let icon = MaterialIcon::new("\u{2699}").size(32.0);
/// let search = MaterialIcon::search();
/// ```
pub struct MaterialIcon {
    icon: String,
    size: f32,
    color: Option<Vec4>,
}

impl MaterialIcon {
    /// Create a new icon from the given text/glyph.
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            size: DEFAULT_ICON_SIZE,
            color: None,
        }
    }

    /// Override the icon size in dp (default: 24.0).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Override the icon color.
    pub fn color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }

    /// Convenience: magnifying glass icon.
    pub fn search() -> Self {
        Self::new("\u{1F50D}")
    }

    /// Convenience: house icon.
    pub fn home() -> Self {
        Self::new("\u{1F3E0}")
    }

    /// Convenience: gear icon.
    pub fn settings() -> Self {
        Self::new("\u{2699}")
    }

    /// Convenience: check mark icon.
    pub fn check() -> Self {
        Self::new("\u{2713}")
    }

    /// Convenience: close/X icon.
    pub fn close() -> Self {
        Self::new("\u{2715}")
    }

    /// Convenience: hamburger menu icon.
    pub fn menu() -> Self {
        Self::new("\u{2630}")
    }
}

impl Widget for MaterialIcon {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve icon color --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let icon_color = self.color.unwrap_or_else(|| {
            theme
                .as_ref()
                .map(|t| t.color.on_surface_variant)
                .unwrap_or(FALLBACK_ON_SURFACE_VARIANT)
        });

        // -- Build styled text node --
        let style = VisualStyle::new()
            .solid_fill(icon_color)
            .text(TextContent::new(self.icon.clone(), self.size));

        ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(style),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;

    #[test]
    fn test_default_size() {
        let mut ctx = WidgetContext::new_test();
        let icon = MaterialIcon::new("\u{2699}");
        let root_id = icon.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            let text = style.text.as_ref().expect("Icon should have text content");
            assert!(
                (text.font_size - DEFAULT_ICON_SIZE).abs() < 0.01,
                "Default icon size should be {DEFAULT_ICON_SIZE}, got {}",
                text.font_size
            );
        } else {
            panic!("Icon should be Styled content");
        }
    }

    #[test]
    fn test_custom_color() {
        let custom_color = Vec4::new(1.0, 0.0, 0.0, 1.0);

        let mut ctx = WidgetContext::new_test();
        let icon = MaterialIcon::new("\u{2699}").color(custom_color);
        let root_id = icon.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - 1.0).abs() < 0.01 && color.y.abs() < 0.01,
                    "Icon should use custom red color, got {color:?}"
                );
            }
        } else {
            panic!("Icon should be Styled content");
        }
    }

    #[test]
    fn test_convenience_constructors() {
        let mut ctx = WidgetContext::new_test();

        // Test all convenience constructors build successfully
        let icons = [
            MaterialIcon::search(),
            MaterialIcon::home(),
            MaterialIcon::settings(),
            MaterialIcon::check(),
            MaterialIcon::close(),
            MaterialIcon::menu(),
        ];

        for icon in &icons {
            let root_id = icon.build(&mut ctx);
            assert!(
                ctx.scene().get_node(root_id).is_some(),
                "Convenience icon should build successfully"
            );
        }
    }
}
