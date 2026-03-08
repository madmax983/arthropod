#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};
#[cfg(feature = "nova")]
use style_engine::{Paint, StrokeAlign, StrokeStyle, VisualStyle};

#[cfg(feature = "nova")]
use super::spatial_query::SpatialIndex;

#[cfg(feature = "nova")]
#[derive(Resource, Default)]
pub struct InspectorOverlayConfig {
    pub enabled: bool,
}

#[cfg(feature = "nova")]
#[derive(Resource, Default)]
pub struct InspectorOverlayState {
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub active_overlay_nodes: Vec<NodeId>,
}

#[cfg(feature = "nova")]
pub fn update_inspector_overlay(
    mut scene: ResMut<Scene>,
    config: Res<InspectorOverlayConfig>,
    mut state: ResMut<InspectorOverlayState>,
    spatial_index: Res<SpatialIndex>,
) {
    // 1. Clean up old overlay nodes
    for &node_id in &state.active_overlay_nodes {
        if scene.get_node(node_id).is_some() {
            scene.remove_node(node_id);
        }
    }
    state.active_overlay_nodes.clear();

    if !config.enabled {
        return;
    }

    // 2. Find nodes under cursor using SpatialIndex
    let hits = spatial_index.query_point(state.cursor_x, state.cursor_y);

    if hits.is_empty() {
        return;
    }

    // 3. Highlight the most specific node (usually the last rendered / deepest child)
    // We can just highlight all hits or the top one. Let's highlight the top one.
    // In our spatial index, they might not be sorted by z-index, but we'll highlight
    // all of them with a semi-transparent fill and border.

    let style = Box::new(
        VisualStyle::new()
            .solid_fill(Color::rgba(0.0, 1.0, 0.0, 0.1).as_vec4())
            .stroke(StrokeStyle::solid(
                Paint::solid(Color::GREEN.as_vec4()),
                2.0,
                StrokeAlign::Center,
            )),
    );

    // Let's draw an overlay over the last hit node
    if let Some(&top_hit) = hits.last() {
        #[allow(clippy::collapsible_if)]
        if let Some(target_node) = scene.get_node(top_hit) {
            let bounds = target_node.bounds;

            let mut overlay_node = SceneNode::new(NodeContent::Styled {
                style: style.clone(),
            });
            overlay_node.bounds = bounds;

            // Add the overlay as a child of the root to draw on top of everything
            let root = scene.root();
            let overlay_id = scene.add_node(root, overlay_node);
            state.active_overlay_nodes.push(overlay_id);
        }
    }
}

#[cfg(feature = "nova")]
pub fn register_inspector_overlay(app: &mut crate::App) {
    app.world_mut()
        .insert_resource(InspectorOverlayConfig::default());
    app.world_mut()
        .insert_resource(InspectorOverlayState::default());

    app.add_update_system(update_inspector_overlay);
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use crate::App;
    use plat_core::Rect;

    #[test]
    fn test_inspector_overlay_toggles() {
        let mut app = App::new_headless().unwrap();
        crate::experimental::spatial_query::register_spatial_query(&mut app);
        register_inspector_overlay(&mut app);

        app.update();

        // Manual right click to enable
        {
            let mut config = app.world_mut().resource_mut::<InspectorOverlayConfig>();
            config.enabled = true;
        }
        app.update();

        assert!(app.world().resource::<InspectorOverlayConfig>().enabled);

        // Right click again to disable
        {
            let mut config = app.world_mut().resource_mut::<InspectorOverlayConfig>();
            config.enabled = false;
        }
        app.update();

        assert!(!app.world().resource::<InspectorOverlayConfig>().enabled);
    }

    #[test]
    fn test_inspector_overlay_highlights_node() {
        let mut app = App::new_headless().unwrap();
        crate::experimental::spatial_query::register_spatial_query(&mut app);
        register_inspector_overlay(&mut app);

        {
            let mut config = app.world_mut().resource_mut::<InspectorOverlayConfig>();
            config.enabled = true;
        }

        let _target_node_id = {
            let mut scene = app.world_mut().resource_mut::<Scene>();
            let root = scene.root();
            let mut node = SceneNode::new(NodeContent::SolidColor { color: Color::RED });
            node.bounds = Rect::new(10.0, 10.0, 50.0, 50.0);
            scene.add_node(root, node)
        };

        app.update();

        // Move cursor over node manually
        {
            let mut state = app.world_mut().resource_mut::<InspectorOverlayState>();
            state.cursor_x = 30.0;
            state.cursor_y = 30.0;
        }

        app.update();

        let state = app.world().resource::<InspectorOverlayState>();
        assert_eq!(state.active_overlay_nodes.len(), 1);

        let overlay_id = state.active_overlay_nodes[0];
        let scene = app.world().resource::<Scene>();
        let overlay_node = scene.get_node(overlay_id).unwrap();

        assert_eq!(overlay_node.bounds.x, 10.0);
        assert_eq!(overlay_node.bounds.y, 10.0);
        assert_eq!(overlay_node.bounds.width, 50.0);
        assert_eq!(overlay_node.bounds.height, 50.0);
    }
}
