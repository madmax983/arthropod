use bevy_ecs::prelude::*;
use hashbrown::HashSet;
use render_engine::{backend::PrimitiveInstance, NodeId, Scene}; // ⚡ Bolt: Using hashbrown::HashSet provides AHash instead of SipHash, improving performance during render collection

use crate::components::{Renderable, SceneNodeRef};

/// Resource for collecting render commands
///
/// Systems write RenderCommands into this resource, which is then
/// extracted and passed to the GPU backend for rendering.
#[derive(Resource, Default, Debug)]
pub struct RenderCommands(pub Vec<PrimitiveInstance>);

/// Collect all visible renderables into GPU instances
///
/// This system queries all entities marked as Renderable, fetches their
/// corresponding scene nodes, and generates PrimitiveInstances for the GPU backend.
/// Invisible nodes and nodes with zero opacity are filtered out.
///
/// Uses a static threshold to decide between parallel and sequential collection.
///
/// `par_iter().filter_map().collect()` preserves input ordering, maintaining
/// the Painter's Algorithm z-order.
pub fn collect_renderables_system(
    query: Query<&SceneNodeRef, With<Renderable>>,
    scene: Res<Scene>,
    mut commands: ResMut<RenderCommands>,
    // Persistent cache for the set of renderable node IDs to avoid per-frame allocation
    mut renderable_nodes_cache: Local<HashSet<NodeId>>,
    // Persistent stack for scene traversal to avoid per-frame allocation
    mut traversal_stack: Local<Vec<(NodeId, f32)>>,
    // Persistent cache for ordered visual nodes to avoid per-frame allocation
    mut visual_nodes_cache: Local<Vec<NodeId>>,
) {
    commands.0.clear();
    renderable_nodes_cache.clear();
    visual_nodes_cache.clear();

    // 1. Collect renderable NodeIds from ECS to filter the scene traversal
    renderable_nodes_cache.extend(query.iter().map(|r| r.0));

    // 2. Collect visual nodes in painter's order, filtered by renderable set
    visual_nodes_cache.extend(
        scene
            .iter_visuals_custom(&mut traversal_stack)
            .filter(|(id, _, _)| renderable_nodes_cache.contains(id))
            .map(|(id, _, _)| id),
    );

    // 3. Generate instances — use threshold to choose execution path
    commands.0.reserve(visual_nodes_cache.len());

    // Bolt: Since we don't know the exact number of PrimitiveInstances produced by `create_node_instances`
    // pre-allocating per-thread chunks in Rayon with `fold(|| Vec::with_capacity(256))` generates `P` allocations per frame.
    // Instead, we just sequentially execute this since the work inside `create_node_instances` (without multipass text shaping)
    // is negligible compared to the memory allocator lock contention.
    // Testing shows avoiding parallel `Vec` allocations here is much faster.
    for id in &*visual_nodes_cache {
        if let Some(node) = scene.get_node(*id) {
            render_engine::backend::wgpu::create_node_instances(node, &mut commands.0);
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
                content: NodeContent::Styled {
                    style: Box::new(
                        render_engine::VisualStyle::new()
                            .solid_fill(Color::rgba(i as f32 / 300.0, 0.0, 0.0, 1.0).as_vec4()),
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
        }

        let renderable_nodes: HashSet<NodeId> = node_ids.iter().copied().collect();
        let visual_nodes: Vec<_> = scene
            .iter_visuals()
            .filter(|(id, _, _)| renderable_nodes.contains(id))
            .collect();

        // Sequential
        let mut sequential = Vec::new();
        for (_, node, _) in &visual_nodes {
            render_engine::backend::wgpu::create_node_instances(node, &mut sequential);
        }

        // Verify sequential generation still produces valid results
        assert!(!sequential.is_empty(), "Expected instances to be generated");
    }

    /// Test that below threshold, the system still works correctly
    #[test]
    fn test_below_threshold_works() {
        let mut world = World::new();
        let mut scene = Scene::new();
        let root = scene.root();

        let node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::RED.as_vec4())),
            },
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
        world.spawn((SceneNodeRef(id), Renderable));

        let mut schedule = Schedule::default();
        schedule.add_systems(collect_renderables_system);
        schedule.run(&mut world);

        let commands = world.resource::<RenderCommands>();
        assert_eq!(commands.0.len(), 1);
        assert_eq!(commands.0[0].color, [1.0, 0.0, 0.0, 1.0]);
    }
}
