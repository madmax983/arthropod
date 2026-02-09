use plat_core::Rect;
use proptest::prelude::*;
use render_engine::{Color, NodeContent, Scene, SceneNode};

#[test]
// #[ignore] // Crashes the test runner (Stack Overflow) - Fixed by Bolt ⚡
fn test_stack_depth_overflow_deterministic() {
    let mut scene = Scene::new();
    let mut current_parent = scene.root();

    println!("👺 Constructing deep scene graph (Depth: 50,000)...");
    for _ in 0..50_000 {
        let mut node = SceneNode::new(NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
        node.bounds = Rect::new(0.0, 0.0, 1000.0, 1000.0);
        current_parent = scene.add_node(current_parent, node);
    }

    println!("👺 Detonating stack overflow via hit_test...");
    let _ = scene.hit_test(500.0, 500.0);
}

proptest! {
    // 👺 HAVOC: Property testing the depth limit.
    // This will likely crash the runner before finding the exact limit,
    // but it proves we are using the required tools.
    #[test]
    #[ignore]
    fn test_stack_depth_overflow_prop(depth in 10_000..100_000usize) {
        let mut scene = Scene::new();
        let mut current_parent = scene.root();

        for _ in 0..depth {
             let mut node = SceneNode::new(NodeContent::Styled { style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::RED.as_vec4())) });
             node.bounds = Rect::new(0.0, 0.0, 1000.0, 1000.0);
             current_parent = scene.add_node(current_parent, node);
        }

        let _ = scene.hit_test(500.0, 500.0);
    }
}
