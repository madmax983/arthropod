//! Flexbox layout tests - Written FIRST following TDD

use layout_engine::{FlexDirection, FlexStyle, LayoutConstraints, LayoutEngine};

#[test]
fn test_flex_row_distributes_space() {
    let mut engine = LayoutEngine::new();

    let root = engine.create_node(FlexStyle {
        direction: FlexDirection::Row,
        ..Default::default()
    });

    let child1 = engine.create_node(FlexStyle {
        flex_grow: 1.0,
        ..Default::default()
    });

    let child2 = engine.create_node(FlexStyle {
        flex_grow: 2.0,
        ..Default::default()
    });

    engine.add_child(root, child1);
    engine.add_child(root, child2);

    let constraints = LayoutConstraints {
        max_width: Some(300.0),
        max_height: Some(100.0),
        ..Default::default()
    };

    engine.compute_layout(root, constraints);

    let layout1 = engine.get_layout(child1).unwrap();
    let layout2 = engine.get_layout(child2).unwrap();

    // Child 1 should get 1/3 of space (100px)
    assert_eq!(layout1.width, 100.0);

    // Child 2 should get 2/3 of space (200px)
    assert_eq!(layout2.width, 200.0);
}

#[test]
fn test_flex_column_stacks_children() {
    let mut engine = LayoutEngine::new();

    let root = engine.create_node(FlexStyle {
        direction: FlexDirection::Column,
        ..Default::default()
    });

    let child1 = engine.create_node(FlexStyle {
        flex_grow: 1.0,
        ..Default::default()
    });

    let child2 = engine.create_node(FlexStyle {
        flex_grow: 1.0,
        ..Default::default()
    });

    engine.add_child(root, child1);
    engine.add_child(root, child2);

    let constraints = LayoutConstraints {
        max_width: Some(100.0),
        max_height: Some(200.0),
        ..Default::default()
    };

    engine.compute_layout(root, constraints);

    let layout1 = engine.get_layout(child1).unwrap();
    let layout2 = engine.get_layout(child2).unwrap();

    // Each child should get half the height
    assert_eq!(layout1.height, 100.0);
    assert_eq!(layout2.height, 100.0);

    // Child 2 should be positioned below child 1
    assert_eq!(layout2.y, 100.0);
}

#[test]
fn test_flex_gap() {
    let mut engine = LayoutEngine::new();

    let root = engine.create_node(FlexStyle {
        direction: FlexDirection::Row,
        gap: 10.0,
        ..Default::default()
    });

    let child1 = engine.create_node(FlexStyle {
        width: Some(50.0),
        height: Some(50.0),
        ..Default::default()
    });

    let child2 = engine.create_node(FlexStyle {
        width: Some(50.0),
        height: Some(50.0),
        ..Default::default()
    });

    engine.add_child(root, child1);
    engine.add_child(root, child2);

    let constraints = LayoutConstraints {
        max_width: Some(200.0),
        max_height: Some(100.0),
        ..Default::default()
    };

    engine.compute_layout(root, constraints);

    let _layout1 = engine.get_layout(child1).unwrap();
    let layout2 = engine.get_layout(child2).unwrap();

    // Child 2 should be positioned 10px after child 1
    assert_eq!(layout2.x, 60.0); // 50px + 10px gap
}

#[test]
fn test_flex_padding() {
    let mut engine = LayoutEngine::new();

    let root = engine.create_node(FlexStyle {
        direction: FlexDirection::Column,
        padding_left: 10.0,
        padding_top: 20.0,
        ..Default::default()
    });

    let child = engine.create_node(FlexStyle {
        width: Some(50.0),
        height: Some(50.0),
        ..Default::default()
    });

    engine.add_child(root, child);

    let constraints = LayoutConstraints {
        max_width: Some(200.0),
        max_height: Some(200.0),
        ..Default::default()
    };

    engine.compute_layout(root, constraints);

    let layout = engine.get_layout(child).unwrap();

    // Child should be offset by padding
    assert_eq!(layout.x, 10.0);
    assert_eq!(layout.y, 20.0);
}
