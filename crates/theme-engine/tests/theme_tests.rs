//! Integration tests for theme-engine
//!
//! Following TDD: These tests are written BEFORE implementation

use theme_engine::{BackgroundMaterial, Color, DesignTokens, SystemTheme};

#[test]
#[cfg(target_os = "windows")]
fn test_system_theme_queries_windows_accent() {
    // Test: SystemTheme should query Windows registry for accent color
    let theme = SystemTheme::query().expect("Should query system theme");

    // Should return valid RGBA values
    assert!(theme.accent_color.x >= 0.0 && theme.accent_color.x <= 1.0);
    assert!(theme.accent_color.y >= 0.0 && theme.accent_color.y <= 1.0);
    assert!(theme.accent_color.z >= 0.0 && theme.accent_color.z <= 1.0);
    assert!(theme.accent_color.w >= 0.0 && theme.accent_color.w <= 1.0);
}

#[test]
#[cfg(target_os = "windows")]
fn test_system_theme_detects_mica_support() {
    // Test: Should detect if Windows 11 supports Mica
    let theme = SystemTheme::query().expect("Should query system theme");

    // On Windows 11+, should support transparency
    // This is a capability check, not a strict requirement
    let _ = theme.supports_transparency;
}

#[test]
#[cfg(target_os = "windows")]
fn test_system_theme_queries_dark_mode() {
    // Test: Should detect system dark mode preference
    let theme = SystemTheme::query().expect("Should query system theme");

    // Should return a valid boolean
    let _ = theme.is_dark_mode;
}

#[test]
fn test_design_tokens_from_system() {
    // Test: DesignTokens should resolve from SystemTheme
    let theme = SystemTheme::query().expect("Should query system theme");
    let tokens = DesignTokens::from_system(&theme);

    // Should have resolved surface tokens
    // The exact material depends on platform/OS version, but should be Some
    assert!(matches!(
        tokens.surface_primary,
        theme_engine::TokenValue::Material(_)
    ));
}

#[test]
fn test_design_tokens_spacing_scale() {
    // Test: DesignTokens should provide spacing scale
    let theme = SystemTheme::query().expect("Should query system theme");
    let tokens = DesignTokens::from_system(&theme);

    // Spacing should follow 8px base scale
    assert_eq!(tokens.space_xs, 4.0);
    assert_eq!(tokens.space_sm, 8.0);
    assert_eq!(tokens.space_md, 16.0);
    assert_eq!(tokens.space_lg, 24.0);
    assert_eq!(tokens.space_xl, 32.0);
}

#[test]
fn test_design_tokens_radius_scale() {
    // Test: DesignTokens should provide border radius scale
    let theme = SystemTheme::query().expect("Should query system theme");
    let tokens = DesignTokens::from_system(&theme);

    // Radius should be reasonable values
    assert!(tokens.radius_sm >= 2.0);
    assert!(tokens.radius_md >= 4.0);
    assert!(tokens.radius_lg >= 8.0);
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_material_types() {
    // Test: Windows materials should be available
    let theme = SystemTheme::query().expect("Should query system theme");

    // Should expose available materials
    assert!(theme.available_materials.len() > 0);

    // Should include at least Acrylic (available on Win10+)
    let has_acrylic = theme
        .available_materials
        .iter()
        .any(|m| matches!(m, BackgroundMaterial::Windows(_)));
    assert!(has_acrylic, "Should support at least one Windows material");
}

#[test]
fn test_design_tokens_color_semantics() {
    // Test: Semantic color tokens should be defined
    let theme = SystemTheme::query().expect("Should query system theme");
    let tokens = DesignTokens::from_system(&theme);

    // Should have text colors
    assert!(tokens.text_primary.w > 0.0); // Alpha should be set
    assert!(tokens.text_secondary.w > 0.0);

    // Should have surface colors
    assert!(tokens.surface_secondary.as_color().w > 0.0);
}
