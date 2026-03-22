use super::core::App;
use arthropod_ecs::Renderable;
use render_engine::{NodeId, Scene};

/// Integrate widget scene into app ECS (spawn entities).
///
/// Iterates over the subtree starting at `widget_root` and spawns an ECS entity
/// for each node, attaching the [`Renderable`] component.
///
/// This assumes the nodes are already present in the App's [`Scene`] resource.
pub fn integrate_widget_scene(app: &mut App, widget_root: NodeId) {
    // 1. Collect all NodeIds in the subtree to avoid borrowing conflicts
    let mut nodes_to_spawn = Vec::new();

    {
        let scene = app.world().resource::<Scene>();
        let mut stack = vec![widget_root];
        let mut visited = std::collections::HashSet::new();

        while let Some(id) = stack.pop() {
            if !visited.insert(id) {
                continue;
            }

            if let Some(node) = scene.get_node(id) {
                nodes_to_spawn.push(id);
                for &child_id in &node.children {
                    stack.push(child_id);
                }
            }
        }
    }

    // 2. Spawn ECS entities for each node
    for node_id in nodes_to_spawn {
        // Check if entity already exists to avoid duplicates
        // get_entity_mut implementation uses a query, so we can check if it returns anything
        if app.get_entity_mut(node_id).is_none() {
            app.spawn(node_id).insert(Renderable);
        }
    }
}
