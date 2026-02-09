use plat_core::Rect;
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};

#[test]
fn test_cycle_stack_overflow() {
    let mut scene = Scene::new();
    let root = scene.root();

    // Create Node A attached to Root
    let mut node_a = SceneNode::new(NodeContent::Styled {
        style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::RED.as_vec4())),
    });
    node_a.bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    let id_a = scene.add_node(root, node_a);

    println!("Attempting to reparent Root -> A (creating cycle)...");

    // We try to reparent the Root itself to A.
    // Root has no parent, so we pass a dummy ID for old_parent.
    // The implementation of reparent_node will fail to find the dummy old_parent (harmlessly),
    // but will proceed to add Root to A's children and set Root's parent to A.
    let dummy_id = NodeId(999999);
    scene.reparent_node(root, dummy_id, id_a);

    println!("Cycle created (if not prevented). Detonating...");

    // This checks children of Root (contains A), then children of A (contains Root)...
    // Stack overflow expected.
    let hit = scene.hit_test(50.0, 50.0);

    println!("Survived hit_test! Result: {:?}", hit);

    // Check if Root is child of A
    let a_node = scene.get_node(id_a).unwrap();
    assert!(
        !a_node.children.contains(&root),
        "Cycle detected: A contains Root as child!"
    );
}
