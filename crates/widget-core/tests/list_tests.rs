//! List widget tests - Written FIRST following TDD

use widget_core::{List, Text, Widget, WidgetContext};

#[test]
fn test_list_column_empty() {
    let mut ctx = WidgetContext::new_test();

    let list = List::column();
    let node_id = list.build(&mut ctx);

    // Verify scene node was created
    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(
        scene_node.children.len(),
        0,
        "Empty list should have no children"
    );
}

#[test]
fn test_list_column_with_items() {
    let mut ctx = WidgetContext::new_test();

    let mut list = List::column();
    list.push(Text::new("Item 1"));
    list.push(Text::new("Item 2"));
    list.push(Text::new("Item 3"));

    let node_id = list.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 3, "List should have 3 children");
}

#[test]
fn test_list_row_with_items() {
    let mut ctx = WidgetContext::new_test();

    let mut list = List::row();
    list.push(Text::new("A"));
    list.push(Text::new("B"));

    let node_id = list.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 2, "List should have 2 children");

    // Verify it's a row layout
    let layout = ctx.get_layout_style(node_id).unwrap();
    assert!(
        layout.direction == layout_engine::FlexDirection::Row,
        "Should be row layout"
    );
}

#[test]
fn test_list_extend() {
    let mut ctx = WidgetContext::new_test();

    let items = ["A", "B", "C", "D"];
    let mut list = List::column();
    list.extend(items.iter().map(|s| Text::new(*s)));

    let node_id = list.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 4, "List should have 4 children");
}

#[test]
fn test_list_with_gap() {
    let mut ctx = WidgetContext::new_test();

    let mut list = List::column().gap(10.0);
    list.push(Text::new("Item 1"));
    list.push(Text::new("Item 2"));

    let node_id = list.build(&mut ctx);

    let layout = ctx.get_layout_style(node_id).unwrap();
    assert_eq!(layout.gap, 10.0, "Gap should be 10.0");
}

#[test]
fn test_list_with_padding() {
    let mut ctx = WidgetContext::new_test();

    let mut list = List::column().padding(16.0);
    list.push(Text::new("Item"));

    let node_id = list.build(&mut ctx);

    let layout = ctx.get_layout_style(node_id).unwrap();
    assert_eq!(layout.padding_top, 16.0);
    assert_eq!(layout.padding_bottom, 16.0);
    assert_eq!(layout.padding_left, 16.0);
    assert_eq!(layout.padding_right, 16.0);
}

#[test]
fn test_list_from_helper() {
    let mut ctx = WidgetContext::new_test();

    let items = ["Item 1", "Item 2", "Item 3"];
    let list = widget_core::list_from(items.iter().map(|s| Text::new(*s)));

    let node_id = list.build(&mut ctx);

    let scene_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(scene_node.children.len(), 3, "List should have 3 children");
}
