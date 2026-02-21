use bevy_ecs::prelude::*;
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};
use std::collections::{HashMap, HashSet};
use style_engine::{Paint, StrokeAlign, StrokeStyle, VisualStyle};

#[derive(Resource, Default)]
pub struct XRayConfig {
    pub enabled: bool,
}

#[derive(Resource, Default)]
pub struct XRayState {
    // Maps target node ID -> debug node ID
    pub active_nodes: HashMap<NodeId, NodeId>,
}

pub fn update_xray(
    mut scene: ResMut<Scene>,
    config: Res<XRayConfig>,
    mut state: ResMut<XRayState>,
) {
    if !config.enabled {
        if !state.active_nodes.is_empty() {
            for debug_node in state.active_nodes.values() {
                // Check if node exists before removing (it might have been removed with parent)
                if scene.get_node(*debug_node).is_some() {
                    scene.remove_node(*debug_node);
                }
            }
            state.active_nodes.clear();
        }
        return;
    }

    // 1. Identify valid targets (all non-debug nodes)
    // We collect them first to avoid borrow issues while mutating scene
    // We also need to filter out our own debug nodes from being targets
    let debug_ids: HashSet<NodeId> = state.active_nodes.values().copied().collect();

    let mut targets = Vec::new();
    for (id, node) in scene.nodes() {
        if debug_ids.contains(&id) {
            continue;
        }
        targets.push((id, node.bounds));
    }

    // 2. Reconcile state
    // Remove debug nodes for targets that no longer exist
    state.active_nodes.retain(|target, _| {
        if scene.get_node(*target).is_none() {
            // Target gone, debug node is gone too (automatically)
            false
        } else {
            true
        }
    });

    // Also check if debug nodes themselves were deleted unexpectedly
    state
        .active_nodes
        .retain(|_, debug| scene.get_node(*debug).is_some());

    let style = Box::new(VisualStyle::new().stroke(StrokeStyle::solid(
        Paint::solid(Color::RED.as_vec4()),
        1.0,
        StrokeAlign::Inside,
    )));

    for (target_id, bounds) in targets {
        if let Some(&debug_id) = state.active_nodes.get(&target_id) {
            // Update existing
            if let Some(debug_node) = scene.get_node_mut(debug_id) {
                // Sync bounds
                debug_node.bounds = bounds;
            }

            // Ensure it's the last child (drawn on top)
            // We need to release mutable borrow from get_node_mut before getting parent
            let is_last = if let Some(parent) = scene.get_node(target_id) {
                parent.children.last() == Some(&debug_id)
            } else {
                false
            };

            if !is_last {
                scene.reparent_node(debug_id, target_id, target_id);
            }
        } else {
            // Create new debug node
            let mut node = SceneNode::new(NodeContent::Styled {
                style: style.clone(),
            });
            node.bounds = bounds;

            // Add as child of target
            let debug_id = scene.add_node(target_id, node);
            state.active_nodes.insert(target_id, debug_id);
        }
    }
}

pub fn register_xray(app: &mut crate::App) {
    app.world_mut()
        .insert_resource(XRayConfig { enabled: true });
    app.world_mut().insert_resource(XRayState::default());
    app.add_update_system(update_xray);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::App;

    #[test]
    fn test_xray_enables_debug_nodes() {
        let mut app = App::new_headless().unwrap();
        register_xray(&mut app);

        // Add a node
        let root = app.world().resource::<Scene>().root();

        // XRay runs on update
        app.update();

        let state = app.world().resource::<XRayState>();
        assert!(state.active_nodes.contains_key(&root));

        let debug_id = state.active_nodes[&root];
        let scene = app.world().resource::<Scene>();
        assert!(scene.get_node(debug_id).is_some());

        // Check style
        let node = scene.get_node(debug_id).unwrap();
        if let NodeContent::Styled { style } = &node.content {
            assert!(style.stroke.is_some());
        } else {
            panic!("Expected Styled content");
        }
    }

    #[test]
    fn test_xray_disables_debug_nodes() {
        let mut app = App::new_headless().unwrap();
        register_xray(&mut app);

        app.update(); // Enable

        {
            let mut config = app.world_mut().resource_mut::<XRayConfig>();
            config.enabled = false;
        }

        app.update(); // Disable

        let state = app.world().resource::<XRayState>();
        assert!(state.active_nodes.is_empty());

        let scene = app.world().resource::<Scene>();
        // Root should still exist
        let root = scene.root();
        assert!(scene.get_node(root).is_some());
        // Root should have no children (debug node removed)
        assert!(scene.get_node(root).unwrap().children.is_empty());
    }
}
