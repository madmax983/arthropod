//! TextInput widget tests - Written FIRST following TDD

use widget_core::{TextInput, WidgetContext, Widget};
use flux_state::{Runtime, Signal};

#[test]
fn test_text_input_creates_node() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), String::new());
    let input = TextInput::new(value);
    let node_id = input.build(&mut ctx);

    // Should create a node
    assert!(ctx.scene().get_node(node_id).is_some());
}

#[test]
fn test_text_input_displays_value() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), "Hello".to_string());
    let input = TextInput::new(value);
    let node_id = input.build(&mut ctx);

    // Should have text child displaying value
    let node = ctx.scene().get_node(node_id).unwrap();
    assert!(node.children.len() > 0, "Input should have text display");
}

#[test]
fn test_text_input_accepts_keyboard_input() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), "Hello".to_string());
    let input = TextInput::new(value);
    let node_id = input.build(&mut ctx);

    // Focus input
    ctx.focus_node(node_id);

    // Type a character
    ctx.send_char('!');

    // Value should update
    let final_value = ctx.get_text_input_value(node_id).unwrap();
    assert_eq!(final_value, "Hello!");
}

#[test]
fn test_text_input_cursor_position() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), "Hello".to_string());
    let input = TextInput::new(value);
    let node_id = input.build(&mut ctx);

    ctx.focus_node(node_id);

    // Cursor should start at end
    assert_eq!(ctx.get_cursor_position(node_id), Some(5));

    // Move cursor left
    ctx.send_key_left();
    assert_eq!(ctx.get_cursor_position(node_id), Some(4));

    // Move cursor right
    ctx.send_key_right();
    assert_eq!(ctx.get_cursor_position(node_id), Some(5));
}

#[test]
fn test_text_input_backspace() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), "Hello".to_string());
    let input = TextInput::new(value);
    let node_id = input.build(&mut ctx);

    ctx.focus_node(node_id);
    ctx.send_backspace();

    // Should remove last character
    let final_value = ctx.get_text_input_value(node_id).unwrap();
    assert_eq!(final_value, "Hell");
}

#[test]
fn test_text_input_delete() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), "Hello".to_string());
    let input = TextInput::new(value);
    let node_id = input.build(&mut ctx);

    ctx.focus_node(node_id);

    // Move cursor to start
    for _ in 0..5 {
        ctx.send_key_left();
    }

    // Delete should remove character after cursor
    ctx.send_delete();
    let final_value = ctx.get_text_input_value(node_id).unwrap();
    assert_eq!(final_value, "ello");
}

#[test]
fn test_text_input_validation() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), "ab".to_string());
    let input = TextInput::new(value)
        .validator(|s| {
            if s.len() < 3 {
                Err("Too short".to_string())
            } else {
                Ok(())
            }
        });

    let node_id = input.build(&mut ctx);

    // Should have validation error
    assert!(ctx.has_validation_error(node_id), "Should fail validation");
    assert_eq!(
        ctx.get_validation_error(node_id),
        Some("Too short".to_string())
    );
}

#[test]
fn test_text_input_validation_passes() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), "Hello".to_string());
    let input = TextInput::new(value)
        .validator(|s| {
            if s.len() < 3 {
                Err("Too short".to_string())
            } else {
                Ok(())
            }
        });

    let node_id = input.build(&mut ctx);

    // Should pass validation
    assert!(!ctx.has_validation_error(node_id), "Should pass validation");
}

#[test]
fn test_text_input_focus() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), String::new());
    let input = TextInput::new(value);
    let node_id = input.build(&mut ctx);

    // Initially not focused
    assert!(!ctx.is_focused(node_id), "Should not be focused initially");

    // Focus the input
    ctx.focus_node(node_id);
    assert!(ctx.is_focused(node_id), "Should be focused after focus_node");

    // Blur the input
    ctx.blur_node(node_id);
    assert!(!ctx.is_focused(node_id), "Should not be focused after blur");
}

#[test]
fn test_text_input_placeholder() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), String::new());
    let input = TextInput::new(value)
        .placeholder("Enter text...");

    let node_id = input.build(&mut ctx);

    // Should display placeholder when empty
    assert!(ctx.has_placeholder(node_id), "Should show placeholder");
}

#[test]
fn test_text_input_readonly() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), "Hello".to_string());
    let input = TextInput::new(value)
        .readonly(true);

    let node_id = input.build(&mut ctx);
    ctx.focus_node(node_id);

    // Try to type
    ctx.send_char('!');

    // Value should not change (readonly blocks input)
    let final_value = ctx.get_text_input_value(node_id).unwrap();
    assert_eq!(final_value, "Hello");
}

#[test]
fn test_text_input_max_length() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let value = Signal::new(runtime.clone(), "Hello".to_string());
    let input = TextInput::new(value)
        .max_length(6);

    let node_id = input.build(&mut ctx);
    ctx.focus_node(node_id);

    // Try to add more characters
    ctx.send_char('!');
    let value_after_first = ctx.get_text_input_value(node_id).unwrap();
    assert_eq!(value_after_first, "Hello!"); // Should work

    ctx.send_char('!');
    let value_after_second = ctx.get_text_input_value(node_id).unwrap();
    assert_eq!(value_after_second, "Hello!"); // Should be blocked
}
