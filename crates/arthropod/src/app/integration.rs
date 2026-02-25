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
    //    (We can't iterate the Scene while mutating the App/World)
    let nodes_to_spawn = {
        let scene = app.world().resource::<Scene>();
        let mut nodes = Vec::new();
        let mut stack = vec![widget_root];

        while let Some(id) = stack.pop() {
            if let Some(node) = scene.get_node(id) {
                nodes.push(id);
                // Push children to stack
                // Note: Order doesn't matter for spawning entities
                stack.extend(node.children.iter().copied());
            }
        }
        nodes
    };

    // 2. Spawn ECS entities for each node
    for node_id in nodes_to_spawn {
        app.spawn(node_id).insert(Renderable);
    }
}
