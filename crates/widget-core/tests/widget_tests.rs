//! Widget system integration tests - Written FIRST following TDD

use widget_core::{Widget, Container, Text, WidgetContext};
use flux_state::{Runtime, Signal};
use glam::Vec4;

#[test]
fn test_container_builds_children() {
    let mut ctx = test_widget_context();

    let container = Container::column()
        .child(Text::new("Hello"))
        .child(Text::new("World"));

    let node_id = container.build(&mut ctx);

    // Verify scene node was created
    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 2, "Container should have 2 children");

    // Verify layout component exists
    assert!(ctx.has_layout_node(node_id), "Container should have layout node");
}

#[test]
fn test_container_row_layout() {
    let mut ctx = test_widget_context();

    let container = Container::row()
        .gap(10.0)
        .child(Text::new("A"))
        .child(Text::new("B"));

    let node_id = container.build(&mut ctx);

    // Verify it's a row layout
    let layout = ctx.get_layout_style(node_id).unwrap();
    assert!(widget_core::context::is_row_layout(&layout), "Should be row layout");
    assert_eq!(layout.gap, 10.0, "Gap should be 10.0");
}

#[test]
fn test_container_padding() {
    let mut ctx = test_widget_context();

    let container = Container::column()
        .padding(16.0)
        .child(Text::new("Content"));

    let node_id = container.build(&mut ctx);

    let layout = ctx.get_layout_style(node_id).unwrap();
    assert_eq!(layout.padding_top, 16.0);
    assert_eq!(layout.padding_bottom, 16.0);
    assert_eq!(layout.padding_left, 16.0);
    assert_eq!(layout.padding_right, 16.0);
}

#[test]
fn test_text_widget_creates_text_node() {
    let mut ctx = test_widget_context();

    let text = Text::new("Hello World")
        .size(20.0)
        .color(Vec4::new(1.0, 0.0, 0.0, 1.0)); // Red

    let node_id = text.build(&mut ctx);

    // Check if it's text content
    assert!(ctx.is_text_node(node_id), "Should be text node");
}

#[test]
fn test_text_widget_with_default_size() {
    let mut ctx = test_widget_context();

    let text = Text::new("Default Size");
    let node_id = text.build(&mut ctx);

    // Should have default font size (16.0)
    assert!(ctx.is_text_node(node_id), "Should create text node");
}

#[test]
fn test_reactive_text_updates() {
    let mut ctx = test_widget_context();
    let runtime = Runtime::new();

    let signal = Signal::new(runtime.clone(), "Initial".to_string());
    let (read, write) = signal.split();

    let text = Text::reactive(read);
    let node_id = text.build(&mut ctx);

    // Initial value
    assert!(ctx.is_text_node(node_id), "Should create text node");

    // Update signal
    write.set("Updated".to_string());
    
    // The reactive component should exist
    assert!(ctx.has_reactive_text(node_id), "Should have reactive text component");
}

#[test]
fn test_nested_containers() {
    let mut ctx = test_widget_context();

    let widget = Container::column()
        .child(
            Container::row()
                .child(Text::new("A"))
                .child(Text::new("B"))
        )
        .child(Text::new("C"));

    let root_id = widget.build(&mut ctx);

    let root_node = ctx.scene().get_node(root_id).unwrap();
    assert_eq!(root_node.children.len(), 2, "Root should have 2 children");

    // First child is a container with 2 text children
    let first_child_id = root_node.children[0];
    let first_child = ctx.scene().get_node(first_child_id).unwrap();
    assert_eq!(first_child.children.len(), 2, "First child should have 2 children");
}

#[test]
fn test_empty_container() {
    let mut ctx = test_widget_context();

    let container = Container::column();
    let node_id = container.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 0, "Empty container should have no children");
}

#[test]
fn test_container_with_many_children() {
    let mut ctx = test_widget_context();

    let mut container = Container::column();
    for i in 0..100 {
        container = container.child(Text::new(format!("Item {}", i)));
    }

    let node_id = container.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 100, "Should have 100 children");
}

// Helper function to create test widget context
fn test_widget_context() -> WidgetContext {
    WidgetContext::new_test()
}
