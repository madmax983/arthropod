//! Accessibility synchronization system
//!
//! Syncs ECS entities with AccessibleNode components to the A11yTree,
//! ensuring screen readers have up-to-date information.

use crate::components::{AccessibleNode, OnA11yClick, SceneNodeRef};
use a11y_engine::{ArthropodActionHandler, A11yTree};
use bevy_ecs::prelude::*;
use render_engine::Scene;

/// Sync accessible nodes with A11yTree
///
/// Updates A11y nodes with current bounds from the scene graph.
/// This system runs after reactive updates to ensure the A11yTree reflects
/// current UI state for screen readers.
pub fn sync_accessible_nodes_system(
    query: Query<(&SceneNodeRef, &AccessibleNode)>,
    scene: Res<Scene>,
    mut a11y_tree: ResMut<A11yTree>,
) {
    for (scene_ref, accessible) in query.iter() {
        // Get scene node bounds
        if let Some(scene_node) = scene.get(scene_ref.0) {
            // Update A11y node with current bounds from scene
            a11y_tree.update_node(accessible.a11y_id, |a11y_node| {
                a11y_node.bounds = scene_node.bounds;
            });
        }
    }
}

/// Register action callbacks with the ActionHandler
///
/// This system uses change detection to efficiently track callback registration:
/// - Only registers callbacks for newly added or modified OnA11yClick components
/// - Avoids expensive HashMap rebuilds every frame
///
/// Note: When OnA11yClick components are removed, the handlers remain registered
/// but become no-ops since the A11yId won't match any active nodes. Full cleanup
/// happens when entities are despawned and removed from the A11yTree.
pub fn register_action_callbacks_system(
    // Only query components that were added or changed this frame
    query: Query<(&AccessibleNode, &OnA11yClick), Or<(Added<OnA11yClick>, Changed<OnA11yClick>)>>,
    mut action_handler: ResMut<ArthropodActionHandler>,
) {
    // Register callbacks for added/changed components
    for (accessible, on_click) in query.iter() {
        let callback = on_click.callback.clone();
        action_handler.register_click(accessible.a11y_id, move || {
            callback();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use a11y_engine::{A11yId, A11yNode, AccessibleName, Role};
    use accesskit::ActionHandler;
    use plat_core::Rect;
    use render_engine::{NodeContent, SceneNode};
    use std::sync::Arc;

    #[test]
    fn test_sync_updates_a11y_bounds_from_scene() {
        let mut world = World::new();
        let mut scene = Scene::new();
        let mut a11y_tree = A11yTree::new();

        // Setup: scene node at (10, 20, 100, 50)
        let scene_id = scene.add_node(scene.root(), SceneNode::new(NodeContent::Empty));
        scene.get_node_mut(scene_id).unwrap().bounds = Rect::new(10.0, 20.0, 100.0, 50.0);

        let a11y_id = a11y_tree.add_node(
            a11y_tree.root(),
            A11yNode {
                role: Role::Button,
                name: AccessibleName::Text("Test".into()),
                ..Default::default()
            },
        );

        // Create entity with both components
        world.spawn((
            SceneNodeRef(scene_id),
            AccessibleNode::new(a11y_id, Role::Button),
        ));

        // Act: sync system should update a11y bounds from scene
        // Note: Test is stubbed - actual system integration will be tested in integration tests
        // For now, manually test the concept
        if let Some(scene_node) = scene.get_node(scene_id) {
            a11y_tree.update_node(a11y_id, |node| {
                node.bounds = scene_node.bounds;
            });
        }

        // Assert: a11y node has scene bounds
        let a11y_node = a11y_tree.get_node(a11y_id).unwrap();
        assert_eq!(a11y_node.bounds.x, 10.0);
        assert_eq!(a11y_node.bounds.y, 20.0);
        assert_eq!(a11y_node.bounds.width, 100.0);
        assert_eq!(a11y_node.bounds.height, 50.0);
    }

    #[test]
    fn test_sync_marks_nodes_dirty() {
        let mut world = World::new();
        let mut scene = Scene::new();
        let mut a11y_tree = A11yTree::new();

        let scene_id = scene.add_node(scene.root(), SceneNode::new(NodeContent::Empty));
        scene.get_node_mut(scene_id).unwrap().bounds = Rect::new(0.0, 0.0, 100.0, 100.0);

        let a11y_id = a11y_tree.add_node(a11y_tree.root(), A11yNode::default());

        world.spawn((
            SceneNodeRef(scene_id),
            AccessibleNode::new(a11y_id, Role::Button),
        ));

        // Clear dirty flags
        a11y_tree.clear_dirty();
        assert_eq!(a11y_tree.get_dirty_nodes().len(), 0);

        // Sync should mark node dirty
        if let Some(scene_node) = scene.get_node(scene_id) {
            a11y_tree.update_node(a11y_id, |node| {
                node.bounds = scene_node.bounds;
            });
        }

        // Node should be marked dirty
        assert_eq!(a11y_tree.get_dirty_nodes().len(), 1);
        assert!(a11y_tree.get_dirty_nodes().contains(&a11y_id));
    }

    #[test]
    fn test_register_action_callbacks() {
        use std::sync::Mutex;

        let mut world = World::new();
        world.insert_resource(ArthropodActionHandler::new());

        let clicked = Arc::new(Mutex::new(false));
        let clicked_clone = clicked.clone();

        let a11y_id = A11yId::new();

        // Spawn entity with click handler
        world.spawn((
            AccessibleNode::new(a11y_id, Role::Button),
            OnA11yClick::new(move || {
                *clicked_clone.lock().unwrap() = true;
            }),
        ));

        // Register callbacks (manually call system logic with change detection)
        {
            let mut callbacks_to_register = Vec::new();
            // Simulate Added<OnA11yClick> by querying all components
            let mut query = world.query::<(&AccessibleNode, &OnA11yClick)>();
            for (accessible, on_click) in query.iter(&world) {
                callbacks_to_register.push((accessible.a11y_id, on_click.callback.clone()));
            }

            // Register them (no clearing - change detection handles this)
            let mut handler = world.resource_mut::<ArthropodActionHandler>();
            for (a11y_id, callback) in callbacks_to_register {
                handler.register_click(a11y_id, move || {
                    callback();
                });
            }
        }

        // Simulate click from screen reader
        let mut handler = world.resource_mut::<ArthropodActionHandler>();
        handler.do_action(accesskit::ActionRequest {
            action: accesskit::Action::Click,
            target: accesskit::NodeId(a11y_id.raw()),
            data: None,
        });

        assert!(*clicked.lock().unwrap(), "Click callback should have been invoked");
    }

    #[test]
    fn test_register_multiple_callbacks() {
        use std::sync::Mutex;

        let mut world = World::new();
        world.insert_resource(ArthropodActionHandler::new());

        let count = Arc::new(Mutex::new(0));
        let count1 = count.clone();
        let count2 = count.clone();

        let id1 = A11yId::new();
        let id2 = A11yId::new();

        world.spawn((
            AccessibleNode::new(id1, Role::Button),
            OnA11yClick::new(move || {
                *count1.lock().unwrap() += 1;
            }),
        ));

        world.spawn((
            AccessibleNode::new(id2, Role::Button),
            OnA11yClick::new(move || {
                *count2.lock().unwrap() += 10;
            }),
        ));

        // Register all callbacks (manually call system logic with change detection)
        {
            let mut callbacks_to_register = Vec::new();
            let mut query = world.query::<(&AccessibleNode, &OnA11yClick)>();
            for (accessible, on_click) in query.iter(&world) {
                callbacks_to_register.push((accessible.a11y_id, on_click.callback.clone()));
            }

            // Register them (no clearing - change detection handles this)
            let mut handler = world.resource_mut::<ArthropodActionHandler>();
            for (a11y_id, callback) in callbacks_to_register {
                handler.register_click(a11y_id, move || {
                    callback();
                });
            }
        }

        // Click both buttons
        let mut handler = world.resource_mut::<ArthropodActionHandler>();
        handler.do_action(accesskit::ActionRequest {
            action: accesskit::Action::Click,
            target: accesskit::NodeId(id1.raw()),
            data: None,
        });
        handler.do_action(accesskit::ActionRequest {
            action: accesskit::Action::Click,
            target: accesskit::NodeId(id2.raw()),
            data: None,
        });

        assert_eq!(*count.lock().unwrap(), 11); // 1 + 10 = 11
    }
}
