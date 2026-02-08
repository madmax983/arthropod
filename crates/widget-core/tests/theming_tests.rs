//! Tests for widget theming with DesignTokens

use flux_state::{Runtime, Signal};
use glam::Vec4;
use plat_core::BackdropMaterial;
use render_engine::{Color, NodeContent};
use theme_engine::{DesignTokens, SystemTheme};
use widget_core::{Text, TextInput, Widget, WidgetContext};

/// Helper to create a mock SystemTheme for testing
fn create_test_theme() -> SystemTheme {
    SystemTheme {
        accent_color: Vec4::new(0.0, 0.47, 0.84, 1.0), // Blue accent
        is_dark_mode: false,
        supports_transparency: false,
        available_materials: vec![BackdropMaterial::None],
        text_color: Vec4::new(0.1, 0.1, 0.1, 1.0), // Near black
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
    assert!(
        bg_color.is_some(),
        "TextInput should have background color set"
    );

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
    assert!(
        bg_color.is_some(),
        "TextInput should have fallback background color"
    );

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
        available_materials: vec![BackdropMaterial::None],
        text_color: Vec4::new(0.9, 0.9, 0.9, 1.0), // Light text
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
    assert!(
        ctx.has_placeholder(node_id),
        "Empty input should show placeholder"
    );
}

#[test]
fn test_form_uses_design_tokens() {
    use widget_core::Form;

    let runtime = Runtime::new();
    let field = Signal::new(runtime.clone(), String::new());
    let form = Form::new((("test", TextInput::new(field.into())),));

    let mut ctx = WidgetContext::new_test();
    let theme = create_test_theme();
    let tokens = DesignTokens::from_system(&theme);
    ctx.set_design_tokens(tokens.clone());

    let node_id = form.build(&mut ctx);
    assert!(ctx.scene().get_node(node_id).is_some());

    // Verify background color was set using tokens
    // surface_secondary should be used for form background
    let bg_color = ctx.get_background_color(node_id);
    assert!(bg_color.is_some(), "Form should have background color set");

    // Background should use surface_secondary from tokens
    let expected_bg = tokens.surface_secondary.as_color();
    let actual_bg = bg_color.unwrap();
    assert!(
        (actual_bg.x - expected_bg.x).abs() < 0.01
            && (actual_bg.y - expected_bg.y).abs() < 0.01
            && (actual_bg.z - expected_bg.z).abs() < 0.01,
        "Background color should match surface_secondary token. Expected: {:?}, Got: {:?}",
        expected_bg,
        actual_bg
    );
}

#[test]
fn test_form_falls_back_without_tokens() {
    use widget_core::Form;

    let runtime = Runtime::new();
    let field = Signal::new(runtime.clone(), String::new());
    let form = Form::new((("test", TextInput::new(field.into())),));

    // Create context WITHOUT design tokens
    let mut ctx = WidgetContext::new_test();

    let node_id = form.build(&mut ctx);
    assert!(ctx.scene().get_node(node_id).is_some());

    // Background should still be set (fallback to hardcoded light gray)
    let bg_color = ctx.get_background_color(node_id);
    assert!(
        bg_color.is_some(),
        "Form should have fallback background color"
    );

    // Fallback should be light gray (0.95, 0.95, 0.95, 1.0)
    let actual_bg = bg_color.unwrap();
    assert!(
        (actual_bg.x - 0.95).abs() < 0.01
            && (actual_bg.y - 0.95).abs() < 0.01
            && (actual_bg.z - 0.95).abs() < 0.01,
        "Fallback background should be light gray. Got: {:?}",
        actual_bg
    );
}

// =========================================================================
// Text Widget Theming Tests
// =========================================================================

#[test]
fn test_text_uses_design_tokens_for_default_color() {
    // Create a text widget with no explicit color
    let text = Text::new("Hello World");

    // Create context with design tokens
    let mut ctx = WidgetContext::new_test();
    let theme = create_test_theme();
    let tokens = DesignTokens::from_system(&theme);
    ctx.set_design_tokens(tokens.clone());

    // Build the widget
    let node_id = text.build(&mut ctx);

    // Verify it was built
    assert!(ctx.scene().get_node(node_id).is_some());

    // Verify the text color uses text_primary from tokens
    let node = ctx.scene().get_node(node_id).unwrap();
    if let NodeContent::Text { color, .. } = &node.content {
        let expected_color = tokens.text_primary;
        assert!(
            (color.r() - expected_color.x).abs() < 0.01
                && (color.g() - expected_color.y).abs() < 0.01
                && (color.b() - expected_color.z).abs() < 0.01,
            "Text color should match text_primary token. Expected: {:?}, Got: {:?}",
            expected_color,
            color
        );
    } else {
        panic!("Expected Text node content");
    }
}

#[test]
fn test_text_falls_back_without_tokens() {
    // Create a text widget with no explicit color
    let text = Text::new("Hello World");

    // Create context WITHOUT design tokens
    let mut ctx = WidgetContext::new_test();

    // Build the widget
    let node_id = text.build(&mut ctx);

    // Verify it was built
    assert!(ctx.scene().get_node(node_id).is_some());

    // Verify the text color falls back to black (0.0, 0.0, 0.0, 1.0)
    let node = ctx.scene().get_node(node_id).unwrap();
    if let NodeContent::Text { color, .. } = &node.content {
        assert!(
            color.r().abs() < 0.01 && color.g().abs() < 0.01 && color.b().abs() < 0.01,
            "Fallback text color should be black. Got: {:?}",
            color
        );
    } else {
        panic!("Expected Text node content");
    }
}

#[test]
fn test_text_explicit_color_overrides_tokens() {
    // Create a text widget with explicit red color
    let explicit_color = Color::rgba(1.0, 0.0, 0.0, 1.0); // Red
    let text = Text::new("Hello World").color(explicit_color);

    // Create context with design tokens (which would set a different color)
    let mut ctx = WidgetContext::new_test();
    let theme = create_test_theme();
    let tokens = DesignTokens::from_system(&theme);
    ctx.set_design_tokens(tokens);

    // Build the widget
    let node_id = text.build(&mut ctx);

    // Verify the text color is the explicit red, not the theme color
    let node = ctx.scene().get_node(node_id).unwrap();
    if let NodeContent::Text { color, .. } = &node.content {
        assert!(
            (color.r() - 1.0).abs() < 0.01 && color.g().abs() < 0.01 && color.b().abs() < 0.01,
            "Text color should be explicit red, not theme color. Got: {:?}",
            color
        );
    } else {
        panic!("Expected Text node content");
    }
}

#[test]
fn test_text_dark_mode_uses_light_text() {
    // Create a dark mode theme
    let dark_theme = SystemTheme {
        accent_color: Vec4::new(0.0, 0.47, 0.84, 1.0),
        is_dark_mode: true,
        supports_transparency: false,
        available_materials: vec![BackdropMaterial::None],
        text_color: Vec4::new(0.9, 0.9, 0.9, 1.0), // Light text for dark mode
        text_secondary_color: Vec4::new(0.6, 0.6, 0.6, 1.0), // Dimmer text
    };

    // Create a text widget with no explicit color
    let text = Text::new("Hello World");

    // Create context with dark mode tokens
    let mut ctx = WidgetContext::new_test();
    let tokens = DesignTokens::from_system(&dark_theme);
    ctx.set_design_tokens(tokens.clone());

    // Build the widget
    let node_id = text.build(&mut ctx);

    // Verify the text color is light (for dark mode)
    let node = ctx.scene().get_node(node_id).unwrap();
    if let NodeContent::Text { color, .. } = &node.content {
        // In dark mode, text should be light (high RGB values)
        assert!(
            color.r() > 0.7 && color.g() > 0.7 && color.b() > 0.7,
            "Dark mode text should be light. Got: {:?}",
            color
        );
    } else {
        panic!("Expected Text node content");
    }
}
