//! Layer 2: Design Tokens - Semantic design system
//!
//! Resolves semantic tokens (surface_primary, text_primary, etc.) from SystemTheme.
//! Provides consistent design language across the application.

use crate::{Color, SystemTheme};
use plat_core::BackdropMaterial;

/// A token value that can be either a color or a material
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    /// Solid color
    Color(Color),

    /// Platform-native material
    Material(BackdropMaterial),
}

impl TokenValue {
    /// Convert to color, resolving materials to their fallback color
    pub fn as_color(&self) -> Color {
        match self {
            TokenValue::Color(c) => *c,
            TokenValue::Material(BackdropMaterial::None) => {
                // Should ideally not happen if None is treated as "No special effect",
                // but we can return transparent or white/black depending on context.
                // For now, let's assume a generic fallback.
                Color::new(1.0, 1.0, 1.0, 1.0)
            }
            TokenValue::Material(BackdropMaterial::Mica) => {
                // Mica fallback color (semi-transparent)
                Color::new(0.95, 0.95, 0.95, 0.8)
            }
            TokenValue::Material(BackdropMaterial::Acrylic) => {
                // Acrylic fallback color
                Color::new(0.95, 0.95, 0.95, 0.7)
            }
            TokenValue::Material(BackdropMaterial::MicaAlt) => {
                // Mica Alt fallback color
                Color::new(0.9, 0.9, 0.9, 0.8)
            }
        }
    }
}

/// Semantic design tokens resolved from system theme
///
/// Provides a consistent design language with semantic naming:
/// - Surface tokens: Background colors/materials
/// - Text tokens: Foreground text colors
/// - Accent tokens: Interactive element colors
/// - Spacing tokens: Layout spacing values
/// - Radius tokens: Border radius values
#[derive(Debug, Clone)]
pub struct DesignTokens {
    // Surface tokens (backgrounds)
    pub surface_primary: TokenValue,
    pub surface_secondary: TokenValue,
    pub surface_elevated: TokenValue,

    // Text tokens
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,

    // Accent tokens
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,

    // Spacing tokens (8px base scale)
    pub space_xs: f32,
    pub space_sm: f32,
    pub space_md: f32,
    pub space_lg: f32,
    pub space_xl: f32,
    pub space_2xl: f32,

    // Border radius tokens
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
    pub radius_xl: f32,
}

impl DesignTokens {
    /// Create design tokens from system theme
    ///
    /// Resolves semantic tokens based on platform capabilities:
    /// - Windows 11: Uses Mica for primary surface
    /// - Windows 10: Uses Acrylic for primary surface
    /// - Other platforms: Uses solid colors
    ///
    /// # Example
    ///
    /// ```no_run
    /// use theme_engine::{SystemTheme, DesignTokens};
    ///
    /// let theme = SystemTheme::query()?;
    /// let tokens = DesignTokens::from_system(&theme);
    ///
    /// // Use tokens in UI
    /// let bg = tokens.surface_primary.as_color();
    /// # Ok::<(), theme_engine::ThemeError>(())
    /// ```
    pub fn from_system(theme: &SystemTheme) -> Self {
        // Choose best available material for primary surface
        let surface_primary = theme
            .available_materials
            .iter()
            .find(|m| matches!(m, BackdropMaterial::Mica))
            .or_else(|| {
                theme
                    .available_materials
                    .iter()
                    .find(|m| matches!(m, BackdropMaterial::Acrylic))
            })
            .map(|m| TokenValue::Material(*m))
            .unwrap_or_else(|| {
                TokenValue::Color(if theme.is_dark_mode {
                    Color::new(0.12, 0.12, 0.12, 1.0)
                } else {
                    Color::new(0.95, 0.95, 0.95, 1.0)
                })
            });

        // Secondary surface: slightly different from primary
        let surface_secondary = if theme.is_dark_mode {
            TokenValue::Color(Color::new(0.15, 0.15, 0.15, 1.0))
        } else {
            TokenValue::Color(Color::new(0.92, 0.92, 0.92, 1.0))
        };

        // Elevated surface: for modals, dropdowns
        let surface_elevated = theme
            .available_materials
            .iter()
            .find(|m| matches!(m, BackdropMaterial::MicaAlt))
            .or_else(|| {
                theme
                    .available_materials
                    .iter()
                    .find(|m| matches!(m, BackdropMaterial::Acrylic))
            })
            .map(|m| TokenValue::Material(*m))
            .unwrap_or_else(|| {
                TokenValue::Color(if theme.is_dark_mode {
                    Color::new(0.18, 0.18, 0.18, 1.0)
                } else {
                    Color::new(1.0, 1.0, 1.0, 1.0)
                })
            });

        // Text colors
        let text_primary = theme.text_color;
        let text_secondary = theme.text_secondary_color;
        let text_tertiary = if theme.is_dark_mode {
            Color::new(0.5, 0.5, 0.5, 1.0)
        } else {
            Color::new(0.6, 0.6, 0.6, 1.0)
        };

        // Accent colors
        let accent = theme.accent_color;
        let accent_hover = Self::lighten_color(accent, 0.1);
        let accent_pressed = Self::darken_color(accent, 0.1);

        Self {
            surface_primary,
            surface_secondary,
            surface_elevated,
            text_primary,
            text_secondary,
            text_tertiary,
            accent,
            accent_hover,
            accent_pressed,
            // 8px base spacing scale
            space_xs: 4.0,
            space_sm: 8.0,
            space_md: 16.0,
            space_lg: 24.0,
            space_xl: 32.0,
            space_2xl: 48.0,
            // Border radius scale
            radius_sm: 2.0,
            radius_md: 4.0,
            radius_lg: 8.0,
            radius_xl: 12.0,
        }
    }

    /// Lighten a color by a factor (0.0 - 1.0)
    fn lighten_color(color: Color, factor: f32) -> Color {
        Color::new(
            (color.x + (1.0 - color.x) * factor).min(1.0),
            (color.y + (1.0 - color.y) * factor).min(1.0),
            (color.z + (1.0 - color.z) * factor).min(1.0),
            color.w,
        )
    }

    /// Darken a color by a factor (0.0 - 1.0)
    fn darken_color(color: Color, factor: f32) -> Color {
        Color::new(
            (color.x * (1.0 - factor)).max(0.0),
            (color.y * (1.0 - factor)).max(0.0),
            (color.z * (1.0 - factor)).max(0.0),
            color.w,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_value_as_color() {
        let color = Color::new(1.0, 0.0, 0.0, 1.0);
        let token = TokenValue::Color(color);
        assert_eq!(token.as_color(), color);

        // Testing Material fallback is harder without duplicate logic, but we can test identity
        let material = TokenValue::Material(BackdropMaterial::Mica);
        assert!(material.as_color().w < 1.0); // Should be transparent
    }

    #[test]
    fn test_lighten_darken() {
        let color = Color::new(0.5, 0.5, 0.5, 1.0);

        let lighter = DesignTokens::lighten_color(color, 0.2);
        assert!(lighter.x > color.x);

        let darker = DesignTokens::darken_color(color, 0.2);
        assert!(darker.x < color.x);
    }

    #[test]
    fn test_design_tokens_spacing_scale() {
        let theme = SystemTheme::query().unwrap_or_else(|_| SystemTheme {
            accent_color: Color::new(0.0, 0.5, 1.0, 1.0),
            is_dark_mode: false,
            supports_transparency: false,
            available_materials: vec![],
            text_color: Color::new(0.0, 0.0, 0.0, 1.0),
            text_secondary_color: Color::new(0.5, 0.5, 0.5, 1.0),
        });

        let tokens = DesignTokens::from_system(&theme);

        // Verify spacing scale follows 8px base
        assert_eq!(tokens.space_xs, 4.0);
        assert_eq!(tokens.space_sm, 8.0);
        assert_eq!(tokens.space_md, 16.0);
        assert_eq!(tokens.space_lg, 24.0);
        assert_eq!(tokens.space_xl, 32.0);
    }
}
