//! Sentry Tests for Scene Graph Correctness
//!
//! "If it isn't tested, it's broken." - Sentry

use plat_core::Rect;
use render_engine::{Color, NodeContent, Scene, SceneNode, VisualStyle};

#[test]
fn test_strict_nested_hit_test() {
    let mut scene = Scene::new();
    let root = scene.root();

    // Parent container (Bottom)
    let mut parent = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().solid_fill(Color::WHITE.as_vec4())),
    });
    parent.bounds = Rect::new(0.0, 0.0, 200.0, 200.0);
    let parent_id = scene.add_node(root, parent);

    // Child inside parent (Top)
    let mut child = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
    });
    child.bounds = Rect::new(50.0, 50.0, 50.0, 50.0);
    let child_id = scene.add_node(parent_id, child);

    // Hit test in overlap region MUST return child (it is drawn on top)
    let hit = scene.hit_test(75.0, 75.0);

    // Sentry demands strictness. No "or" logic.
    assert_eq!(
        hit,
        Some(child_id),
        "Hit test failed strict Z-order check: Child should mask Parent"
    );
}

#[test]
#[should_panic(expected = "Parent node NodeId(1) does not exist")]
fn test_orphan_node_creation_panics() {
    let mut scene = Scene::new();
    let root = scene.root();

    // 1. Create a parent
    let parent_id = scene.add_node(root, SceneNode::new(NodeContent::Empty));

    // 2. Remove the parent
    scene.remove_node(parent_id);

    // 3. Verify parent is gone
    assert!(scene.get_node(parent_id).is_none());

    // 4. Attempt to add a node to the non-existent parent
    // This MUST panic to prevent creating a zombie node.
    scene.add_node(parent_id, SceneNode::new(NodeContent::Empty));
}
