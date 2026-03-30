//! MD3 MaterialButton -- pill-shaped button with 5 variants.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::{FlexAlign, FlexJustifyContent, FlexStyle};
use render_engine::node::NodeContent;
use render_engine::{NodeId, Paint, StrokeStyle, TextContent, VisualStyle};
use std::sync::Arc;
use style_engine::StrokeAlign;
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 primary (#6750A4)
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 on_primary (white)
const FALLBACK_ON_PRIMARY: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

/// MD3 secondary_container (#E8DEF8)
const FALLBACK_SECONDARY_CONTAINER: Vec4 = Vec4::new(0.906, 0.831, 0.996, 1.0);

/// MD3 on_secondary_container (#1D192B)
const FALLBACK_ON_SECONDARY_CONTAINER: Vec4 = Vec4::new(0.114, 0.012, 0.329, 1.0);

/// MD3 surface (#FEF7FF)
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 outline (#79747E)
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

/// Transparent fill
const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

/// Disabled content opacity multiplier.
const DISABLED_ALPHA: f32 = 0.38;

/// Fixed button height in dp.
const BUTTON_HEIGHT: f32 = 40.0;

/// Horizontal padding in dp.
const BUTTON_PADDING_H: f32 = 24.0;

/// MD3 pill corner radius (full rounding).
const PILL_CORNER_RADIUS: f32 = 20.0;

/// MD3 label_large font size.
const LABEL_LARGE_SIZE: f32 = 14.0;

/// Elevation tint factor for the Elevated variant.
const ELEVATION_TINT_FACTOR: f32 = 0.05;

// ---------------------------------------------------------------------------
// MaterialButtonVariant
// ---------------------------------------------------------------------------

/// MD3 button variant controlling background, text color, and border.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaterialButtonVariant {
    /// Primary background, on_primary text.
    #[default]
    Filled,
    /// Secondary container background, on_secondary_container text.
    Tonal,
    /// Transparent background with 1dp outline, primary text.
    Outlined,
    /// Transparent background, primary text, no border.
    Text,
    /// Surface background with elevation tint, primary text.
    Elevated,
}

// ---------------------------------------------------------------------------
// MaterialButton
// ---------------------------------------------------------------------------

/// MD3 Button with pill shape and five visual variants.
///
/// Builds a 40dp-tall container with 24dp horizontal padding and full pill
/// corner radius (20dp). Colors are resolved from [`MaterialTheme`] with
/// sensible MD3 fallbacks.
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::components::MaterialButton;
///
/// let btn = MaterialButton::new("Save")
///     .on_click(|| println!("Saved!"));
///
/// let tonal = MaterialButton::new("Cancel").tonal();
/// ```
pub struct MaterialButton {
    text: String,
    variant: MaterialButtonVariant,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    disabled: bool,
}

impl MaterialButton {
    /// Create a new filled button with the given label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: MaterialButtonVariant::Filled,
            on_click: None,
            disabled: false,
        }
    }

    /// Switch to the Tonal variant.
    pub fn tonal(mut self) -> Self {
        self.variant = MaterialButtonVariant::Tonal;
        self
    }

    /// Switch to the Outlined variant.
    pub fn outlined(mut self) -> Self {
        self.variant = MaterialButtonVariant::Outlined;
        self
    }

    /// Switch to the Text variant (no background, no border).
    pub fn text_variant(mut self) -> Self {
        self.variant = MaterialButtonVariant::Text;
        self
    }

    /// Switch to the Elevated variant (surface + elevation tint).
    pub fn elevated(mut self) -> Self {
        self.variant = MaterialButtonVariant::Elevated;
        self
    }

    /// Set the click handler.
    pub fn on_click(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(f));
        self
    }

    /// Disable the button. Disables click interaction and reduces opacity.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Widget for MaterialButton {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let primary = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let on_primary = theme
            .as_ref()
            .map(|t| t.color.on_primary)
            .unwrap_or(FALLBACK_ON_PRIMARY);
        let secondary_container = theme
            .as_ref()
            .map(|t| t.color.secondary_container)
            .unwrap_or(FALLBACK_SECONDARY_CONTAINER);
        let on_secondary_container = theme
            .as_ref()
            .map(|t| t.color.on_secondary_container)
            .unwrap_or(FALLBACK_ON_SECONDARY_CONTAINER);
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);

        // -- Determine variant-specific colors --
        let (fill_color, text_color, stroke) = match self.variant {
            MaterialButtonVariant::Filled => (primary, on_primary, None),
            MaterialButtonVariant::Tonal => (secondary_container, on_secondary_container, None),
            MaterialButtonVariant::Outlined => (
                TRANSPARENT,
                primary,
                Some(StrokeStyle::solid(
                    Paint::solid(outline),
                    1.0,
                    StrokeAlign::Inside,
                )),
            ),
            MaterialButtonVariant::Text => (TRANSPARENT, primary, None),
            MaterialButtonVariant::Elevated => {
                let tinted = Vec4::new(
                    (surface.x + primary.x * ELEVATION_TINT_FACTOR).min(1.0),
                    (surface.y + primary.y * ELEVATION_TINT_FACTOR).min(1.0),
                    (surface.z + primary.z * ELEVATION_TINT_FACTOR).min(1.0),
                    surface.w,
                );
                (tinted, primary, None)
            }
        };

        // -- Apply disabled alpha --
        let (fill_color, text_color) = if self.disabled {
            (
                Vec4::new(fill_color.x, fill_color.y, fill_color.z, DISABLED_ALPHA),
                Vec4::new(text_color.x, text_color.y, text_color.z, DISABLED_ALPHA),
            )
        } else {
            (fill_color, text_color)
        };

        // -- Build visual style --
        let mut style = VisualStyle::new()
            .solid_fill(fill_color)
            .corner_radius(PILL_CORNER_RADIUS);
        if let Some(s) = stroke {
            style = style.stroke(s);
        }

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
                height: Some(BUTTON_HEIGHT),
                padding_left: BUTTON_PADDING_H,
                padding_right: BUTTON_PADDING_H,
                justify_content: FlexJustifyContent::Center,
                align_items: FlexAlign::Center,
                ..Default::default()
            },
        );

        // -- Text label --
        let text_style = VisualStyle::new()
            .solid_fill(text_color)
            .text(TextContent::new(self.text.clone(), LABEL_LARGE_SIZE));
        let text_node = ctx.create_node(
            container,
            NodeContent::Styled {
                style: Box::new(text_style),
            },
        );

        // Suppress unused variable warning
        let _ = text_node;

        // -- Click handler --
        if !self.disabled
            && let Some(ref on_click) = self.on_click
        {
            let callback = Arc::clone(on_click);
            ctx.add_clickable(container, callback);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Paint;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_filled_button_default() {
        let mut ctx = WidgetContext::new_test();
        let btn = MaterialButton::new("Save");
        let root_id = btn.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // Filled variant should have primary fill
            assert!(!style.fills.is_empty(), "Filled button should have a fill");
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_PRIMARY.x).abs() < 0.01
                        && (color.y - FALLBACK_PRIMARY.y).abs() < 0.01,
                    "Filled button should use primary color, got {color:?}"
                );
            }
            // Should NOT have a stroke
            assert!(
                style.stroke.is_none(),
                "Filled button should not have a stroke"
            );
            // Should have pill corner radius
            assert!(
                (style.corner_radii.top_left - PILL_CORNER_RADIUS).abs() < 0.01,
                "Button should have pill corner radius"
            );
        } else {
            panic!("MaterialButton container should be Styled content");
        }

        // Should have a text child
        assert!(
            !node.children.is_empty(),
            "MaterialButton should have a text child"
        );
    }

    #[test]
    fn test_tonal_variant() {
        let mut ctx = WidgetContext::new_test();
        let btn = MaterialButton::new("Cancel").tonal();
        let root_id = btn.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SECONDARY_CONTAINER.x).abs() < 0.01,
                    "Tonal button should use secondary_container fill, got {color:?}"
                );
            }
        } else {
            panic!("MaterialButton container should be Styled content");
        }
    }

    #[test]
    fn test_outlined_variant() {
        let mut ctx = WidgetContext::new_test();
        let btn = MaterialButton::new("Details").outlined();
        let root_id = btn.build(&mut ctx);

        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content {
            // Transparent fill
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    color.w < 0.01,
                    "Outlined button fill should be transparent, got alpha={}",
                    color.w
                );
            }
            // Should have a stroke
            assert!(
                style.stroke.is_some(),
                "Outlined button should have a stroke"
            );
            let stroke = style.stroke.as_ref().unwrap();
            assert!(
                (stroke.weight - 1.0).abs() < 0.01,
                "Outlined stroke weight should be 1dp"
            );
        } else {
            panic!("MaterialButton container should be Styled content");
        }
    }

    #[test]
    fn test_disabled_button() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = Arc::clone(&clicked);

        let mut ctx = WidgetContext::new_test();
        let btn = MaterialButton::new("Disabled")
            .disabled(true)
            .on_click(move || {
                clicked_clone.store(true, Ordering::SeqCst);
            });
        let root_id = btn.build(&mut ctx);

        assert!(
            !ctx.has_clickable(root_id),
            "Disabled button should not be clickable"
        );

        // Fill should have reduced alpha
        let node = ctx.scene().get_node(root_id).unwrap();
        if let NodeContent::Styled { ref style } = node.content
            && let Paint::Solid(color) = &style.fills[0]
        {
            assert!(
                (color.w - DISABLED_ALPHA).abs() < 0.01,
                "Disabled button fill should have reduced alpha ({}), got {}",
                DISABLED_ALPHA,
                color.w
            );
        }
    }
}
