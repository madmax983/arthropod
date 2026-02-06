use bevy_ecs::prelude::*;
use rayon::prelude::*;
use render_engine::{backend::RectInstance, NodeId, Scene};
use std::collections::HashSet;

use crate::adaptive::AdaptiveThresholds;
use crate::components::{Renderable, SceneNodeRef};

/// Resource for collecting render commands
///
/// Systems write RenderCommands into this resource, which is then
/// extracted and passed to the GPU backend for rendering.
#[derive(Resource, Default, Debug)]
pub struct RenderCommands(pub Vec<RectInstance>);

/// Collect all visible renderables into GPU instances
///
/// This system queries all entities marked as Renderable, fetches their
/// corresponding scene nodes, and generates RectInstances for the GPU backend.
/// Invisible nodes and nodes with zero opacity are filtered out.
///
/// Uses adaptive thresholds to decide between parallel and sequential collection.
/// The threshold is dynamically adjusted based on observed frame metrics to
/// minimize overhead while maximizing throughput. See `AdaptiveThresholds` for details.
///
/// `par_iter().filter_map().collect()` preserves input ordering, maintaining
/// the Painter's Algorithm z-order.
pub fn collect_renderables_system(
    query: Query<&SceneNodeRef, With<Renderable>>,
    scene: Res<Scene>,
    mut commands: ResMut<RenderCommands>,
    thresholds: Res<AdaptiveThresholds>,
) {
    commands.0.clear();

    // 1. Collect renderable NodeIds from ECS to filter the scene traversal
    let renderable_nodes: HashSet<NodeId> = query.iter().map(|r| r.0).collect();

    // 2. Collect visual nodes in painter's order, filtered by renderable set
    let visual_nodes: Vec<_> = scene
        .iter_visuals()
        .filter(|(id, _)| renderable_nodes.contains(id))
        .collect();

    // 3. Generate instances — use adaptive threshold to choose execution path
    let threshold = thresholds.as_ref().current().render_parallel_threshold;
    if visual_nodes.len() >= threshold {
        commands.0 = visual_nodes
            .par_iter()
            .filter_map(|(_, node)| render_engine::backend::wgpu::create_rect_instance(node))
            .collect();
    } else {
        for (_, node) in &visual_nodes {
            if let Some(instance) = render_engine::backend::wgpu::create_rect_instance(node) {
                commands.0.push(instance);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plat_core::Rect;
    use render_engine::{Color, NodeContent, SceneNode, Transform2D};

    /// Test that parallel and sequential paths produce identical output
    #[test]
    fn test_parallel_output_matches_sequential() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Create enough nodes to exceed the threshold
        let mut node_ids = Vec::new();
        for i in 0..300 {
            let node = SceneNode {
                content: NodeContent::Rect {
                    color: Color::rgba(i as f32 / 300.0, 0.0, 0.0, 1.0),
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
        }

        let renderable_nodes: HashSet<NodeId> = node_ids.iter().copied().collect();
        let visual_nodes: Vec<_> = scene
            .iter_visuals()
            .filter(|(id, _)| renderable_nodes.contains(id))
            .collect();

        // Sequential
        let sequential: Vec<_> = visual_nodes
            .iter()
            .filter_map(|(_, node)| render_engine::backend::wgpu::create_rect_instance(node))
            .collect();

        // Parallel
        let parallel: Vec<_> = visual_nodes
            .par_iter()
            .filter_map(|(_, node)| render_engine::backend::wgpu::create_rect_instance(node))
            .collect();

        assert_eq!(sequential.len(), parallel.len());
        for (seq, par) in sequential.iter().zip(parallel.iter()) {
            assert_eq!(seq.pos, par.pos, "Position mismatch");
            assert_eq!(seq.color, par.color, "Color mismatch");
        }
    }

    /// Test that below threshold, the system still works correctly
    #[test]
    fn test_below_threshold_works() {
        let mut world = World::new();
        let mut scene = Scene::new();
        let root = scene.root();

        let node = SceneNode {
            content: NodeContent::Rect { color: Color::RED },
            transform: Transform2D::identity(),
            bounds: Rect::new(0.0, 0.0, 100.0, 100.0),
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        let id = scene.add_node(root, node);

        world.insert_resource(scene);
        world.insert_resource(RenderCommands::default());
        world.insert_resource(AdaptiveThresholds::new());
        world.spawn((SceneNodeRef(id), Renderable));

        let mut schedule = Schedule::default();
        schedule.add_systems(collect_renderables_system);
        schedule.run(&mut world);

        let commands = world.resource::<RenderCommands>();
        assert_eq!(commands.0.len(), 1);
        assert_eq!(commands.0[0].color, [1.0, 0.0, 0.0, 1.0]);
    }
}
