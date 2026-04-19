//! MD3 color scheme generation from a seed color.
//!
//! Implements HSL-based tonal palette generation following Material Design 3
//! color system guidelines. A single seed color produces a complete set of
//! color roles for both light and dark themes.

use glam::Vec4;

/// MD3 color scheme with all color roles.
///
/// Generated from a seed color via [`ColorScheme::from_seed_light`] or
/// [`ColorScheme::from_seed_dark`]. Contains primary, secondary, tertiary,
/// error, surface, outline, and inverse color groups.
///
/// # Examples
///
/// ```
/// use glam::Vec4;
/// use material_ui::theme::ColorScheme;
///
/// // Create a light color scheme from a red seed
/// let light_scheme = ColorScheme::from_seed_light(Vec4::new(0.9, 0.2, 0.2, 1.0));
/// assert!(light_scheme.surface.x > 0.9); // Surface is light
///
/// // Create a dark color scheme from the same red seed
/// let dark_scheme = ColorScheme::from_seed_dark(Vec4::new(0.9, 0.2, 0.2, 1.0));
/// assert!(dark_scheme.surface.x < 0.2); // Surface is dark
/// ```
#[derive(Debug, Clone)]
pub struct ColorScheme {
    // Primary
    /// The most prominent color, used for key components like FABs, prominent buttons, and active states.
    pub primary: Vec4,
    /// Color used for text and icons overlaid on top of the primary color.
    pub on_primary: Vec4,
    /// A standalone color used for elements needing less emphasis than primary, such as tonal buttons.
    pub primary_container: Vec4,
    /// Color used for text and icons overlaid on top of the primary container color.
    pub on_primary_container: Vec4,

    // Secondary
    /// Used for less prominent components, like filter chips, expanding components, and secondary buttons.
    pub secondary: Vec4,
    /// Color used for text and icons overlaid on top of the secondary color.
    pub on_secondary: Vec4,
    /// A standalone color for secondary elements needing less emphasis, like tonal secondary buttons.
    pub secondary_container: Vec4,
    /// Color used for text and icons overlaid on top of the secondary container color.
    pub on_secondary_container: Vec4,

    // Tertiary
    /// Used for contrasting accents that balance primary and secondary colors, such as badges or active inputs.
    pub tertiary: Vec4,
    /// Color used for text and icons overlaid on top of the tertiary color.
    pub on_tertiary: Vec4,
    /// A standalone color for tertiary elements needing less emphasis.
    pub tertiary_container: Vec4,
    /// Color used for text and icons overlaid on top of the tertiary container color.
    pub on_tertiary_container: Vec4,

    // Error
    /// Used to indicate errors and severe warnings, such as invalid text fields.
    pub error: Vec4,
    /// Color used for text and icons overlaid on top of the error color.
    pub on_error: Vec4,
    /// A standalone error color for elements needing less emphasis, like error states on cards.
    pub error_container: Vec4,
    /// Color used for text and icons overlaid on top of the error container color.
    pub on_error_container: Vec4,

    // Surface
    /// The default background color for the application page or root background.
    pub surface: Vec4,
    /// Color used for primary text and icons overlaid on top of the surface color.
    pub on_surface: Vec4,
    /// A variant of the surface color for subtle grouping or boundaries without sharp lines.
    pub surface_variant: Vec4,
    /// Color used for secondary text and icons overlaid on top of the surface variant color.
    pub on_surface_variant: Vec4,
    /// The lowest emphasis surface container, slightly lighter/darker than surface based on theme.
    pub surface_container_lowest: Vec4,
    /// A low emphasis surface container, used for cards or bounded areas.
    pub surface_container_low: Vec4,
    /// The standard surface container, providing a mid-level baseline for layered content.
    pub surface_container: Vec4,
    /// A high emphasis surface container, used for dialogs or popups.
    pub surface_container_high: Vec4,
    /// The highest emphasis surface container, used for temporary surfaces like drawers or menus.
    pub surface_container_highest: Vec4,

    // Outline
    /// Used for prominent boundaries, such as text field outlines or dividers requiring high contrast.
    pub outline: Vec4,
    /// Used for decorative boundaries, dividers, and decorative outlines requiring low contrast.
    pub outline_variant: Vec4,

    // Inverse
    /// A color that strongly contrasts with the surface color, used for snackbars and tooltips.
    pub inverse_surface: Vec4,
    /// Color used for text and icons overlaid on top of the inverse surface color.
    pub inverse_on_surface: Vec4,
    /// Used for actions within inverse surface components, like a button on a snackbar.
    pub inverse_primary: Vec4,

    // Misc
    /// A semi-transparent overlay color applied behind modal layers like dialogs or bottom sheets.
    pub scrim: Vec4,
    /// Color used for casting structural shadows under elevated components.
    pub shadow: Vec4,
}

impl ColorScheme {
    /// Generate a light color scheme from a seed color.
    ///
    /// The seed color's hue and saturation are extracted and used to generate
    /// a complete MD3 tonal palette. Lightness values follow MD3 light theme
    /// conventions.
    ///
    /// # Arguments
    ///
    /// * `seed` - RGBA color as Vec4 (components in 0.0..1.0 range)
    pub fn from_seed_light(seed: Vec4) -> Self {
        let (h, s, _) = rgb_to_hsl(seed.x, seed.y, seed.z);

        // Ensure minimum saturation for a vibrant palette
        let s_primary = s.max(0.4);
        let s_secondary = s_primary * 0.5;
        let h_tertiary = (h + 60.0 / 360.0) % 1.0;
        let s_tertiary = s_primary * 0.7;
        let s_neutral = s_primary * 0.08;
        let s_neutral_variant = s_primary * 0.15;

        // Error palette: red hue, fixed saturation
        let h_error = 0.0;
        let s_error = 0.8;

        Self {
            // Primary
            primary: tone(h, s_primary, 0.40),
            on_primary: tone(h, s_primary, 1.00),
            primary_container: tone(h, s_primary, 0.90),
            on_primary_container: tone(h, s_primary, 0.10),

            // Secondary
            secondary: tone(h, s_secondary, 0.40),
            on_secondary: tone(h, s_secondary, 1.00),
            secondary_container: tone(h, s_secondary, 0.90),
            on_secondary_container: tone(h, s_secondary, 0.10),

            // Tertiary
            tertiary: tone(h_tertiary, s_tertiary, 0.40),
            on_tertiary: tone(h_tertiary, s_tertiary, 1.00),
            tertiary_container: tone(h_tertiary, s_tertiary, 0.90),
            on_tertiary_container: tone(h_tertiary, s_tertiary, 0.10),

            // Error
            error: tone(h_error, s_error, 0.40),
            on_error: tone(h_error, s_error, 1.00),
            error_container: tone(h_error, s_error, 0.90),
            on_error_container: tone(h_error, s_error, 0.10),

            // Surface (neutral palette)
            surface: tone(h, s_neutral, 0.98),
            on_surface: tone(h, s_neutral, 0.10),
            surface_variant: tone(h, s_neutral_variant, 0.90),
            on_surface_variant: tone(h, s_neutral_variant, 0.30),
            surface_container_lowest: tone(h, s_neutral, 1.00),
            surface_container_low: tone(h, s_neutral, 0.96),
            surface_container: tone(h, s_neutral, 0.94),
            surface_container_high: tone(h, s_neutral, 0.92),
            surface_container_highest: tone(h, s_neutral, 0.90),

            // Outline
            outline: tone(h, s_neutral_variant, 0.50),
            outline_variant: tone(h, s_neutral_variant, 0.80),

            // Inverse (dark-on-light)
            inverse_surface: tone(h, s_neutral, 0.20),
            inverse_on_surface: tone(h, s_neutral, 0.95),
            inverse_primary: tone(h, s_primary, 0.80),

            // Misc
            scrim: Vec4::new(0.0, 0.0, 0.0, 1.0),
            shadow: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    /// Generate a dark color scheme from a seed color.
    ///
    /// Uses inverted lightness values compared to the light scheme,
    /// with lighter content colors on darker surfaces.
    ///
    /// # Arguments
    ///
    /// * `seed` - RGBA color as Vec4 (components in 0.0..1.0 range)
    pub fn from_seed_dark(seed: Vec4) -> Self {
        let (h, s, _) = rgb_to_hsl(seed.x, seed.y, seed.z);

        let s_primary = s.max(0.4);
        let s_secondary = s_primary * 0.5;
        let h_tertiary = (h + 60.0 / 360.0) % 1.0;
        let s_tertiary = s_primary * 0.7;
        let s_neutral = s_primary * 0.08;
        let s_neutral_variant = s_primary * 0.15;

        let h_error = 0.0;
        let s_error = 0.8;

        Self {
            // Primary (inverted: lighter primary on dark background)
            primary: tone(h, s_primary, 0.80),
            on_primary: tone(h, s_primary, 0.20),
            primary_container: tone(h, s_primary, 0.30),
            on_primary_container: tone(h, s_primary, 0.90),

            // Secondary
            secondary: tone(h, s_secondary, 0.80),
            on_secondary: tone(h, s_secondary, 0.20),
            secondary_container: tone(h, s_secondary, 0.30),
            on_secondary_container: tone(h, s_secondary, 0.90),

            // Tertiary
            tertiary: tone(h_tertiary, s_tertiary, 0.80),
            on_tertiary: tone(h_tertiary, s_tertiary, 0.20),
            tertiary_container: tone(h_tertiary, s_tertiary, 0.30),
            on_tertiary_container: tone(h_tertiary, s_tertiary, 0.90),

            // Error
            error: tone(h_error, s_error, 0.80),
            on_error: tone(h_error, s_error, 0.20),
            error_container: tone(h_error, s_error, 0.30),
            on_error_container: tone(h_error, s_error, 0.90),

            // Surface (dark surfaces)
            surface: tone(h, s_neutral, 0.06),
            on_surface: tone(h, s_neutral, 0.90),
            surface_variant: tone(h, s_neutral_variant, 0.30),
            on_surface_variant: tone(h, s_neutral_variant, 0.80),
            surface_container_lowest: tone(h, s_neutral, 0.04),
            surface_container_low: tone(h, s_neutral, 0.10),
            surface_container: tone(h, s_neutral, 0.12),
            surface_container_high: tone(h, s_neutral, 0.17),
            surface_container_highest: tone(h, s_neutral, 0.22),

            // Outline
            outline: tone(h, s_neutral_variant, 0.60),
            outline_variant: tone(h, s_neutral_variant, 0.30),

            // Inverse (light-on-dark)
            inverse_surface: tone(h, s_neutral, 0.90),
            inverse_on_surface: tone(h, s_neutral, 0.20),
            inverse_primary: tone(h, s_primary, 0.40),

            // Misc
            scrim: Vec4::new(0.0, 0.0, 0.0, 1.0),
            shadow: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

// ---------------------------------------------------------------------------
// HSL helpers (private)
// ---------------------------------------------------------------------------

/// Convert RGB (0.0..1.0) to HSL (h in 0.0..1.0, s in 0.0..1.0, l in 0.0..1.0).
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-6 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < 1e-6 {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() < 1e-6 {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h, s, l)
}

/// Convert HSL (all 0.0..1.0) to RGB (0.0..1.0).
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s.abs() < 1e-6 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

/// Helper for HSL-to-RGB conversion — maps a hue segment to an RGB channel.
fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Generate a color at a specific tone (lightness) while preserving hue.
///
/// Saturation is reduced at extreme lightness values to avoid clipping
/// artifacts, producing more natural tonal progressions.
fn tone(h: f32, s: f32, lightness: f32) -> Vec4 {
    // Reduce saturation at extreme lightness to avoid clipping
    let s_adjusted = s * (1.0 - (2.0 * lightness - 1.0).abs() * 0.5);
    let (r, g, b) = hsl_to_rgb(h, s_adjusted, lightness);
    Vec4::new(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsl_roundtrip() {
        let cases = [
            (1.0, 0.0, 0.0), // red
            (0.0, 1.0, 0.0), // green
            (0.0, 0.0, 1.0), // blue
            (0.5, 0.5, 0.5), // grey
            (1.0, 1.0, 1.0), // white
            (0.0, 0.0, 0.0), // black
        ];
        for (r, g, b) in cases {
            let (h, s, l) = rgb_to_hsl(r, g, b);
            let (r2, g2, b2) = hsl_to_rgb(h, s, l);
            assert!(
                (r - r2).abs() < 1e-4 && (g - g2).abs() < 1e-4 && (b - b2).abs() < 1e-4,
                "Roundtrip failed for ({r}, {g}, {b}) -> ({h}, {s}, {l}) -> ({r2}, {g2}, {b2})"
            );
        }
    }

    #[test]
    fn test_rgb_to_hsl_red() {
        let (h, s, l) = rgb_to_hsl(1.0, 0.0, 0.0);
        assert!((h - 0.0).abs() < 1e-4, "Red hue should be ~0.0, got {h}");
        assert!(
            (s - 1.0).abs() < 1e-4,
            "Red saturation should be 1.0, got {s}"
        );
        assert!(
            (l - 0.5).abs() < 1e-4,
            "Red lightness should be 0.5, got {l}"
        );
    }

    #[test]
    fn test_rgb_to_hsl_grey() {
        let (_, s, _) = rgb_to_hsl(0.5, 0.5, 0.5);
        assert!(s < 1e-4, "Grey should have zero saturation, got {s}");
    }

    #[test]
    fn test_tone_produces_valid_colors() {
        let color = tone(0.6, 0.5, 0.4);
        assert!(color.x >= 0.0 && color.x <= 1.0);
        assert!(color.y >= 0.0 && color.y <= 1.0);
        assert!(color.z >= 0.0 && color.z <= 1.0);
        assert_eq!(color.w, 1.0);
    }

    #[test]
    fn test_tone_extreme_lightness() {
        // Very light (near white)
        let light = tone(0.3, 0.8, 0.99);
        assert!(light.x > 0.9);
        assert!(light.y > 0.9);
        assert!(light.z > 0.9);

        // Very dark (near black)
        let dark = tone(0.3, 0.8, 0.01);
        assert!(dark.x < 0.1);
        assert!(dark.y < 0.1);
        assert!(dark.z < 0.1);
    }

    #[test]
    fn test_from_seed_light_produces_valid_colors() {
        let seed = Vec4::new(0.4, 0.3, 0.9, 1.0);
        let scheme = ColorScheme::from_seed_light(seed);

        // All alpha channels should be 1.0
        assert_eq!(scheme.primary.w, 1.0);
        assert_eq!(scheme.on_primary.w, 1.0);
        assert_eq!(scheme.surface.w, 1.0);

        // Surface should be very light
        assert!(
            scheme.surface.x > 0.9,
            "Light surface should be bright, got {}",
            scheme.surface.x
        );

        // On-surface should be very dark
        assert!(
            scheme.on_surface.x < 0.15,
            "On-surface should be dark, got {}",
            scheme.on_surface.x
        );

        // Primary should be saturated (meaningful difference between channels)
        let max_ch = scheme.primary.x.max(scheme.primary.y).max(scheme.primary.z);
        let min_ch = scheme.primary.x.min(scheme.primary.y).min(scheme.primary.z);
        assert!(
            max_ch - min_ch > 0.1,
            "Primary should be saturated, diff={}",
            max_ch - min_ch
        );
    }

    #[test]
    fn test_from_seed_dark_produces_valid_colors() {
        let seed = Vec4::new(0.4, 0.3, 0.9, 1.0);
        let scheme = ColorScheme::from_seed_dark(seed);

        // Dark surface should be very dark
        assert!(
            scheme.surface.x < 0.15,
            "Dark surface should be dim, got {}",
            scheme.surface.x
        );

        // On-surface should be very light
        assert!(
            scheme.on_surface.x > 0.85,
            "Dark on-surface should be bright, got {}",
            scheme.on_surface.x
        );
    }

    #[test]
    fn test_different_seeds_produce_different_primaries() {
        let red = ColorScheme::from_seed_light(Vec4::new(0.9, 0.2, 0.2, 1.0));
        let blue = ColorScheme::from_seed_light(Vec4::new(0.2, 0.2, 0.9, 1.0));
        let diff = (red.primary - blue.primary).length();
        assert!(
            diff > 0.1,
            "Different seeds should produce different primaries, diff={diff}"
        );
    }

    #[test]
    fn test_error_colors_are_red_toned() {
        let scheme = ColorScheme::from_seed_light(Vec4::new(0.2, 0.8, 0.2, 1.0));
        assert!(
            scheme.error.x > scheme.error.y,
            "Error should be red-dominant: r={} > g={}",
            scheme.error.x,
            scheme.error.y
        );
        assert!(
            scheme.error.x > scheme.error.z,
            "Error should be red-dominant: r={} > b={}",
            scheme.error.x,
            scheme.error.z
        );
    }

    #[test]
    fn test_scrim_is_black() {
        let scheme = ColorScheme::from_seed_light(Vec4::new(0.5, 0.5, 0.5, 1.0));
        assert!(scheme.scrim.x < 0.05);
        assert!(scheme.scrim.y < 0.05);
        assert!(scheme.scrim.z < 0.05);
        assert_eq!(scheme.scrim.w, 1.0);
    }

    #[test]
    fn test_shadow_is_black() {
        let scheme = ColorScheme::from_seed_dark(Vec4::new(0.3, 0.6, 0.9, 1.0));
        assert!(scheme.shadow.x < 0.05);
        assert!(scheme.shadow.y < 0.05);
        assert!(scheme.shadow.z < 0.05);
        assert_eq!(scheme.shadow.w, 1.0);
    }

    #[test]
    fn test_light_container_lighter_than_primary() {
        let scheme = ColorScheme::from_seed_light(Vec4::new(0.4, 0.3, 0.9, 1.0));
        let primary_lum = scheme.primary.x + scheme.primary.y + scheme.primary.z;
        let container_lum =
            scheme.primary_container.x + scheme.primary_container.y + scheme.primary_container.z;
        assert!(
            container_lum > primary_lum,
            "Container ({container_lum}) should be lighter than primary ({primary_lum})"
        );
    }

    #[test]
    fn test_dark_container_darker_than_primary() {
        let scheme = ColorScheme::from_seed_dark(Vec4::new(0.4, 0.3, 0.9, 1.0));
        let primary_lum = scheme.primary.x + scheme.primary.y + scheme.primary.z;
        let container_lum =
            scheme.primary_container.x + scheme.primary_container.y + scheme.primary_container.z;
        assert!(
            container_lum < primary_lum,
            "Dark container ({container_lum}) should be darker than primary ({primary_lum})"
        );
    }

    #[test]
    fn test_tertiary_hue_shifted_from_primary() {
        // Use a seed with clear hue
        let seed = Vec4::new(0.9, 0.2, 0.2, 1.0); // red seed
        let scheme = ColorScheme::from_seed_light(seed);

        // Tertiary should have a different dominant channel than primary
        // (hue shifted +60 degrees from red should be towards yellow)
        // We just check they're different enough
        let diff = (scheme.primary - scheme.tertiary).length();
        assert!(
            diff > 0.05,
            "Tertiary should differ from primary, diff={diff}"
        );
    }

    #[test]
    fn test_inverse_surface_contrast() {
        let light = ColorScheme::from_seed_light(Vec4::new(0.4, 0.3, 0.9, 1.0));
        // Inverse surface should be dark in a light theme
        let inv_lum = light.inverse_surface.x + light.inverse_surface.y + light.inverse_surface.z;
        assert!(
            inv_lum < 1.0,
            "Inverse surface should be dark in light theme, lum={inv_lum}"
        );

        // Inverse on-surface should be light in a light theme
        let inv_on_lum =
            light.inverse_on_surface.x + light.inverse_on_surface.y + light.inverse_on_surface.z;
        assert!(
            inv_on_lum > 2.0,
            "Inverse on-surface should be light in light theme, lum={inv_on_lum}"
        );
    }

    #[test]
    fn test_neutral_grey_seed_still_produces_palette() {
        // Even a pure grey seed should produce a usable scheme
        let scheme = ColorScheme::from_seed_light(Vec4::new(0.5, 0.5, 0.5, 1.0));
        assert_eq!(scheme.primary.w, 1.0);
        assert!(scheme.surface.x > 0.9);
        assert!(scheme.on_surface.x < 0.15);
    }
}
