//! Tests for scene graph data structures and operations.

use render_engine::{Color, NodeContent, Scene, SceneNode, Transform2D};

#[test]
fn test_scene_has_root() {
    let scene = Scene::new();
    let root = scene.root();

    assert!(
        scene.get_node(root).is_some(),
        "Scene should have a root node"
    );
}

#[test]
fn test_add_node_returns_unique_id() {
    let mut scene = Scene::new();
    let root = scene.root();

    let node1 = scene.add_node(root, SceneNode::new(NodeContent::Empty));
    let node2 = scene.add_node(root, SceneNode::new(NodeContent::Empty));

    assert_ne!(node1, node2, "Each node should have a unique ID");
}

#[test]
fn test_get_added_node() {
    let mut scene = Scene::new();
    let root = scene.root();

    let color = Color::RED;
    let node_id = scene.add_node(
        root,
        SceneNode::new(NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(color.as_vec4())),
        }),
    );

    let node = scene.get_node(node_id);
    assert!(node.is_some(), "Should be able to retrieve added node");

    if let Some(node) = node {
        match &node.content {
            NodeContent::Styled { style } => {
                assert!(!style.fills.is_empty(), "Expected at least one fill");
                if let render_engine::Paint::Solid(c) = &style.fills[0] {
                    assert_eq!(c.x, color.r());
                    assert_eq!(c.y, color.g());
                    assert_eq!(c.z, color.b());
                } else {
                    panic!("Expected solid fill color");
                }
            }
            _ => panic!("Expected Styled content"),
        }
    }
}

#[test]
fn test_modify_node() {
    let mut scene = Scene::new();
    let root = scene.root();

    let node_id = scene.add_node(
        root,
        SceneNode::new(NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        }),
    );

    // Modify the node
    if let Some(node) = scene.get_node_mut(node_id) {
        node.content = NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::BLUE.as_vec4())),
        };
        node.opacity = 0.5;
    }

    // Verify modification
    let node = scene.get_node(node_id).expect("Node should exist");
    match &node.content {
        NodeContent::Styled { style } => {
            assert!(!style.fills.is_empty(), "Expected at least one fill");
            if let render_engine::Paint::Solid(color) = &style.fills[0] {
                assert_eq!(color.z, 1.0, "Color should be blue");
            } else {
                panic!("Expected solid fill color");
            }
        }
        _ => panic!("Expected Styled content"),
    }
    assert_eq!(node.opacity, 0.5);
}

#[test]
fn test_node_has_transform() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut node = SceneNode::new(NodeContent::Empty);
    node.transform = Transform2D::translate(100.0, 200.0);

    let node_id = scene.add_node(root, node);

    let retrieved = scene.get_node(node_id).expect("Node should exist");
    let translation = retrieved.transform.translation();
    assert_eq!(translation.x, 100.0);
    assert_eq!(translation.y, 200.0);
}

#[test]
fn test_node_visibility() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut node = SceneNode::new(NodeContent::Empty);
    node.visible = false;

    let node_id = scene.add_node(root, node);

    let retrieved = scene.get_node(node_id).expect("Node should exist");
    assert!(!retrieved.visible, "Node should be invisible");

    // Make it visible
    if let Some(node) = scene.get_node_mut(node_id) {
        node.visible = true;
    }

    let retrieved = scene.get_node(node_id).expect("Node should exist");
    assert!(retrieved.visible, "Node should now be visible");
}

#[test]
fn test_dirty_tracking_on_add() {
    let mut scene = Scene::new();
    let root = scene.root();

    let node_id = scene.add_node(root, SceneNode::new(NodeContent::Empty));

    let dirty = scene.take_dirty();
    assert!(
        dirty.contains(&node_id),
        "Newly added node should be marked dirty"
    );
}

#[test]
fn test_dirty_tracking_manual_mark() {
    let mut scene = Scene::new();
    let root = scene.root();

    let node_id = scene.add_node(root, SceneNode::new(NodeContent::Empty));

    // Clear dirty list
    scene.take_dirty();

    // Manually mark dirty
    scene.mark_dirty(node_id);

    let dirty = scene.take_dirty();
    assert!(
        dirty.contains(&node_id),
        "Manually marked node should be in dirty list"
    );
}

#[test]
fn test_take_dirty_clears_list() {
    let mut scene = Scene::new();
    let root = scene.root();

    scene.add_node(root, SceneNode::new(NodeContent::Empty));

    let dirty1 = scene.take_dirty();
    assert!(!dirty1.is_empty(), "Should have dirty nodes");

    let dirty2 = scene.take_dirty();
    assert!(dirty2.is_empty(), "Dirty list should be cleared after take");
}

#[test]
fn test_color_constants() {
    assert_eq!(Color::RED.r(), 1.0);
    assert_eq!(Color::RED.g(), 0.0);
    assert_eq!(Color::GREEN.g(), 1.0);
    assert_eq!(Color::BLUE.b(), 1.0);
    assert_eq!(Color::WHITE.r(), 1.0);
    assert_eq!(Color::WHITE.g(), 1.0);
    assert_eq!(Color::WHITE.b(), 1.0);
    assert_eq!(Color::BLACK.r(), 0.0);
}

#[test]
fn test_transform_identity() {
    let t = Transform2D::IDENTITY;
    // Test that identity doesn't change points
    let point = glam::Vec2::new(5.0, 7.0);
    let transformed = t.transform_point(point);
    assert_eq!(transformed, point);
}

#[test]
fn test_transform_translate() {
    let t = Transform2D::translate(10.0, 20.0);
    let translation = t.translation();
    assert_eq!(translation.x, 10.0);
    assert_eq!(translation.y, 20.0);

    // Verify transformation works correctly
    let point = glam::Vec2::new(5.0, 7.0);
    let transformed = t.transform_point(point);
    assert_eq!(transformed, glam::Vec2::new(15.0, 27.0));
}

#[test]
fn test_transform_scale() {
    let t = Transform2D::scale(2.0, 3.0);

    // Verify scaling works correctly
    let point = glam::Vec2::new(5.0, 7.0);
    let transformed = t.transform_point(point);
    assert_eq!(transformed, glam::Vec2::new(10.0, 21.0));
}
