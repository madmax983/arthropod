use bevy_ecs::prelude::*;
use render_engine::{backend::RectInstance, NodeId, Scene};
use std::collections::HashSet;

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
/// Scene is now accessed as a safe Resource - no more unsafe pointer code!
pub fn collect_renderables_system(
    query: Query<&SceneNodeRef, With<Renderable>>,
    scene: Res<Scene>,
    mut commands: ResMut<RenderCommands>,
) {
    commands.0.clear();

    // 1. Collect renderable NodeIds from ECS to filter the scene traversal
    let renderable_nodes: HashSet<NodeId> = query.iter().map(|r| r.0).collect();

    // 2. Iterate scene in strict visual order (Painter's Algorithm)
    // Res<Scene> implements Deref, so we can call Scene methods directly
    for (id, node) in scene.iter_visuals() {
        // Skip nodes that are not managed by ECS or marked as Renderable
        if !renderable_nodes.contains(&id) {
            continue;
        }

        // Use helper for rect instances
        // Note: This helper handles visibility and opacity checks internally
        if let Some(instance) = render_engine::backend::wgpu::create_rect_instance(node) {
            commands.0.push(instance);
        }
    }
}
