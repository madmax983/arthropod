//! MD3 Skeleton widget -- loading placeholder.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::FlexStyle;
use render_engine::node::NodeContent;
use render_engine::{NodeId, VisualStyle};
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 surface_variant (#E7E0EC) -- skeleton placeholder fill.
const FALLBACK_SURFACE_VARIANT: Vec4 = Vec4::new(0.906, 0.878, 0.925, 1.0);

/// Default width for text skeleton in dp.
const DEFAULT_TEXT_WIDTH: f32 = 200.0;

/// Default height for text skeleton in dp.
const DEFAULT_TEXT_HEIGHT: f32 = 20.0;

/// Corner radius for text variant in dp.
const TEXT_CORNER_RADIUS: f32 = 4.0;

/// Corner radius for rounded variant in dp.
const ROUNDED_CORNER_RADIUS: f32 = 8.0;

/// Skeleton shape variant following MD3 placeholder patterns.
///
/// Each variant determines the corner radius and default dimensions of the
/// loading placeholder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkeletonVariant {
    /// Rounded rectangle, 1-line text height. Corner radius = 4dp.
    Text,
    /// Circle. Corner radius = size / 2.
    Circular,
    /// Sharp-cornered rectangle. Corner radius = 0.
    Rectangular,
    /// Slightly rounded rectangle. Corner radius = 8dp.
    Rounded,
}

/// MD3 Skeleton -- loading placeholder.
///
/// Renders a simple filled shape (using `surface_variant` color) to indicate
/// content that is still loading. The shape varies by [`SkeletonVariant`].
///
/// Pulsing animation is planned for a future release once the animation
/// system is available.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::data_display::{Skeleton, SkeletonVariant};
///
/// // Text placeholder
/// let text_skel = Skeleton::text().width(300.0);
///
/// // Circular avatar placeholder
/// let avatar_skel = Skeleton::circular(48.0);
///
/// // Rectangular image placeholder
/// let img_skel = Skeleton::rectangular(200.0, 150.0);
///
/// // Rounded card placeholder
/// let card_skel = Skeleton::rounded(300.0, 100.0);
/// ```
pub struct Skeleton {
    variant: SkeletonVariant,
    width: f32,
    height: f32,
}

impl Skeleton {
    /// Create a text skeleton placeholder (width=200, height=20, 4dp corners).
    pub fn text() -> Self {
        Self {
            variant: SkeletonVariant::Text,
            width: DEFAULT_TEXT_WIDTH,
            height: DEFAULT_TEXT_HEIGHT,
        }
    }

    /// Create a circular skeleton placeholder (size x size, fully rounded).
    pub fn circular(size: f32) -> Self {
        Self {
            variant: SkeletonVariant::Circular,
            width: size,
            height: size,
        }
    }

    /// Create a rectangular skeleton placeholder (sharp corners).
    pub fn rectangular(width: f32, height: f32) -> Self {
        Self {
            variant: SkeletonVariant::Rectangular,
            width,
            height,
        }
    }

    /// Create a rounded skeleton placeholder (8dp corners).
    pub fn rounded(width: f32, height: f32) -> Self {
        Self {
            variant: SkeletonVariant::Rounded,
            width,
            height,
        }
    }

    /// Override the width in dp.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Override the height in dp.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
}

impl Widget for Skeleton {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let fill = theme
            .as_ref()
            .map(|t| t.color.surface_variant)
            .unwrap_or(FALLBACK_SURFACE_VARIANT);

        // -- Determine corner radius by variant --
        let corner_radius = match self.variant {
            SkeletonVariant::Text => TEXT_CORNER_RADIUS,
            SkeletonVariant::Circular => self.width.min(self.height) / 2.0,
            SkeletonVariant::Rectangular => 0.0,
            SkeletonVariant::Rounded => ROUNDED_CORNER_RADIUS,
        };

        // -- Build the placeholder node --
        let style = VisualStyle::new()
            .solid_fill(fill)
            .corner_radius(corner_radius);

        let node = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(style),
            },
        );
        ctx.set_layout_style(
            node,
            FlexStyle {
                width: Some(self.width),
                height: Some(self.height),
                ..Default::default()
            },
        );

        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;

    #[test]
    fn test_skeleton_text_variant() {
        let mut ctx = WidgetContext::new_test();
        let skel = Skeleton::text();
        let node_id = skel.build(&mut ctx);

        let node = ctx.scene().get_node(node_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(!style.fills.is_empty(), "Skeleton should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE_VARIANT.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE_VARIANT.y).abs() < 0.01
                        && (color.z - FALLBACK_SURFACE_VARIANT.z).abs() < 0.01,
                    "Skeleton fill should match surface_variant, got {color:?}"
                );
            } else {
                panic!("Skeleton fill should be a solid paint");
            }
            // Text variant should have 4dp corner radius
            assert!(
                (style.corner_radii.top_left - TEXT_CORNER_RADIUS).abs() < 0.01,
                "Text skeleton corner radius should be {}dp, got {}",
                TEXT_CORNER_RADIUS,
                style.corner_radii.top_left
            );
        } else {
            panic!("Skeleton node should be Styled content");
        }
    }

    #[test]
    fn test_skeleton_circular_variant() {
        let mut ctx = WidgetContext::new_test();
        let skel = Skeleton::circular(48.0);
        let node_id = skel.build(&mut ctx);

        let node = ctx.scene().get_node(node_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // Circular variant should have corner radius = size / 2
            let expected_radius = 24.0;
            assert!(
                (style.corner_radii.top_left - expected_radius).abs() < 0.01,
                "Circular skeleton corner radius should be {expected_radius}dp, got {}",
                style.corner_radii.top_left
            );
        } else {
            panic!("Skeleton node should be Styled content");
        }
    }

    #[test]
    fn test_skeleton_rectangular_variant() {
        let mut ctx = WidgetContext::new_test();
        let skel = Skeleton::rectangular(100.0, 80.0);
        let node_id = skel.build(&mut ctx);

        let node = ctx.scene().get_node(node_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // Rectangular variant should have 0 corner radius
            assert!(
                style.corner_radii.top_left.abs() < 0.01,
                "Rectangular skeleton corner radius should be 0, got {}",
                style.corner_radii.top_left
            );
        } else {
            panic!("Skeleton node should be Styled content");
        }
    }

    #[test]
    fn test_skeleton_custom_dimensions() {
        let skel = Skeleton::text().width(400.0).height(32.0);
        assert!(
            (skel.width - 400.0).abs() < 0.01,
            "Width should be 400.0, got {}",
            skel.width
        );
        assert!(
            (skel.height - 32.0).abs() < 0.01,
            "Height should be 32.0, got {}",
            skel.height
        );

        // Also verify it builds successfully
        let mut ctx = WidgetContext::new_test();
        let node_id = skel.build(&mut ctx);
        assert!(
            ctx.scene().get_node(node_id).is_some(),
            "Custom-sized skeleton should build successfully"
        );
    }
}
