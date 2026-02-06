use crate::components::{SpatialNode, Stats, Viewport};
use arthropod_ecs::SceneNodeRef;
use bevy_ecs::prelude::*;
use plat_core::Rect;
use render_engine::Scene;

/// Updates the stats resource with total and visible node counts.
pub fn stats_system(mut stats: ResMut<Stats>, query: Query<&SceneNodeRef>, scene: Res<Scene>) {
    let mut visible = 0;
    let mut total = 0;

    for scene_node_ref in query.iter() {
        total += 1;
        if let Some(node) = scene.get_node(scene_node_ref.0) {
            if node.visible {
                visible += 1;
            }
        }
    }

    stats.total_nodes = total;
    stats.visible_nodes = visible;
}

/// Updates the scene node bounds based on the spatial node position and camera.
/// This system effectively "projects" the infinite 2D space onto the screen.
pub fn spatial_transform_system(
    viewport: Res<Viewport>,
    mut scene: ResMut<Scene>,
    query: Query<(&SpatialNode, &SceneNodeRef)>,
) {
    let camera = &viewport.camera;
    let viewport_w = camera.viewport_size.x;
    let viewport_h = camera.viewport_size.y;

    for (spatial_node, scene_node_ref) in query.iter() {
        if let Some(node) = scene.get_node_mut(scene_node_ref.0) {
            let screen_pos = camera.world_to_screen(spatial_node.position);
            let screen_size = spatial_node.size * camera.zoom;

            // Update bounds (x, y, width, height)
            // We center the node at the position
            let x = screen_pos.x - screen_size.x / 2.0;
            let y = screen_pos.y - screen_size.y / 2.0;
            let w = screen_size.x;
            let h = screen_size.y;

            node.bounds = Rect::new(x, y, w, h);

            // Spatial Culling (Frustum Check)
            // Check if the node's bounds intersect the viewport (0, 0, width, height)
            let is_visible = x < viewport_w && x + w > 0.0 && y < viewport_h && y + h > 0.0;
            let was_visible = node.visible;
            node.visible = is_visible;

            // Mark as dirty if visibility changed or if it is visible (to update position)
            // If it remains invisible, we don't need to trigger a redraw for it
            if was_visible || is_visible {
                scene.mark_dirty(scene_node_ref.0);
            }
        }
    }
}
