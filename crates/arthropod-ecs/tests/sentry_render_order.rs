use arthropod_ecs::{collect_renderables_system, RenderCommands, Renderable, SceneNodeRef};
use bevy_ecs::prelude::*;
use plat_core::Rect;
use render_engine::{Color, NodeContent, Scene, SceneNode, Transform2D, VisualStyle};

#[test]
fn test_parallel_execution_preserves_order() {
    let mut world = World::new();
    let mut scene = Scene::new();
    let root = scene.root();

    // Threshold is 1000. Use 2000 to force parallel execution.
    const COUNT: usize = 2000;

    let mut node_ids = Vec::with_capacity(COUNT);

    for i in 0..COUNT {
        // Create nodes with distinct colors to verify order
        // Color R channel encodes the index (normalized)
        let r = (i as f32) / (COUNT as f32);

        let node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new().solid_fill(Color::rgba(r, 0.0, 0.0, 1.0).as_vec4()),
                ),
            },
            transform: Transform2D::identity(),
            bounds: Rect::new(i as f32, 0.0, 10.0, 10.0),
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        let id = scene.add_node(root, node);
        node_ids.push(id);

        // Add to ECS
        world.spawn((SceneNodeRef(id), Renderable));
    }

    world.insert_resource(scene);
    world.insert_resource(RenderCommands::default());

    let mut schedule = Schedule::default();
    schedule.add_systems(collect_renderables_system);
    schedule.run(&mut world);

    let commands = world.resource::<RenderCommands>();

    assert_eq!(commands.0.len(), COUNT, "Should collect all renderables");

    // Verify order
    for (index, instance) in commands.0.iter().enumerate() {
        // Verify Position (x == index)
        // instance.pos is [f32; 2]
        assert_eq!(
            instance.pos[0], index as f32,
            "Position mismatch at index {}",
            index
        );

        // Verify Color
        let expected_r = (index as f32) / (COUNT as f32);
        // Allow small epsilon for float comparison
        assert!(
            (instance.color[0] - expected_r).abs() < 1e-5,
            "Color mismatch at index {}. Expected {}, Got {}",
            index,
            expected_r,
            instance.color[0]
        );
    }
}
