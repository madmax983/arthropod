//! Debug test to understand taffy behavior

use layout_engine::{FlexDirection, FlexStyle, LayoutConstraints, LayoutEngine};

#[test]
fn debug_flex_grow() {
    let mut engine = LayoutEngine::new();

    let root = engine.create_node(FlexStyle {
        direction: FlexDirection::Row,
        ..Default::default()
    });

    let child1 = engine.create_node(FlexStyle {
        flex_grow: 1.0,
        ..Default::default()
    });

    engine.add_child(root, child1);

    let constraints = LayoutConstraints {
        max_width: Some(300.0),
        max_height: Some(100.0),
        ..Default::default()
    };

    engine.compute_layout(root, constraints);

    let root_layout = engine.get_layout(root).unwrap();
    let child_layout = engine.get_layout(child1).unwrap();

    println!("Root: {:?}", root_layout);
    println!("Child: {:?}", child_layout);
}
