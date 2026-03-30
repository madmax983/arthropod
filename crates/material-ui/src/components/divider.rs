//! MD3 MaterialDivider -- full-width, inset, or middle-inset divider line.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::FlexStyle;
use render_engine::node::NodeContent;
use render_engine::{NodeId, VisualStyle};
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 outline_variant (#CAC4D0)
const FALLBACK_OUTLINE_VARIANT: Vec4 = Vec4::new(0.792, 0.769, 0.812, 1.0);

/// Divider thickness in dp.
const DIVIDER_THICKNESS: f32 = 1.0;

/// Inset margin in dp (MD3 standard).
const INSET_MARGIN: f32 = 16.0;

// ---------------------------------------------------------------------------
// DividerVariant
// ---------------------------------------------------------------------------

/// MD3 divider inset variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DividerVariant {
    /// Full-width divider (no margins).
    #[default]
    FullWidth,
    /// 16dp left margin.
    Inset,
    /// 16dp margin on both sides.
    MiddleInset,
}

// ---------------------------------------------------------------------------
// MaterialDivider
// ---------------------------------------------------------------------------

/// MD3 Divider -- 1dp line in outline_variant color.
///
/// Supports horizontal and vertical orientations with full-width, inset, and
/// middle-inset margin variants.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::components::MaterialDivider;
///
/// let h = MaterialDivider::horizontal();
/// let inset = MaterialDivider::horizontal().inset();
/// let v = MaterialDivider::vertical();
/// ```
pub struct MaterialDivider {
    variant: DividerVariant,
    vertical: bool,
}

impl MaterialDivider {
    /// Create a horizontal divider (full-width by default).
    pub fn horizontal() -> Self {
        Self {
            variant: DividerVariant::FullWidth,
            vertical: false,
        }
    }

    /// Create a vertical divider (full-width by default).
    pub fn vertical() -> Self {
        Self {
            variant: DividerVariant::FullWidth,
            vertical: true,
        }
    }

    /// Switch to the inset variant (16dp left margin).
    pub fn inset(mut self) -> Self {
        self.variant = DividerVariant::Inset;
        self
    }

    /// Switch to the middle-inset variant (16dp margins on both sides).
    pub fn middle_inset(mut self) -> Self {
        self.variant = DividerVariant::MiddleInset;
        self
    }
}

impl Widget for MaterialDivider {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme color --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let outline_variant = theme
            .as_ref()
            .map(|t| t.color.outline_variant)
            .unwrap_or(FALLBACK_OUTLINE_VARIANT);

        // -- Determine inset padding --
        let (pad_start, pad_end) = match self.variant {
            DividerVariant::FullWidth => (0.0, 0.0),
            DividerVariant::Inset => (INSET_MARGIN, 0.0),
            DividerVariant::MiddleInset => (INSET_MARGIN, INSET_MARGIN),
        };

        let has_inset = pad_start > 0.0 || pad_end > 0.0;

        // -- Build the 1dp colored line --
        let line_style = VisualStyle::new().solid_fill(outline_variant);

        if has_inset {
            // Wrap line in a container that provides padding for the inset
            let wrapper = ctx.create_node(ctx.root(), NodeContent::Empty);
            let wrapper_layout = if self.vertical {
                FlexStyle {
                    padding_top: pad_start,
                    padding_bottom: pad_end,
                    flex_grow: 1.0,
                    ..Default::default()
                }
            } else {
                FlexStyle {
                    padding_left: pad_start,
                    padding_right: pad_end,
                    flex_grow: 1.0,
                    ..Default::default()
                }
            };
            ctx.set_layout_style(wrapper, wrapper_layout);

            let line = ctx.create_node(
                wrapper,
                NodeContent::Styled {
                    style: Box::new(line_style),
                },
            );
            let line_layout = if self.vertical {
                FlexStyle {
                    width: Some(DIVIDER_THICKNESS),
                    flex_grow: 1.0,
                    ..Default::default()
                }
            } else {
                FlexStyle {
                    height: Some(DIVIDER_THICKNESS),
                    flex_grow: 1.0,
                    ..Default::default()
                }
            };
            ctx.set_layout_style(line, line_layout);

            wrapper
        } else {
            // Full-width: just the line node
            let line = ctx.create_node(
                ctx.root(),
                NodeContent::Styled {
                    style: Box::new(line_style),
                },
            );
            let line_layout = if self.vertical {
                FlexStyle {
                    width: Some(DIVIDER_THICKNESS),
                    flex_grow: 1.0,
                    ..Default::default()
                }
            } else {
                FlexStyle {
                    height: Some(DIVIDER_THICKNESS),
                    flex_grow: 1.0,
                    ..Default::default()
                }
            };
            ctx.set_layout_style(line, line_layout);

            line
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;

    #[test]
    fn test_horizontal_divider() {
        let mut ctx = WidgetContext::new_test();
        let divider = MaterialDivider::horizontal();
        let root_id = divider.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_OUTLINE_VARIANT.x).abs() < 0.01,
                    "Divider should use outline_variant color, got {color:?}"
                );
            }
        } else {
            panic!("Divider should be Styled content");
        }
    }

    #[test]
    fn test_vertical_divider() {
        let mut ctx = WidgetContext::new_test();
        let divider = MaterialDivider::vertical();
        let root_id = divider.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Vertical divider should build successfully"
        );
    }

    #[test]
    fn test_inset_variant() {
        let divider = MaterialDivider::horizontal().inset();
        assert_eq!(
            divider.variant,
            DividerVariant::Inset,
            "Should be Inset variant"
        );

        let middle = MaterialDivider::horizontal().middle_inset();
        assert_eq!(
            middle.variant,
            DividerVariant::MiddleInset,
            "Should be MiddleInset variant"
        );
    }
}
