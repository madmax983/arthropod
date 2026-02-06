use bevy_ecs::prelude::*;
use render_engine::Scene;
use arthropod_ecs::SceneNodeRef;
use plat_core::Rect;
use crate::components::{SpatialNode, Viewport};

/// Updates the scene node bounds based on the spatial node position and camera.
/// This system effectively "projects" the infinite 2D space onto the screen.
pub fn spatial_transform_system(
    viewport: Res<Viewport>,
    mut scene: ResMut<Scene>,
    query: Query<(&SpatialNode, &SceneNodeRef)>,
) {
    let camera = &viewport.camera;

    for (spatial_node, scene_node_ref) in query.iter() {
        if let Some(node) = scene.get_node_mut(scene_node_ref.0) {
            let screen_pos = camera.world_to_screen(spatial_node.position);
            let screen_size = spatial_node.size * camera.zoom;

            // Update bounds (x, y, width, height)
            // We center the node at the position
            let x = screen_pos.x - screen_size.x / 2.0;
            let y = screen_pos.y - screen_size.y / 2.0;

            node.bounds = Rect::new(
                x,
                y,
                screen_size.x,
                screen_size.y
            );

            // Mark as dirty to ensure redraw
            scene.mark_dirty(scene_node_ref.0);
        }
    }
}
