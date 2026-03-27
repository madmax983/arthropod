//! MD3 MaterialCard -- container with Elevated/Filled/Outlined variants.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::FlexStyle;
use render_engine::node::NodeContent;
use render_engine::{NodeId, Paint, StrokeStyle, VisualStyle};
use style_engine::StrokeAlign;
use widget_core::WidgetContext;
use widget_core::widget_trait::Widget;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 surface (#FEF7FF)
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 surface_variant (#E7E0EC)
const FALLBACK_SURFACE_VARIANT: Vec4 = Vec4::new(0.906, 0.878, 0.925, 1.0);

/// MD3 primary (#6750A4) -- used for elevation tint.
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 outline (#79747E)
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// Default corner radius (MD3 medium).
const DEFAULT_CORNER_RADIUS: f32 = 12.0;

/// Default padding in dp.
const DEFAULT_PADDING: f32 = 16.0;

/// Elevation tint factor for Elevated variant.
const ELEVATION_TINT_FACTOR: f32 = 0.05;

// ---------------------------------------------------------------------------
// CardVariant
// ---------------------------------------------------------------------------

/// MD3 card variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardVariant {
    /// Surface background with elevation tint.
    Elevated,
    /// Surface-variant background.
    #[default]
    Filled,
    /// Surface background with 1dp outline.
    Outlined,
}

// ---------------------------------------------------------------------------
// MaterialCard
// ---------------------------------------------------------------------------

/// MD3 Card with three visual variants.
///
/// Builds a container with MD3 medium corner radius (12dp), padding, and
/// variant-specific styling. Accepts an optional child widget.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::components::MaterialCard;
/// use widget_core::Text;
///
/// let card = MaterialCard::new()
///     .child(Text::new("Card content"))
///     .padding(24.0);
///
/// let outlined = MaterialCard::new()
///     .outlined()
///     .child(Text::new("Outlined card"));
/// ```
pub struct MaterialCard {
    variant: CardVariant,
    child: Option<Box<dyn Widget>>,
    padding: f32,
    corner_radius: f32,
}

impl MaterialCard {
    /// Create a new filled card with MD3 defaults.
    pub fn new() -> Self {
        Self {
            variant: CardVariant::Filled,
            child: None,
            padding: DEFAULT_PADDING,
            corner_radius: DEFAULT_CORNER_RADIUS,
        }
    }

    /// Switch to the Elevated variant.
    pub fn elevated(mut self) -> Self {
        self.variant = CardVariant::Elevated;
        self
    }

    /// Switch to the Outlined variant.
    pub fn outlined(mut self) -> Self {
        self.variant = CardVariant::Outlined;
        self
    }

    /// Set a child widget rendered inside the card.
    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    /// Override padding in dp (default: 16.0).
    pub fn padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }

    /// Override corner radius in dp (default: 12.0).
    pub fn corner_radius(mut self, r: f32) -> Self {
        self.corner_radius = r;
        self
    }
}

impl Default for MaterialCard {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MaterialCard {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);
        let surface_variant = theme
            .as_ref()
            .map(|t| t.color.surface_variant)
            .unwrap_or(FALLBACK_SURFACE_VARIANT);
        let primary = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);

        // -- Variant-specific style --
        let style = match self.variant {
            CardVariant::Elevated => {
                let tinted = Vec4::new(
                    (surface.x + primary.x * ELEVATION_TINT_FACTOR).min(1.0),
                    (surface.y + primary.y * ELEVATION_TINT_FACTOR).min(1.0),
                    (surface.z + primary.z * ELEVATION_TINT_FACTOR).min(1.0),
                    surface.w,
                );
                VisualStyle::new()
                    .solid_fill(tinted)
                    .corner_radius(self.corner_radius)
            }
            CardVariant::Filled => VisualStyle::new()
                .solid_fill(surface_variant)
                .corner_radius(self.corner_radius),
            CardVariant::Outlined => VisualStyle::new()
                .solid_fill(surface)
                .corner_radius(self.corner_radius)
                .stroke(StrokeStyle::solid(
                    Paint::solid(outline),
                    1.0,
                    StrokeAlign::Inside,
                )),
        };

        // -- Container node --
        let container = ctx.create_node(
            ctx.root(),
            NodeContent::Styled {
                style: Box::new(style),
            },
        );
        ctx.set_layout_style(
            container,
            FlexStyle {
                padding_left: self.padding,
                padding_right: self.padding,
                padding_top: self.padding,
                padding_bottom: self.padding,
                ..Default::default()
            },
        );

        // -- Child widget (optional) --
        if let Some(ref child_widget) = self.child {
            let child_id = child_widget.build(ctx);
            ctx.reparent_to(child_id, container);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;

    #[test]
    fn test_filled_default() {
        let mut ctx = WidgetContext::new_test();
        let card = MaterialCard::new();
        let root_id = card.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // Filled variant should use surface_variant fill
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE_VARIANT.x).abs() < 0.01,
                    "Filled card should use surface_variant fill, got {color:?}"
                );
            }
            // Should NOT have a stroke
            assert!(
                style.stroke.is_none(),
                "Filled card should not have a stroke"
            );
            // MD3 medium corner radius
            assert!(
                (style.corner_radii.top_left - DEFAULT_CORNER_RADIUS).abs() < 0.01,
                "Card should have 12dp corner radius"
            );
        } else {
            panic!("Card container should be Styled content");
        }
    }

    #[test]
    fn test_elevated_variant() {
        let mut ctx = WidgetContext::new_test();
        let card = MaterialCard::new().elevated();
        let root_id = card.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // Elevated should have tinted surface fill (slightly different from base surface)
            if let Paint::Solid(color) = &style.fills[0] {
                // Tinted should be >= surface
                assert!(
                    color.x >= FALLBACK_SURFACE.x - 0.01,
                    "Elevated card fill should be tinted surface, got {color:?}"
                );
            }
            // No stroke for elevated
            assert!(
                style.stroke.is_none(),
                "Elevated card should not have a stroke"
            );
        } else {
            panic!("Card container should be Styled content");
        }
    }

    #[test]
    fn test_outlined_variant() {
        let mut ctx = WidgetContext::new_test();
        let card = MaterialCard::new().outlined();
        let root_id = card.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            assert!(style.stroke.is_some(), "Outlined card should have a stroke");
            let stroke = style.stroke.as_ref().unwrap();
            assert!(
                (stroke.weight - 1.0).abs() < 0.01,
                "Outline stroke weight should be 1dp"
            );
        } else {
            panic!("Card container should be Styled content");
        }
    }

    #[test]
    fn test_with_child() {
        let mut ctx = WidgetContext::new_test();
        let card = MaterialCard::new().child(widget_core::Text::new("Hello"));
        let root_id = card.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        assert!(
            !node.children.is_empty(),
            "Card with child should have children"
        );
    }
}
