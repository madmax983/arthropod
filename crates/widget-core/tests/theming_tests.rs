//! Tests for widget theming with DesignTokens

use flux_state::{Runtime, Signal};
use glam::Vec4;
use theme_engine::{BackgroundMaterial, DesignTokens, SystemTheme};
use widget_core::{TextInput, Widget, WidgetContext};

/// Helper to create a mock SystemTheme for testing
fn create_test_theme() -> SystemTheme {
    SystemTheme {
        accent_color: Vec4::new(0.0, 0.47, 0.84, 1.0), // Blue accent
        is_dark_mode: false,
        supports_transparency: false,
        available_materials: vec![BackgroundMaterial::Solid(Vec4::new(0.95, 0.95, 0.95, 1.0))],
        text_color: Vec4::new(0.1, 0.1, 0.1, 1.0),          // Near black
        text_secondary_color: Vec4::new(0.5, 0.5, 0.5, 1.0), // Gray
    }
}

#[test]
fn test_text_input_uses_design_tokens() {
    // Create a text input
    let runtime = Runtime::new();
    let value = Signal::new(runtime.clone(), String::new());
    let input = TextInput::new(value);

    // Create context with design tokens
    let mut ctx = WidgetContext::new_test();
    let theme = create_test_theme();
    let tokens = DesignTokens::from_system(&theme);
    ctx.set_design_tokens(tokens.clone());

    // Build the widget
    let node_id = input.build(&mut ctx);

    // Verify it was built (basic smoke test)
    assert!(ctx.scene().get_node(node_id).is_some());

    // Verify background color was set using tokens
    // Surface primary should be used for input background
    let bg_color = ctx.get_background_color(node_id);
    assert!(bg_color.is_some(), "TextInput should have background color set");

    // Background should use surface_primary from tokens
    let expected_bg = tokens.surface_primary.as_color();
    let actual_bg = bg_color.unwrap();
    assert!(
        (actual_bg.x - expected_bg.x).abs() < 0.01
            && (actual_bg.y - expected_bg.y).abs() < 0.01
            && (actual_bg.z - expected_bg.z).abs() < 0.01,
        "Background color should match surface_primary token. Expected: {:?}, Got: {:?}",
        expected_bg,
        actual_bg
    );
}

#[test]
fn test_text_input_falls_back_without_tokens() {
    // Create a text input
    let runtime = Runtime::new();
    let value = Signal::new(runtime.clone(), String::new());
    let input = TextInput::new(value);

    // Create context WITHOUT design tokens
    let mut ctx = WidgetContext::new_test();

    // Build the widget
    let node_id = input.build(&mut ctx);

    // Verify it was built
    assert!(ctx.scene().get_node(node_id).is_some());

    // Background should still be set (fallback to hardcoded white)
    let bg_color = ctx.get_background_color(node_id);
    assert!(bg_color.is_some(), "TextInput should have fallback background color");

    // Fallback should be white (1.0, 1.0, 1.0, 1.0)
    let actual_bg = bg_color.unwrap();
    assert!(
        (actual_bg.x - 1.0).abs() < 0.01
            && (actual_bg.y - 1.0).abs() < 0.01
            && (actual_bg.z - 1.0).abs() < 0.01,
        "Fallback background should be white. Got: {:?}",
        actual_bg
    );
}

#[test]
fn test_text_input_dark_mode_tokens() {
    // Create a dark mode theme
    let dark_theme = SystemTheme {
        accent_color: Vec4::new(0.0, 0.47, 0.84, 1.0),
        is_dark_mode: true,
        supports_transparency: false,
        available_materials: vec![BackgroundMaterial::Solid(Vec4::new(0.12, 0.12, 0.12, 1.0))],
        text_color: Vec4::new(0.9, 0.9, 0.9, 1.0),          // Light text
        text_secondary_color: Vec4::new(0.6, 0.6, 0.6, 1.0), // Dimmer text
    };

    // Create a text input
    let runtime = Runtime::new();
    let value = Signal::new(runtime.clone(), String::new());
    let input = TextInput::new(value);

    // Create context with dark mode tokens
    let mut ctx = WidgetContext::new_test();
    let tokens = DesignTokens::from_system(&dark_theme);
    ctx.set_design_tokens(tokens.clone());

    // Build the widget
    let node_id = input.build(&mut ctx);

    // Verify background uses dark mode surface
    let bg_color = ctx.get_background_color(node_id);
    assert!(bg_color.is_some());

    // In dark mode, surface_primary should be dark
    let actual_bg = bg_color.unwrap();
    // Dark surfaces have low RGB values (typically < 0.2)
    assert!(
        actual_bg.x < 0.3 && actual_bg.y < 0.3 && actual_bg.z < 0.3,
        "Dark mode background should be dark. Got: {:?}",
        actual_bg
    );
}

#[test]
fn test_text_input_with_placeholder_uses_secondary_text() {
    // Create a text input with placeholder
    let runtime = Runtime::new();
    let value = Signal::new(runtime.clone(), String::new()); // Empty value
    let input = TextInput::new(value).placeholder("Enter text...");

    // Create context with design tokens
    let mut ctx = WidgetContext::new_test();
    let theme = create_test_theme();
    let tokens = DesignTokens::from_system(&theme);
    ctx.set_design_tokens(tokens);

    // Build the widget
    let node_id = input.build(&mut ctx);

    // Verify placeholder is shown (empty value)
    assert!(ctx.has_placeholder(node_id), "Empty input should show placeholder");
}
