//! MD3 Avatar widget -- circular display of initials or a single letter.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, VisualStyle};
use widget_core::widget_trait::Widget;
use widget_core::{Text, WidgetContext};

// ---------------------------------------------------------------------------
// MD3 fallback colors (used when no MaterialTheme is provided)
// ---------------------------------------------------------------------------

/// MD3 primary_container (#EADDFF)
const FALLBACK_PRIMARY_CONTAINER: Vec4 = Vec4::new(0.918, 0.867, 1.0, 1.0);

/// MD3 on_primary_container (#21005D)
const FALLBACK_ON_PRIMARY_CONTAINER: Vec4 = Vec4::new(0.129, 0.0, 0.365, 1.0);

/// Default avatar size in dp.
const DEFAULT_SIZE: f32 = 40.0;

/// MD3 Avatar -- circular display of initials or a letter.
///
/// Renders a circle with the primary container background color and centered
/// text in the on-primary-container color. Commonly used to represent users
/// or entities with their initials.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::data_display::Avatar;
///
/// let avatar = Avatar::new("JD")
///     .size(48.0);
/// ```
pub struct Avatar {
    text: String,
    size: f32,
    background: Option<Vec4>,
}

impl Avatar {
    /// Create a new avatar displaying the given text (typically initials).
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            size: DEFAULT_SIZE,
            background: None,
        }
    }

    /// Override the avatar size in dp (default: 40.0).
    ///
    /// The avatar is always rendered as a circle (corner_radius = size / 2).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Override the background color (default: primary_container from theme).
    pub fn background(mut self, color: Vec4) -> Self {
        self.background = Some(color);
        self
    }
}

impl Widget for Avatar {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary_container = theme
            .as_ref()
            .map(|t| t.color.primary_container)
            .unwrap_or(FALLBACK_PRIMARY_CONTAINER);
        let on_primary_container = theme
            .as_ref()
            .map(|t| t.color.on_primary_container)
            .unwrap_or(FALLBACK_ON_PRIMARY_CONTAINER);

        let bg_color = self.background.unwrap_or(primary_container);
        let corner_radius = self.size / 2.0;

        // -- Build circular container --
        let style = VisualStyle::new()
            .solid_fill(bg_color)
            .corner_radius(corner_radius);

        let container = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(style),
            },
        );
        ctx.set_layout_style(
            container,
            FlexStyle {
                width: Some(self.size),
                height: Some(self.size),
                justify_content: FlexJustifyContent::Center,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // -- Centered text child --
        let text_widget = Text::new(self.text.clone()).color(render_engine::Color::rgba(
            on_primary_container.x,
            on_primary_container.y,
            on_primary_container.z,
            on_primary_container.w,
        ));
        let text_id = text_widget.build(ctx);
        ctx.reparent_to(text_id, container);

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;

    #[test]
    fn test_avatar_builds() {
        let mut ctx = WidgetContext::new_test();
        let avatar = Avatar::new("A");
        let root_id = avatar.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Avatar root node should exist in scene"
        );
    }

    #[test]
    fn test_avatar_custom_size() {
        let mut ctx = WidgetContext::new_test();
        let avatar = Avatar::new("XY").size(64.0);
        let root_id = avatar.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // Corner radius should be size / 2 = 32.0
            assert!(
                (style.corner_radii.top_left - 32.0).abs() < 0.01,
                "Avatar corner radius should be half the size (32.0), got {}",
                style.corner_radii.top_left
            );
        } else {
            panic!("Avatar container should be Styled content");
        }
    }

    #[test]
    fn test_avatar_custom_background() {
        let custom_bg = Vec4::new(0.8, 0.2, 0.3, 1.0);
        let mut ctx = WidgetContext::new_test();
        let avatar = Avatar::new("M").background(custom_bg);
        let root_id = avatar.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(!style.fills.is_empty(), "Avatar should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - custom_bg.x).abs() < 0.01
                        && (color.y - custom_bg.y).abs() < 0.01
                        && (color.z - custom_bg.z).abs() < 0.01,
                    "Avatar background should match custom color, got {color:?}"
                );
            } else {
                panic!("Avatar fill should be a solid paint");
            }
        } else {
            panic!("Avatar container should be Styled content");
        }
    }

    #[test]
    fn test_avatar_default_circle_shape() {
        let mut ctx = WidgetContext::new_test();
        let avatar = Avatar::new("Z");
        let root_id = avatar.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // Default size=40, so corner_radius should be 20 (perfect circle)
            let expected_radius = DEFAULT_SIZE / 2.0;
            assert!(
                (style.corner_radii.top_left - expected_radius).abs() < 0.01,
                "Default avatar corner radius should be {} (circle), got {}",
                expected_radius,
                style.corner_radii.top_left
            );
            assert!(
                (style.corner_radii.top_right - expected_radius).abs() < 0.01,
                "All corners should have equal radius"
            );
            assert!(
                (style.corner_radii.bottom_right - expected_radius).abs() < 0.01,
                "All corners should have equal radius"
            );
            assert!(
                (style.corner_radii.bottom_left - expected_radius).abs() < 0.01,
                "All corners should have equal radius"
            );
        } else {
            panic!("Avatar container should be Styled content");
        }
    }
}
