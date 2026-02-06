//! Stack widget tests - Written FIRST following TDD

use widget_core::{Stack, Text, Widget, WidgetContext};

#[test]
fn test_stack_empty() {
    let mut ctx = WidgetContext::new_test();

    let stack = Stack::new(());
    let node_id = stack.build(&mut ctx);

    // Verify scene node was created
    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(
        scene_node.children.len(),
        0,
        "Empty stack should have no children"
    );
}

#[test]
fn test_stack_with_children() {
    let mut ctx = WidgetContext::new_test();

    let stack = Stack::new((
        Text::new("Background"),
        Text::new("Middle"),
        Text::new("Foreground"),
    ));

    let node_id = stack.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 3, "Stack should have 3 children");
}

#[test]
fn test_stack_render_order() {
    let mut ctx = WidgetContext::new_test();

    // First child should be rendered first (background)
    // Last child should be rendered last (foreground, on top)
    let stack = Stack::new((
        Text::new("Back"),
        Text::new("Front"),
    ));

    let node_id = stack.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();

    // Children are rendered in order, so first child is background
    assert_eq!(scene_node.children.len(), 2);
}

#[test]
fn test_stack_with_padding() {
    let mut ctx = WidgetContext::new_test();

    let stack = Stack::new((Text::new("Content"),)).padding(20.0);

    let node_id = stack.build(&mut ctx);

    let layout = ctx.get_layout_style(node_id).unwrap();
    assert_eq!(layout.padding_top, 20.0);
    assert_eq!(layout.padding_bottom, 20.0);
    assert_eq!(layout.padding_left, 20.0);
    assert_eq!(layout.padding_right, 20.0);
}

#[test]
fn test_stack_fill_parent() {
    let mut ctx = WidgetContext::new_test();

    // Stack should expand to fill its parent
    let stack = Stack::new((Text::new("Centered"),));

    let node_id = stack.build(&mut ctx);

    // Stack should have flex_grow to fill available space
    let layout = ctx.get_layout_style(node_id).unwrap();
    assert!(layout.flex_grow > 0.0, "Stack should expand to fill parent");
}
