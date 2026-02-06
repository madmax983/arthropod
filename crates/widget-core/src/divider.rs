//! Divider widget - visual separator line

use crate::{Widget, WidgetContext};
use glam::Vec4;
use layout_engine::FlexStyle;
use render_engine::{Color, NodeContent, NodeId};

/// Divider widget for visual separation
///
/// Creates a thin line (horizontal or vertical) to visually separate content.
/// Respects theme colors by default.
///
/// # Example
///
/// ```no_run
/// use widget_core::{Divider, Column, Text};
///
/// // Horizontal divider (default)
/// let column = Column::new((
///     Text::new("Section 1"),
///     Divider::horizontal(),
///     Text::new("Section 2"),
/// ));
///
/// // Vertical divider in row
/// use widget_core::Row;
/// let row = Row::new((
///     Text::new("Left"),
///     Divider::vertical(),
///     Text::new("Right"),
/// ));
///
/// // Customized divider
/// let custom = Divider::horizontal()
///     .thickness(2.0)
///     .color(glam::Vec4::new(1.0, 0.0, 0.0, 1.0))  // Red
///     .margin(10.0);
/// ```
#[derive(Clone)]
pub struct Divider {
    orientation: Orientation,
    thickness: f32,
    color: Option<Vec4>,
    margin: f32,
}

/// Divider orientation
#[derive(Clone, Copy, PartialEq)]
enum Orientation {
    Horizontal,
    Vertical,
}

impl Divider {
    /// Create a horizontal divider (spans full width)
    pub fn horizontal() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            thickness: 1.0,
            color: None, // Use theme
            margin: 0.0,
        }
    }

    /// Create a vertical divider (spans full height)
    pub fn vertical() -> Self {
        Self {
            orientation: Orientation::Vertical,
            thickness: 1.0,
            color: None, // Use theme
            margin: 0.0,
        }
    }

    /// Set divider thickness in pixels
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Set divider color (overrides theme)
    pub fn color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }

    /// Set margin (space before and after divider)
    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }
}

impl Widget for Divider {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // Resolve color: explicit > theme > hardcoded fallback
        let resolved_color = match self.color {
            Some(c) => c,
            None => ctx
                .design_tokens()
                .map(|t| {
                    // Use surface_tertiary for subtle divider or create a border color
                    // For now use a semi-transparent version of text_secondary
                    let text = t.text_secondary;
                    Vec4::new(text.x, text.y, text.z, 0.3) // 30% opacity
                })
                .unwrap_or(Vec4::new(0.5, 0.5, 0.5, 0.3)), // Gray fallback
        };

        // Create divider node as a colored rectangle
        let root_id = ctx.root();
        let node_id = ctx.create_node(
            root_id,
            NodeContent::Rect {
                color: Color::rgba(
                    resolved_color.x,
                    resolved_color.y,
                    resolved_color.z,
                    resolved_color.w,
                ),
            },
        );

        // Configure layout based on orientation
        let style = match self.orientation {
            Orientation::Horizontal => FlexStyle {
                // Full width, fixed height
                height: Some(self.thickness),
                // Margins on top and bottom
                ..Default::default()
            },
            Orientation::Vertical => FlexStyle {
                // Fixed width, full height
                width: Some(self.thickness),
                // Margins on left and right
                ..Default::default()
            },
        };

        ctx.set_layout_style(node_id, style);

        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divider_horizontal() {
        let divider = Divider::horizontal();
        assert!(matches!(divider.orientation, Orientation::Horizontal));
        assert_eq!(divider.thickness, 1.0);
        assert_eq!(divider.margin, 0.0);
        assert_eq!(divider.color, None);
    }

    #[test]
    fn test_divider_vertical() {
        let divider = Divider::vertical();
        assert!(matches!(divider.orientation, Orientation::Vertical));
        assert_eq!(divider.thickness, 1.0);
        assert_eq!(divider.margin, 0.0);
        assert_eq!(divider.color, None);
    }

    #[test]
    fn test_divider_thickness() {
        let divider = Divider::horizontal().thickness(2.5);
        assert_eq!(divider.thickness, 2.5);
    }

    #[test]
    fn test_divider_color() {
        let color = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let divider = Divider::horizontal().color(color);
        assert_eq!(divider.color, Some(color));
    }

    #[test]
    fn test_divider_margin() {
        let divider = Divider::horizontal().margin(10.0);
        assert_eq!(divider.margin, 10.0);
    }

    #[test]
    fn test_divider_chained_builders() {
        let color = Vec4::new(0.5, 0.5, 0.5, 1.0);
        let divider = Divider::vertical()
            .thickness(3.0)
            .color(color)
            .margin(8.0);

        assert!(matches!(divider.orientation, Orientation::Vertical));
        assert_eq!(divider.thickness, 3.0);
        assert_eq!(divider.color, Some(color));
        assert_eq!(divider.margin, 8.0);
    }
}
