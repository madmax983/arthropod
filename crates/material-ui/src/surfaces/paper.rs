//! MD3 Paper widget -- basic surface container with elevation levels.

use crate::theme::MaterialTheme;
use glam::Vec4;
use layout_engine::FlexStyle;
use render_engine::node::NodeContent;
use render_engine::{NodeId, Paint, StrokeStyle, VisualStyle};
use style_engine::StrokeAlign;
use widget_core::Widget;
use widget_core::WidgetContext;

// ---------------------------------------------------------------------------
// MD3 fallback colors
// ---------------------------------------------------------------------------

/// MD3 surface (#FEF7FF) -- paper background.
const FALLBACK_SURFACE: Vec4 = Vec4::new(0.996, 0.969, 1.0, 1.0);

/// MD3 primary (#6750A4) -- used for elevation tint.
const FALLBACK_PRIMARY: Vec4 = Vec4::new(0.404, 0.314, 0.643, 1.0);

/// MD3 outline (#79747E) -- outlined variant border.
const FALLBACK_OUTLINE: Vec4 = Vec4::new(0.475, 0.455, 0.494, 1.0);

// ---------------------------------------------------------------------------
// Dimension constants
// ---------------------------------------------------------------------------

/// Default corner radius in dp.
const DEFAULT_CORNER_RADIUS: f32 = 12.0;

/// Default padding in dp.
const DEFAULT_PADDING: f32 = 16.0;

/// Elevation tint factor per level (primary * factor * level added to surface).
const ELEVATION_TINT_FACTOR: f32 = 0.05;

/// Maximum elevation level (MD3 levels 0-5).
const MAX_ELEVATION: u8 = 5;

// ---------------------------------------------------------------------------
// PaperVariant
// ---------------------------------------------------------------------------

/// Controls how the paper surface is presented.
///
/// - [`Elevated`](PaperVariant::Elevated) -- Uses tonal color shift based on
///   elevation level (higher = slightly lighter).
/// - [`Outlined`](PaperVariant::Outlined) -- Surface color with a 1dp outline
///   border.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaperVariant {
    /// Shadow/elevation via tonal color shift (default).
    #[default]
    Elevated,
    /// 1dp outline border.
    Outlined,
}

// ---------------------------------------------------------------------------
// Paper
// ---------------------------------------------------------------------------

/// MD3 Paper -- basic surface container with elevation levels.
///
/// Paper is the most fundamental surface component in Material Design 3. It
/// provides a background, corner radius, and optional elevation or outline to
/// visually group content.
///
/// # Elevation
///
/// For the [`Elevated`](PaperVariant::Elevated) variant, elevation levels 0-5
/// progressively tint the surface color with the primary color:
///
/// - Level 0: base surface color
/// - Level 1-5: surface + primary * 0.05 * level
///
/// # Example
///
/// ```rust,no_run
/// use material_ui::surfaces::{Paper, PaperVariant};
/// use widget_core::Text;
///
/// let paper = Paper::new()
///     .elevation(2)
///     .child(Text::new("Hello, Paper!"));
///
/// let outlined = Paper::new()
///     .outlined()
///     .child(Text::new("Outlined paper"));
/// ```
pub struct Paper {
    variant: PaperVariant,
    elevation: u8,
    corner_radius: f32,
    child: Option<Box<dyn Widget>>,
    padding: f32,
}

impl Paper {
    /// Create a new paper with MD3 defaults (Elevated, elevation 1, 12dp radius,
    /// 16dp padding).
    pub fn new() -> Self {
        Self {
            variant: PaperVariant::Elevated,
            elevation: 1,
            corner_radius: DEFAULT_CORNER_RADIUS,
            child: None,
            padding: DEFAULT_PADDING,
        }
    }

    /// Switch to the outlined variant (1dp outline border).
    pub fn outlined(mut self) -> Self {
        self.variant = PaperVariant::Outlined;
        self
    }

    /// Set the elevation level (0-5, clamped to MD3 range).
    pub fn elevation(mut self, level: u8) -> Self {
        self.elevation = level.min(MAX_ELEVATION);
        self
    }

    /// Override the corner radius in dp (default: 12.0).
    pub fn corner_radius(mut self, r: f32) -> Self {
        self.corner_radius = r;
        self
    }

    /// Set the child widget rendered inside the paper container.
    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    /// Override the padding in dp (default: 16.0).
    pub fn padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }
}

impl Default for Paper {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the elevated surface color by tinting with primary color.
///
/// `surface + primary * ELEVATION_TINT_FACTOR * level`, clamped to [0, 1].
fn elevated_surface_color(surface: Vec4, primary: Vec4, level: u8) -> Vec4 {
    let tint = ELEVATION_TINT_FACTOR * f32::from(level);
    Vec4::new(
        (surface.x + primary.x * tint).min(1.0),
        (surface.y + primary.y * tint).min(1.0),
        (surface.z + primary.z * tint).min(1.0),
        surface.w, // preserve alpha
    )
}

impl Widget for Paper {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        // -- Resolve theme colors --
        let theme = ctx.get_extension::<MaterialTheme>().cloned();
        let surface = theme
            .as_ref()
            .map(|t| t.color.surface)
            .unwrap_or(FALLBACK_SURFACE);
        let primary = theme
            .as_ref()
            .map(|t| t.color.primary)
            .unwrap_or(FALLBACK_PRIMARY);
        let outline = theme
            .as_ref()
            .map(|t| t.color.outline)
            .unwrap_or(FALLBACK_OUTLINE);

        // -- Build visual style based on variant --
        let style = match self.variant {
            PaperVariant::Elevated => {
                let fill_color = elevated_surface_color(surface, primary, self.elevation);
                VisualStyle::new()
                    .solid_fill(fill_color)
                    .corner_radius(self.corner_radius)
            }
            PaperVariant::Outlined => VisualStyle::new()
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
    fn test_paper_elevated_default() {
        let mut ctx = WidgetContext::new_test();
        let paper = Paper::new();
        let root_id = paper.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(root_id).unwrap();

        // Should be a Styled node
        if let NodeContent::Styled { ref style } = node.content {
            // Should have a fill
            assert!(!style.fills.is_empty(), "Elevated paper should have a fill");

            // Default elevation is 1, so surface should be tinted
            if let Paint::Solid(color) = &style.fills[0] {
                let expected = elevated_surface_color(FALLBACK_SURFACE, FALLBACK_PRIMARY, 1);
                assert!(
                    (color.x - expected.x).abs() < 0.01
                        && (color.y - expected.y).abs() < 0.01
                        && (color.z - expected.z).abs() < 0.01,
                    "Elevated paper (level 1) should have tinted surface color, got {color:?}"
                );
            } else {
                panic!("Paper fill should be solid");
            }

            // Should NOT have a stroke (elevated variant)
            assert!(
                style.stroke.is_none(),
                "Elevated paper should not have a stroke"
            );

            // Should have default corner radius
            assert!(
                (style.corner_radii.top_left - DEFAULT_CORNER_RADIUS).abs() < 0.01,
                "Paper should have default corner radius {DEFAULT_CORNER_RADIUS}, got {}",
                style.corner_radii.top_left
            );
        } else {
            panic!("Paper container should be Styled content");
        }
    }

    #[test]
    fn test_paper_outlined_variant() {
        let mut ctx = WidgetContext::new_test();
        let paper = Paper::new().outlined();
        let root_id = paper.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(root_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            // Should have a fill (base surface color, no tint)
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE.y).abs() < 0.01
                        && (color.z - FALLBACK_SURFACE.z).abs() < 0.01,
                    "Outlined paper should have base surface color, got {color:?}"
                );
            }

            // Should have a stroke (outlined variant)
            assert!(
                style.stroke.is_some(),
                "Outlined paper should have a stroke"
            );
            let stroke = style.stroke.as_ref().unwrap();
            assert!(
                (stroke.weight - 1.0).abs() < 0.01,
                "Outline stroke weight should be 1dp, got {}",
                stroke.weight
            );
        } else {
            panic!("Paper container should be Styled content");
        }
    }

    #[test]
    fn test_paper_custom_elevation() {
        let mut ctx = WidgetContext::new_test();
        let paper = Paper::new().elevation(3);
        let root_id = paper.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(root_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            if let Paint::Solid(color) = &style.fills[0] {
                let expected = elevated_surface_color(FALLBACK_SURFACE, FALLBACK_PRIMARY, 3);
                assert!(
                    (color.x - expected.x).abs() < 0.01
                        && (color.y - expected.y).abs() < 0.01
                        && (color.z - expected.z).abs() < 0.01,
                    "Elevation 3 paper should have more tinted surface, got {color:?}"
                );
                // Level 3 should be lighter than level 0
                assert!(
                    color.x > FALLBACK_SURFACE.x || color.y > FALLBACK_SURFACE.y,
                    "Elevation 3 should produce a lighter surface color"
                );
            } else {
                panic!("Paper fill should be solid");
            }
        } else {
            panic!("Paper container should be Styled content");
        }
    }

    #[test]
    fn test_paper_child_builds() {
        let mut ctx = WidgetContext::new_test();
        let paper = Paper::new().child(widget_core::Text::new("Hello"));
        let root_id = paper.build(&mut ctx);

        let scene = ctx.scene();
        let container = scene.get_node(root_id).unwrap();

        assert!(
            !container.children.is_empty(),
            "Paper with child should have children"
        );
    }

    #[test]
    fn test_paper_custom_corner_radius() {
        let mut ctx = WidgetContext::new_test();
        let paper = Paper::new().corner_radius(24.0);
        let root_id = paper.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(root_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            assert!(
                (style.corner_radii.top_left - 24.0).abs() < 0.01
                    && (style.corner_radii.top_right - 24.0).abs() < 0.01
                    && (style.corner_radii.bottom_right - 24.0).abs() < 0.01
                    && (style.corner_radii.bottom_left - 24.0).abs() < 0.01,
                "Custom corner radius should be 24.0, got {:?}",
                style.corner_radii
            );
        } else {
            panic!("Paper container should be Styled content");
        }
    }

    #[test]
    fn test_paper_elevation_clamped_to_max() {
        let paper = Paper::new().elevation(10);
        assert_eq!(
            paper.elevation, MAX_ELEVATION,
            "Elevation should be clamped to {MAX_ELEVATION}, got {}",
            paper.elevation
        );
    }

    #[test]
    fn test_paper_default_is_same_as_new() {
        let a = Paper::new();
        let b = Paper::default();
        assert_eq!(a.variant, b.variant);
        assert_eq!(a.elevation, b.elevation);
        assert!((a.corner_radius - b.corner_radius).abs() < f32::EPSILON);
        assert!((a.padding - b.padding).abs() < f32::EPSILON);
        assert!(a.child.is_none());
        assert!(b.child.is_none());
    }

    #[test]
    fn test_paper_with_theme() {
        let mut ctx = WidgetContext::new_test();
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        ctx.set_extension(theme);

        let paper = Paper::new().elevation(2);
        let root_id = paper.build(&mut ctx);

        assert!(
            ctx.scene().get_node(root_id).is_some(),
            "Paper with theme should build successfully"
        );
    }

    #[test]
    fn test_paper_elevation_zero_is_base_surface() {
        let mut ctx = WidgetContext::new_test();
        let paper = Paper::new().elevation(0);
        let root_id = paper.build(&mut ctx);

        let scene = ctx.scene();
        let node = scene.get_node(root_id).unwrap();

        if let NodeContent::Styled { ref style } = node.content {
            if let Paint::Solid(color) = &style.fills[0] {
                assert!(
                    (color.x - FALLBACK_SURFACE.x).abs() < 0.01
                        && (color.y - FALLBACK_SURFACE.y).abs() < 0.01
                        && (color.z - FALLBACK_SURFACE.z).abs() < 0.01,
                    "Elevation 0 should use base surface color, got {color:?}"
                );
            }
        } else {
            panic!("Paper container should be Styled content");
        }
    }

    #[test]
    fn test_paper_custom_padding() {
        let paper = Paper::new().padding(32.0);
        assert!(
            (paper.padding - 32.0).abs() < f32::EPSILON,
            "Custom padding should be 32.0, got {}",
            paper.padding
        );
    }
}
