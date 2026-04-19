//! Material Design 3 theme system.
//!
//! Provides a complete MD3 theme generated from a single seed color,
//! including color scheme, typography scale, shape scale, and elevation scale.

mod color;
mod elevation;
mod shape;
mod typography;

pub use color::ColorScheme;
pub use elevation::{ElevationLevel, ElevationScale};
pub use shape::ShapeScale;
pub use typography::{FontWeight, TextStyle, TypographyScale};

use glam::Vec4;
use theme_engine::DesignTokens;

/// Top-level Material Design 3 theme.
///
/// Generated from a single seed color via [`MaterialTheme::from_seed`].
/// Contains all four MD3 subsystems: color, typography, shape, and elevation.
///
/// # Examples
///
/// ```
/// use glam::Vec4;
/// use material_ui::theme::MaterialTheme;
///
/// // Generate a theme from a purple seed
/// let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
/// let tokens = theme.to_design_tokens();
/// ```
#[derive(Debug, Clone)]
pub struct MaterialTheme {
    /// The comprehensive MD3 color palette generated from the seed color.
    pub color: ColorScheme,
    /// The standard 15-slot MD3 typography scale for text rendering.
    pub typography: TypographyScale,
    /// The seven-tier MD3 shape scale for component corner radii.
    pub shape: ShapeScale,
    /// The six-level MD3 tonal elevation system mapping depth to surface tint/shadows.
    pub elevation: ElevationScale,
}

impl MaterialTheme {
    /// Create a light theme from a seed color.
    ///
    /// Alias for [`MaterialTheme::from_seed_light`].
    pub fn from_seed(seed: Vec4) -> Self {
        Self::from_seed_light(seed)
    }

    /// Create a light theme from a seed color.
    pub fn from_seed_light(seed: Vec4) -> Self {
        Self {
            color: ColorScheme::from_seed_light(seed),
            typography: TypographyScale::default(),
            shape: ShapeScale::default(),
            elevation: ElevationScale::default(),
        }
    }

    /// Create a dark theme from a seed color.
    pub fn from_seed_dark(seed: Vec4) -> Self {
        Self {
            color: ColorScheme::from_seed_dark(seed),
            typography: TypographyScale::default(),
            shape: ShapeScale::default(),
            elevation: ElevationScale::default(),
        }
    }

    /// Convert to widget-core [`DesignTokens`] for compatibility with base widgets.
    ///
    /// Maps MD3 color roles to the semantic token system used by
    /// the base widget layer, enabling MD3 themes to work with
    /// existing widgets transparently.
    pub fn to_design_tokens(&self) -> DesignTokens {
        let c = &self.color;
        DesignTokens {
            surface_primary: theme_engine::TokenValue::Color(c.surface),
            surface_secondary: theme_engine::TokenValue::Color(c.surface_container),
            surface_elevated: theme_engine::TokenValue::Color(c.surface_container_high),
            text_primary: c.on_surface,
            text_secondary: c.on_surface_variant,
            text_tertiary: c.outline,
            accent: c.primary,
            accent_hover: c.primary_container,
            accent_pressed: c.on_primary_container,
            space_xs: 4.0,
            space_sm: 8.0,
            space_md: 16.0,
            space_lg: 24.0,
            space_xl: 32.0,
            space_2xl: 48.0,
            radius_sm: self.shape.extra_small,
            radius_md: self.shape.small,
            radius_lg: self.shape.medium,
            radius_xl: self.shape.large,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_seed_creates_complete_theme() {
        let theme = MaterialTheme::from_seed(Vec4::new(0.4, 0.2, 0.8, 1.0));
        // Verify all subsystems are populated
        assert_eq!(theme.color.primary.w, 1.0);
        assert_eq!(theme.typography.body_medium.font_size, 14.0);
        assert_eq!(theme.shape.medium, 12.0);
        assert_eq!(theme.elevation.level0.tint_opacity, 0.0);
    }

    #[test]
    fn test_light_and_dark_differ() {
        let seed = Vec4::new(0.4, 0.2, 0.8, 1.0);
        let light = MaterialTheme::from_seed_light(seed);
        let dark = MaterialTheme::from_seed_dark(seed);

        // Surface colors should be very different
        let diff = (light.color.surface - dark.color.surface).length();
        assert!(
            diff > 0.5,
            "Light and dark surfaces should differ significantly, diff={diff}"
        );
    }

    #[test]
    fn test_to_design_tokens() {
        let theme = MaterialTheme::from_seed(Vec4::new(0.2, 0.5, 0.8, 1.0));
        let tokens = theme.to_design_tokens();

        // Surface primary should map from MD3 surface
        let surface_color = tokens.surface_primary.as_color();
        assert!(
            surface_color.x > 0.9,
            "Surface primary should be light, got {}",
            surface_color.x
        );

        // Accent should map from MD3 primary
        assert_eq!(tokens.accent, theme.color.primary);

        // Radius should map from shape scale
        assert_eq!(tokens.radius_sm, theme.shape.extra_small);
        assert_eq!(tokens.radius_md, theme.shape.small);
        assert_eq!(tokens.radius_lg, theme.shape.medium);
        assert_eq!(tokens.radius_xl, theme.shape.large);

        // Spacing should be standard MD3 values
        assert_eq!(tokens.space_xs, 4.0);
        assert_eq!(tokens.space_sm, 8.0);
        assert_eq!(tokens.space_md, 16.0);
        assert_eq!(tokens.space_lg, 24.0);
        assert_eq!(tokens.space_xl, 32.0);
        assert_eq!(tokens.space_2xl, 48.0);
    }
}
