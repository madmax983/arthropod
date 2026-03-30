//! MD3 Link widget -- styled clickable text.

use crate::theme::MaterialTheme;
use glam::Vec4;
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use std::sync::Arc;
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors (used when no MaterialTheme is provided)
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4)
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 on_surface (#1D1B20)
const FALLBACK_ON_SURFACE: Vec4 = Vec4::new(0.114, 0.106, 0.125, 1.0);

/// Disabled content opacity multiplier.
const DISABLED_ALPHA: f32 = 0.38;

/// Default font size in dp.
const DEFAULT_FONT_SIZE: f32 = 14.0;

/// MD3 Link -- styled clickable text.
///
/// Renders a single text node colored with the primary theme color. When
/// clicked, the supplied callback is invoked. When disabled, the text is
/// rendered with reduced opacity using `on_surface` color.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::inputs::Link;
///
/// let link = Link::new("Learn more")
///     .on_click(|| println!("Navigate!"))
///     .font_size(16.0);
/// ```
pub struct Link {
    text: String,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    disabled: bool,
    font_size: f32,
}

impl Link {
    /// Create a new link with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            on_click: None,
            disabled: false,
            font_size: DEFAULT_FONT_SIZE,
        }
    }

    /// Set the callback invoked when the link is clicked.
    pub fn on_click(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(f));
        self
    }

    /// Disable interaction. The link renders with reduced opacity and ignores
    /// click events.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the font size in dp (default: 14.0).
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}

impl Widget for Link {
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

        // -- Determine text color --
        let text_color = if self.disabled {
            Vec4::new(on_surface.x, on_surface.y, on_surface.z, DISABLED_ALPHA)
        } else {
            primary
        };

        // -- Build styled text node --
        let style = VisualStyle::new()
            .solid_fill(text_color)
            .text(TextContent::new(self.text.clone(), self.font_size));

        let node = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(style),
            },
        );

        // -- Click handler --
        if !self.disabled
            && let Some(ref on_click) = self.on_click
        {
            let callback = Arc::clone(on_click);
            ctx.add_clickable(node, callback);
        }

        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_link_builds() {
        let mut ctx = WidgetContext::new_test();
        let link = Link::new("Click me");
        let root_id = link.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Link root node should exist in scene"
        );
    }

    #[test]
    fn test_link_primary_color() {
        let mut ctx = WidgetContext::new_test();
        let link = Link::new("Learn more");
        let root_id = link.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(
                !style.fills.is_empty(),
                "Link should have at least one fill"
            );
            if let Paint::Solid(color) = &style.fills[0] {
                // Should be primary fallback color
                assert!(
                    (color.x - FALLBACK_PRIMARY.x).abs() < 0.01
                        && (color.y - FALLBACK_PRIMARY.y).abs() < 0.01
                        && (color.z - FALLBACK_PRIMARY.z).abs() < 0.01,
                    "Link fill should match primary color, got {color:?}"
                );
            } else {
                panic!("Link fill should be a solid paint");
            }
        } else {
            panic!("Link node should be Styled content");
        }
    }

    #[test]
    fn test_link_on_click() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = Arc::clone(&clicked);

        let mut ctx = WidgetContext::new_test();
        let link = Link::new("Go").on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        });
        let root_id = link.build(&mut ctx);

        assert!(
            ctx.has_clickable(root_id),
            "Enabled link with on_click should be clickable"
        );

        ctx.trigger_click(root_id);
        assert!(
            clicked.load(Ordering::SeqCst),
            "Link click callback should have fired"
        );
    }

    #[test]
    fn test_link_disabled() {
        let mut ctx = WidgetContext::new_test();
        let link = Link::new("Disabled link").on_click(|| {}).disabled(true);
        let root_id = link.build(&mut ctx);

        assert!(
            !ctx.has_clickable(root_id),
            "Disabled link should not be clickable"
        );

        // Verify reduced alpha on fill
        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content
            && let Some(Paint::Solid(color)) = style.fills.first()
        {
            assert!(
                (color.w - DISABLED_ALPHA).abs() < 0.01,
                "Disabled link should have reduced alpha ({}), got {}",
                DISABLED_ALPHA,
                color.w
            );
        }
    }

    #[test]
    fn test_link_custom_font_size() {
        let mut ctx = WidgetContext::new_test();
        let link = Link::new("Big link").font_size(18.0);
        let root_id = link.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            let text = style.text.as_ref().expect("Link should have text content");
            assert!(
                (text.font_size - 18.0).abs() < 0.01,
                "Font size should be 18.0, got {}",
                text.font_size
            );
        } else {
            panic!("Link node should be Styled content");
        }
    }
}
