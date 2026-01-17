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
    let node_id = scene.add_node(root, SceneNode::new(NodeContent::Rect { color }));

    let node = scene.get_node(node_id);
    assert!(node.is_some(), "Should be able to retrieve added node");

    if let Some(node) = node {
        match node.content {
            NodeContent::Rect { color: c } => {
                assert_eq!(c.r, color.r);
                assert_eq!(c.g, color.g);
                assert_eq!(c.b, color.b);
            }
            _ => panic!("Expected Rect content"),
        }
    }
}

#[test]
fn test_modify_node() {
    let mut scene = Scene::new();
    let root = scene.root();

    let node_id = scene.add_node(
        root,
        SceneNode::new(NodeContent::Rect { color: Color::RED }),
    );

    // Modify the node
    if let Some(node) = scene.get_node_mut(node_id) {
        node.content = NodeContent::Rect { color: Color::BLUE };
        node.opacity = 0.5;
    }

    // Verify modification
    let node = scene.get_node(node_id).expect("Node should exist");
    match node.content {
        NodeContent::Rect { color } => {
            assert_eq!(color.b, 1.0, "Color should be blue");
        }
        _ => panic!("Expected Rect content"),
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
    assert_eq!(retrieved.transform.matrix[0][2], 100.0);
    assert_eq!(retrieved.transform.matrix[1][2], 200.0);
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
    assert_eq!(Color::RED.r, 1.0);
    assert_eq!(Color::RED.g, 0.0);
    assert_eq!(Color::GREEN.g, 1.0);
    assert_eq!(Color::BLUE.b, 1.0);
    assert_eq!(Color::WHITE.r, 1.0);
    assert_eq!(Color::WHITE.g, 1.0);
    assert_eq!(Color::WHITE.b, 1.0);
    assert_eq!(Color::BLACK.r, 0.0);
}

#[test]
fn test_transform_identity() {
    let t = Transform2D::IDENTITY;
    assert_eq!(t.matrix[0][0], 1.0);
    assert_eq!(t.matrix[1][1], 1.0);
    assert_eq!(t.matrix[0][2], 0.0);
    assert_eq!(t.matrix[1][2], 0.0);
}

#[test]
fn test_transform_translate() {
    let t = Transform2D::translate(10.0, 20.0);
    assert_eq!(t.matrix[0][2], 10.0);
    assert_eq!(t.matrix[1][2], 20.0);
}

#[test]
fn test_transform_scale() {
    let t = Transform2D::scale(2.0, 3.0);
    assert_eq!(t.matrix[0][0], 2.0);
    assert_eq!(t.matrix[1][1], 3.0);
}
