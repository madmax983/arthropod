use arthropod::prelude::*;
use glam::Vec2;
use render_engine::{Color, NodeContent, Scene, SceneNode};
use spatial_graph::{SpatialNode, Stats, Viewport, spatial_transform_system, stats_system};
use std::time::{Duration, Instant};

#[derive(Resource)]
struct AnimationState {
    start_time: Instant,
}

fn setup_scene(mut commands: Commands, mut scene: ResMut<Scene>) {
    let root = scene.root();

    // Create 100 random nodes
    for i in 0..100 {
        let x = (i % 10) as f32 * 200.0 - 1000.0;
        let y = (i / 10) as f32 * 200.0 - 1000.0;

        let color = if i % 2 == 0 { Color::RED } else { Color::BLUE };

        // Create SceneNode (visual)
        let node = SceneNode::new(NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(color.as_vec4())),
        });
        let node_id = scene.add_node(root, node);

        // Spawn ECS entity with SpatialNode
        commands.spawn((
            arthropod_ecs::SceneNodeRef(node_id),
            arthropod_ecs::Renderable,
            SpatialNode {
                position: Vec2::new(x, y),
                size: Vec2::new(100.0, 100.0),
            },
        ));
    }
}

fn animate_camera(mut viewport: ResMut<Viewport>, anim: Res<AnimationState>) {
    let elapsed = anim.start_time.elapsed().as_secs_f32();

    // Pan in a circle
    viewport.camera.position = Vec2::new(f32::sin(elapsed) * 500.0, f32::cos(elapsed) * 500.0);

    // Zoom in and out
    viewport.camera.zoom = 1.0 + f32::sin(elapsed * 0.5) * 0.5;
}

fn main() -> Result<(), AppError> {
    // We use new_headless for CI/CD environments where display might not be available
    let mut app = App::new_headless()?;

    app.world_mut().insert_resource(Viewport::default());
    app.world_mut().insert_resource(Stats::default());
    app.world_mut().insert_resource(AnimationState {
        start_time: Instant::now(),
    });

    app.add_update_system(setup_scene.run_if(run_once));
    app.add_update_system(animate_camera);
    app.add_update_system(spatial_transform_system);
    app.add_update_system(stats_system);

    // Run a few frames
    for _ in 0..10 {
        app.update();
        let instances = app.render();
        let stats = app.world().resource::<Stats>();
        println!(
            "Rendered {} instances | Stats: Visible {} / Total {}",
            instances.len(),
            stats.visible_nodes,
            stats.total_nodes
        );
        std::thread::sleep(Duration::from_millis(16));
    }

    println!("Infinite Canvas example ran successfully!");

    Ok(())
}
