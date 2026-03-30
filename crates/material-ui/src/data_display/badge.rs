//! MD3 Badge widget -- count or dot indicator overlaid on a target widget.

use glam::Vec4;
use layout_engine::{FlexAlign, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, TextContent, VisualStyle};
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 error color (#B3261E) -- badge background.
const FALLBACK_ERROR: Vec4 = Vec4::new(0.702, 0.149, 0.118, 1.0);

/// MD3 on_error color (#FFFFFF) -- badge text.
const FALLBACK_ON_ERROR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

/// Dot badge diameter in dp.
const DOT_SIZE: f32 = 6.0;

/// Minimum count badge size in dp.
const MIN_COUNT_SIZE: f32 = 16.0;

/// Count badge font size in dp.
const COUNT_FONT_SIZE: f32 = 11.0;

/// Badge content -- either a simple dot or a count number.
///
/// # Variants
///
/// * `Dot` -- small 6dp circle indicating presence (e.g., unread notification)
/// * `Count(u32)` -- numbered badge showing a count (e.g., message count)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BadgeContent {
    /// Small dot indicator (6dp circle).
    Dot,
    /// Numeric count indicator.
    Count(u32),
}

/// MD3 Badge -- count or dot indicator overlaid on a target widget.
///
/// Wraps a target widget and renders a small badge indicator in the
/// top-right corner. The badge can be either a small dot or a numbered
/// count pill.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::data_display::Badge;
/// use widget_core::Text;
///
/// // Dot badge on an icon
/// let dot = Badge::dot(Text::new("Mail"));
///
/// // Count badge
/// let count = Badge::count(5, Text::new("Mail"));
/// ```
pub struct Badge {
    content: BadgeContent,
    target: Box<dyn Widget>,
}

impl Badge {
    /// Create a dot badge overlaid on the given target widget.
    pub fn dot(target: impl Widget + 'static) -> Self {
        Self {
            content: BadgeContent::Dot,
            target: Box::new(target),
        }
    }

    /// Create a count badge overlaid on the given target widget.
    pub fn count(count: u32, target: impl Widget + 'static) -> Self {
        Self {
            content: BadgeContent::Count(count),
            target: Box::new(target),
        }
    }
}

impl Widget for Badge {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Root container (holds target + badge indicator) --
        let root_style = VisualStyle::new();
        let root = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(root_style),
            },
        );

        // -- Build target widget and reparent to root --
        let target_id = self.target.build(ctx);
        ctx.reparent_to(target_id, root);

        // -- Create badge indicator --
        match &self.content {
            BadgeContent::Dot => {
                let dot_style = VisualStyle::new()
                    .solid_fill(FALLBACK_ERROR)
                    .corner_radius(DOT_SIZE / 2.0);

                let dot_node = ctx.create_node(
                    root,
                    NodeContent::Styled {
                        style: Box::new(dot_style),
                    },
                );
                ctx.set_layout_style(
                    dot_node,
                    FlexStyle {
                        width: Some(DOT_SIZE),
                        height: Some(DOT_SIZE),
                        ..Default::default()
                    },
                );
            }
            BadgeContent::Count(count) => {
                let count_text = count.to_string();
                // Wider pill for multi-digit numbers
                let badge_width = if count_text.len() > 1 {
                    MIN_COUNT_SIZE + (count_text.len() as f32 - 1.0) * 6.0
                } else {
                    MIN_COUNT_SIZE
                };

                let badge_style = VisualStyle::new()
                    .solid_fill(FALLBACK_ERROR)
                    .corner_radius(MIN_COUNT_SIZE / 2.0);

                let badge_node = ctx.create_node(
                    root,
                    NodeContent::Styled {
                        style: Box::new(badge_style),
                    },
                );
                ctx.set_layout_style(
                    badge_node,
                    FlexStyle {
                        width: Some(badge_width),
                        height: Some(MIN_COUNT_SIZE),
                        justify_content: FlexJustifyContent::Center,
                        align_items: FlexAlign::Center,
                        ..Default::default()
                    },
                );

                // -- Count text inside badge --
                let text_style = VisualStyle::new()
                    .solid_fill(FALLBACK_ON_ERROR)
                    .text(TextContent::new(count_text, COUNT_FONT_SIZE));

                ctx.create_node(
                    badge_node,
                    NodeContent::Styled {
                        style: Box::new(text_style),
                    },
                );
            }
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;

    #[test]
    fn test_badge_dot_builds() {
        let mut ctx = WidgetContext::new_test();
        let badge = Badge::dot(widget_core::Text::new("Mail"));
        let root_id = badge.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Badge dot root node should exist in scene"
        );
    }

    #[test]
    fn test_badge_count_builds() {
        let mut ctx = WidgetContext::new_test();
        let badge = Badge::count(5, widget_core::Text::new("Inbox"));
        let root_id = badge.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Badge count root node should exist in scene"
        );
    }

    #[test]
    fn test_badge_has_target_and_indicator() {
        let mut ctx = WidgetContext::new_test();
        let badge = Badge::dot(widget_core::Text::new("Icon"));
        let root_id = badge.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        assert!(
            root_node.children.len() >= 2,
            "Badge root should have at least 2 children (target + indicator), got {}",
            root_node.children.len()
        );
    }

    #[test]
    fn test_badge_count_text() {
        let mut ctx = WidgetContext::new_test();
        let badge = Badge::count(42, widget_core::Text::new("Notifications"));
        let root_id = badge.build(&mut ctx);

        let root_node = ctx.scene().get_node(root_id).unwrap();
        // Last child is the badge indicator container
        let badge_indicator_id = *root_node.children.last().unwrap();
        let badge_indicator = ctx.scene().get_node(badge_indicator_id).unwrap();

        // Badge indicator should have error color background
        if let NodeContent::Styled { ref style } = badge_indicator.content {
            assert!(!style.fills.is_empty(), "Badge should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_ERROR.x).abs() < 0.01
                        && (color.y - FALLBACK_ERROR.y).abs() < 0.01
                        && (color.z - FALLBACK_ERROR.z).abs() < 0.01,
                    "Badge fill should be error color, got {color:?}"
                );
            } else {
                panic!("Badge fill should be a solid paint");
            }
        } else {
            panic!("Badge indicator should be Styled content");
        }

        // Badge indicator should have a text child with the count
        assert!(
            !badge_indicator.children.is_empty(),
            "Count badge should have a text child"
        );
        let text_id = badge_indicator.children[0];
        let text_node = ctx.scene().get_node(text_id).unwrap();
        if let NodeContent::Styled { ref style } = text_node.content {
            let text = style
                .text
                .as_ref()
                .expect("Badge count text node should have TextContent");
            assert_eq!(text.text, "42", "Badge count text should display the count");
            assert!(
                (text.font_size - COUNT_FONT_SIZE).abs() < 0.01,
                "Badge count font size should be {}dp, got {}",
                COUNT_FONT_SIZE,
                text.font_size
            );
        } else {
            panic!("Badge text node should be Styled content");
        }
    }
}
