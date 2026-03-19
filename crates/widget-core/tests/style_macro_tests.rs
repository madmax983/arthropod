//! Tests for style! macro with pseudo-state support
//!
//! Following TDD: These tests are written BEFORE implementation

use glam::Vec4;
use style_engine::CornerRadii;
use theme_engine::{DesignTokens, SystemTheme, TokenValue};
use widget_core::{style, style::Padding, Style, StyleOverrides};

#[test]
fn test_style_macro_basic() {
    let theme = SystemTheme::query().unwrap_or_else(|_| create_test_theme());
    let tokens = DesignTokens::from_system(&theme);

    let s = style! {
        background: tokens.surface_primary.clone();
        color: tokens.text_primary;
        padding: tokens.space_md;
        border_radius: tokens.radius_lg;
    };

    assert!(s.background.is_some());
    assert!(s.color.is_some());
    assert_eq!(s.padding, Some(Padding::uniform(16.0)));
    assert_eq!(s.border_radius, Some(CornerRadii::uniform(8.0)));
}

#[test]
fn test_style_macro_hover_state() {
    let theme = SystemTheme::query().unwrap_or_else(|_| create_test_theme());
    let tokens = DesignTokens::from_system(&theme);

    let s = style! {
        background: tokens.surface_primary.clone();

        &:hover {
            background: tokens.surface_secondary.clone();
        }
    };

    assert!(s.hover.is_some());
    let hover = s.hover.as_ref().unwrap();
    assert!(hover.background.is_some());
}

#[test]
fn test_style_macro_focus_state() {
    let theme = SystemTheme::query().unwrap_or_else(|_| create_test_theme());
    let tokens = DesignTokens::from_system(&theme);

    let s = style! {
        background: tokens.surface_primary.clone();

        &:focus {
            background: tokens.accent;
        }
    };

    assert!(s.focus.is_some());
    let focus = s.focus.as_ref().unwrap();
    assert!(focus.background.is_some());
}

#[test]
fn test_style_macro_active_state() {
    let theme = SystemTheme::query().unwrap_or_else(|_| create_test_theme());
    let tokens = DesignTokens::from_system(&theme);

    let s = style! {
        background: tokens.surface_primary.clone();

        &:active {
            background: tokens.accent_pressed;
        }
    };

    assert!(s.active.is_some());
    let active = s.active.as_ref().unwrap();
    assert!(active.background.is_some());
}

#[test]
fn test_style_macro_disabled_state() {
    let theme = SystemTheme::query().unwrap_or_else(|_| create_test_theme());
    let tokens = DesignTokens::from_system(&theme);

    let s = style! {
        background: tokens.accent;

        &:disabled {
            opacity: 0.5;
        }
    };

    assert!(s.disabled.is_some());
    let disabled = s.disabled.as_ref().unwrap();
    assert_eq!(disabled.opacity, Some(0.5));
}

#[test]
fn test_style_macro_multiple_states() {
    let theme = SystemTheme::query().unwrap_or_else(|_| create_test_theme());
    let tokens = DesignTokens::from_system(&theme);

    let s = style! {
        background: tokens.surface_primary.clone();
        opacity: 1.0;

        &:hover {
            background: tokens.surface_secondary.clone();
            opacity: 0.9;
        }

        &:focus {
            border_radius: 4.0;
        }

        &:disabled {
            opacity: 0.5;
        }
    };

    assert!(s.hover.is_some());
    assert!(s.focus.is_some());
    assert!(s.disabled.is_some());
}

#[test]
fn test_style_resolve_base_only() {
    let theme = SystemTheme::query().unwrap_or_else(|_| create_test_theme());
    let tokens = DesignTokens::from_system(&theme);

    let s = style! {
        background: tokens.surface_primary.clone();
        opacity: 1.0;
        border_radius: 8.0;
    };

    // Resolve with no states active
    let resolved = s.resolve(false, false, false, false);
    assert_eq!(resolved.opacity, 1.0);
    assert_eq!(resolved.border_radius, CornerRadii::uniform(8.0));
}

#[test]
fn test_style_resolve_hover() {
    let theme = SystemTheme::query().unwrap_or_else(|_| create_test_theme());
    let tokens = DesignTokens::from_system(&theme);

    let s = style! {
        background: tokens.surface_primary.clone();
        opacity: 1.0;

        &:hover {
            background: tokens.accent;
            opacity: 0.9;
        }
    };

    // Resolve with hover = true
    let resolved = s.resolve(true, false, false, false);
    assert_eq!(resolved.opacity, 0.9);
}

#[test]
fn test_style_resolve_disabled_overrides_hover() {
    let _theme = SystemTheme::query().unwrap_or_else(|_| create_test_theme());

    let s = style! {
        opacity: 1.0;

        &:hover {
            opacity: 0.9;
        }

        &:disabled {
            opacity: 0.5;
        }
    };

    // Resolve with both hover and disabled - disabled should take precedence
    let resolved = s.resolve(true, false, false, true);
    assert_eq!(resolved.opacity, 0.5);
}

#[test]
fn test_style_resolve_focus() {
    let s = style! {
        border_radius: 4.0;

        &:focus {
            border_radius: 8.0;
        }
    };

    // Resolve with focus = true
    let resolved = s.resolve(false, true, false, false);
    assert_eq!(resolved.border_radius, CornerRadii::uniform(8.0));
}

#[test]
fn test_style_resolve_active() {
    let s = style! {
        opacity: 1.0;

        &:active {
            opacity: 0.8;
        }
    };

    // Resolve with active = true
    let resolved = s.resolve(false, false, true, false);
    assert_eq!(resolved.opacity, 0.8);
}

#[test]
fn test_style_resolve_combined_states() {
    // When multiple states are active (not disabled), all should apply
    let s = style! {
        opacity: 1.0;
        border_radius: 4.0;

        &:hover {
            opacity: 0.9;
        }

        &:focus {
            border_radius: 8.0;
        }
    };

    // Resolve with hover and focus both true
    let resolved = s.resolve(true, true, false, false);
    assert_eq!(resolved.opacity, 0.9);
    assert_eq!(resolved.border_radius, CornerRadii::uniform(8.0));
}

#[test]
fn test_style_new() {
    let s = Style::new();
    assert!(s.background.is_none());
    assert!(s.color.is_none());
    assert!(s.padding.is_none());
    assert!(s.border_radius.is_none());
    assert!(s.opacity.is_none());
    assert!(s.hover.is_none());
    assert!(s.focus.is_none());
    assert!(s.active.is_none());
    assert!(s.disabled.is_none());
}

#[test]
fn test_style_overrides_default() {
    let overrides = StyleOverrides::default();
    assert!(overrides.background.is_none());
    assert!(overrides.color.is_none());
    assert!(overrides.opacity.is_none());
    assert!(overrides.border_radius.is_none());
}

#[test]
fn test_token_value_from_color() {
    let color = Vec4::new(1.0, 0.0, 0.0, 1.0);
    let token: TokenValue = color.into();
    assert!(matches!(token, TokenValue::Color(_)));
}

#[test]
fn test_token_value_identity() {
    // Test that cloned TokenValue equals original (basic identity check)
    let original = TokenValue::Color(Vec4::new(1.0, 0.0, 0.0, 1.0));
    let cloned = original.clone();
    assert_eq!(cloned, original);
}

#[test]
fn test_style_macro_with_color_literal() {
    let color = Vec4::new(0.5, 0.5, 0.5, 1.0);

    let s = style! {
        background: color;
        color: color;
    };

    assert!(s.background.is_some());
    assert!(s.color.is_some());
}

#[test]
fn test_resolved_style_default_opacity() {
    // If no opacity is specified, resolved style should default to 1.0
    let s = style! {
        border_radius: 8.0;
    };

    let resolved = s.resolve(false, false, false, false);
    assert_eq!(resolved.opacity, 1.0);
}

/// Helper to create a test theme for non-Windows platforms or fallback
fn create_test_theme() -> SystemTheme {
    use plat_core::BackdropMaterial;

    SystemTheme {
        accent_color: Vec4::new(0.0, 0.47, 0.84, 1.0),
        is_dark_mode: false,
        supports_transparency: false,
        available_materials: vec![BackdropMaterial::None],
        text_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        text_secondary_color: Vec4::new(0.4, 0.4, 0.4, 1.0),
    }
}
