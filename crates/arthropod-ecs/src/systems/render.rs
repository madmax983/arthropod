use bevy_ecs::prelude::*;
use render_engine::{backend::RectInstance, NodeContent, Scene};

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

    // Res<Scene> implements Deref, so we can call Scene methods directly
    for node_ref in query.iter() {
        if let Some(node) = scene.get(node_ref.0) {
            // Skip invisible nodes and fully transparent nodes
            if !node.visible || node.opacity <= 0.0 {
                continue;
            }

            // Generate render instances based on node content type
            match &node.content {
                NodeContent::Rect { color } | NodeContent::RoundedRect { color, .. } => {
                    commands.0.push(RectInstance {
                        pos: [node.bounds.x, node.bounds.y],
                        size: [node.bounds.width, node.bounds.height],
                        color: [
                            color.r(),
                            color.g(),
                            color.b(),
                            color.a() * node.opacity, // Apply opacity
                        ],
                    });
                }
                NodeContent::Empty => {
                    // Empty nodes have no visual representation
                }
            }
        }
    }
}
