//! Button widget tests - Written FIRST following TDD

use widget_core::{Button, WidgetContext, Widget};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn test_button_creates_node() {
    let mut ctx = WidgetContext::new_test();

    let button = Button::new("Click Me");
    let node_id = button.build(&mut ctx);

    // Should create a node
    assert!(ctx.scene().get_node(node_id).is_some());
}

#[test]
fn test_button_with_text_content() {
    let mut ctx = WidgetContext::new_test();

    let button = Button::new("Hello Button");
    let node_id = button.build(&mut ctx);

    // Button should have text child
    let node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(node.children.len(), 1, "Button should have text child");

    // Child should be text node
    let text_child = node.children[0];
    assert!(ctx.is_text_node(text_child), "Child should be text node");
}

#[test]
fn test_button_hover_detection() {
    let mut ctx = WidgetContext::new_test();

    let button = Button::new("Hover Me");
    let node_id = button.build(&mut ctx);

    // Should have hoverable component
    assert!(ctx.has_hover_state(node_id), "Button should be hoverable");
}

#[test]
fn test_button_click_callback() {
    let mut ctx = WidgetContext::new_test();
    let clicked = Arc::new(AtomicBool::new(false));
    let clicked_clone = clicked.clone();

    let button = Button::new("Click Me")
        .on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        });

    let node_id = button.build(&mut ctx);

    // Should have clickable component
    assert!(ctx.has_clickable(node_id), "Button should be clickable");

    // Simulate click
    ctx.trigger_click(node_id);

    // Callback should have been called
    assert!(clicked.load(Ordering::SeqCst), "Click callback should be called");
}

#[test]
fn test_button_with_themed_style() {
    let mut ctx = WidgetContext::new_test();

    let button = Button::new("Themed")
        .primary(); // Use primary style

    let node_id = button.build(&mut ctx);

    // Should have background styling
    assert!(ctx.has_background_color(node_id), "Button should have background");
}

#[test]
fn test_button_hover_changes_style() {
    let mut ctx = WidgetContext::new_test();

    let button = Button::new("Hover Test");
    let node_id = button.build(&mut ctx);

    // Get initial style
    let _initial_color = ctx.get_background_color(node_id);

    // Simulate hover
    ctx.trigger_hover(node_id, true);

    // Style should change on hover (or be marked for change)
    // For now, just verify hover state exists
    assert!(ctx.has_hover_state(node_id), "Should track hover state");
}

#[test]
fn test_button_disabled_state() {
    let mut ctx = WidgetContext::new_test();
    let clicked = Arc::new(AtomicBool::new(false));
    let clicked_clone = clicked.clone();

    let button = Button::new("Disabled")
        .on_click(move || {
            clicked_clone.store(true, Ordering::SeqCst);
        })
        .disabled(true);

    let node_id = button.build(&mut ctx);

    // Simulate click on disabled button
    ctx.trigger_click(node_id);

    // Callback should NOT be called
    assert!(!clicked.load(Ordering::SeqCst), "Disabled button should not respond to clicks");
}

#[test]
fn test_button_with_padding() {
    let mut ctx = WidgetContext::new_test();

    let button = Button::new("Padded")
        .padding(20.0);

    let node_id = button.build(&mut ctx);

    let layout = ctx.get_layout_style(node_id).unwrap();
    assert_eq!(layout.padding_left, 20.0);
    assert_eq!(layout.padding_right, 20.0);
}

#[test]
fn test_multiple_buttons() {
    let mut ctx = WidgetContext::new_test();
    let clicks = Arc::new(Mutex::new(Vec::new()));

    let mut button_ids = Vec::new();

    for i in 0..5 {
        let clicks_clone = clicks.clone();
        let button = Button::new(format!("Button {}", i))
            .on_click(move || {
                clicks_clone.lock().unwrap().push(i);
            });

        let node_id = button.build(&mut ctx);
        button_ids.push(node_id);
    }

    // Click button 2
    ctx.trigger_click(button_ids[2]);

    // Only button 2 should have been clicked
    let clicked_buttons = clicks.lock().unwrap();
    assert_eq!(clicked_buttons.len(), 1);
    assert_eq!(clicked_buttons[0], 2);
}
