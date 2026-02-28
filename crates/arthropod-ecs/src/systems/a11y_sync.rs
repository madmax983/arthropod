//! Accessibility synchronization system
//!
//! Syncs ECS entities with AccessibleNode components to the A11yTree,
//! ensuring screen readers have up-to-date information.
//!
//! Uses gather-apply pattern for parallelization:
//! - Gather: reads Scene bounds (`Res<Scene>`) — can overlap with render collection
//! - Apply: writes A11yTree (`ResMut<A11yTree>`) — runs after gather

use crate::components::{AccessibleNode, OnA11yClick, SceneNodeRef};
use a11y_engine::{A11yId, A11yTree, ArthropodActionHandler};
use bevy_ecs::prelude::*;
use plat_core::Rect;
use render_engine::Scene;

/// Buffer for a11y bounds gathered from the Scene.
///
/// Populated by `gather_a11y_bounds_system`, consumed by `apply_a11y_bounds_system`.
#[derive(Resource, Default)]
pub struct A11yBoundsBuffer {
    pub entries: Vec<(A11yId, Rect)>,
}

/// Gather phase: reads Scene bounds for all accessible nodes.
///
/// Uses `Res<Scene>` (shared borrow) so it can run in parallel with
/// `collect_renderables_system` which also uses `Res<Scene>`.
pub fn gather_a11y_bounds_system(
    query: Query<(&SceneNodeRef, &AccessibleNode)>,
    scene: Res<Scene>,
    mut buffer: ResMut<A11yBoundsBuffer>,
) {
    buffer.entries.clear();

    for (scene_ref, accessible) in query.iter() {
        if let Some(scene_node) = scene.get(scene_ref.0) {
            buffer.entries.push((accessible.a11y_id, scene_node.bounds));
        }
    }
}

/// Apply phase: writes gathered bounds to the A11yTree.
///
/// Uses `ResMut<A11yTree>` — runs after the gather phase.
pub fn apply_a11y_bounds_system(buffer: Res<A11yBoundsBuffer>, mut a11y_tree: ResMut<A11yTree>) {
    for &(a11y_id, bounds) in &buffer.entries {
        a11y_tree.update_node(a11y_id, |a11y_node| {
            a11y_node.bounds = bounds;
        });
    }
}

/// Monolithic sync (kept for backwards compat, replaced by gather+apply in schedule)
///
/// Updates A11y nodes with current bounds from the scene graph.
/// This system runs after reactive updates to ensure the A11yTree reflects
/// current UI state for screen readers.
#[deprecated(
    since = "0.2.0",
    note = "Use `gather_a11y_bounds_system` + `apply_a11y_bounds_system` instead"
)]
pub fn sync_accessible_nodes_system(
    query: Query<(&SceneNodeRef, &AccessibleNode)>,
    scene: Res<Scene>,
    mut a11y_tree: ResMut<A11yTree>,
) {
    for (scene_ref, accessible) in query.iter() {
        if let Some(scene_node) = scene.get(scene_ref.0) {
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
#[allow(clippy::type_complexity)]
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
    use a11y_engine::{A11yNode, A11yTree, ArthropodActionHandler, Role as A11yRole};
    use accesskit::{ActionHandler, ActionRequest, NodeId as AccessKitNodeId, TreeId};

    #[test]
    fn test_sync_updates_a11y_bounds_from_scene() {
        let mut world = World::new();

        // Create scene with a node
        let mut scene = render_engine::Scene::new();
        let root = scene.root();
        let scene_node = render_engine::SceneNode {
            content: render_engine::NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(render_engine::Color::RED.as_vec4()),
                ),
            },
            transform: render_engine::Transform2D::identity(),
            bounds: Rect::new(10.0, 20.0, 200.0, 100.0),
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        let node_id = scene.add_node(root, scene_node);

        // Create a11y tree with a node
        let mut a11y_tree = A11yTree::new();
        let a11y_root = a11y_tree.root();
        let button_node = A11yNode {
            role: A11yRole::Button,
            ..Default::default()
        };
        let a11y_id = a11y_tree.add_node(a11y_root, button_node);

        world.insert_resource(scene);
        world.insert_resource(a11y_tree);
        world.insert_resource(A11yBoundsBuffer::default());

        // Create ECS entity linking scene to a11y
        let accessible = AccessibleNode::new(a11y_id, A11yRole::Button);
        world.spawn((SceneNodeRef(node_id), accessible));

        // Run gather + apply
        let mut schedule = Schedule::default();
        schedule.add_systems((
            gather_a11y_bounds_system,
            apply_a11y_bounds_system.after(gather_a11y_bounds_system),
        ));
        schedule.run(&mut world);

        // Verify bounds updated
        let tree = world.resource::<A11yTree>();
        let a11y = tree.get_node(a11y_id).expect("Node should exist");
        assert_eq!(a11y.bounds.x, 10.0);
        assert_eq!(a11y.bounds.y, 20.0);
        assert_eq!(a11y.bounds.width, 200.0);
        assert_eq!(a11y.bounds.height, 100.0);
    }

    #[test]
    fn test_sync_marks_nodes_dirty() {
        let mut world = World::new();

        let mut scene = render_engine::Scene::new();
        let root = scene.root();
        let scene_node = render_engine::SceneNode {
            content: render_engine::NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(render_engine::Color::RED.as_vec4()),
                ),
            },
            transform: render_engine::Transform2D::identity(),
            bounds: Rect::new(5.0, 10.0, 50.0, 30.0),
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        let node_id = scene.add_node(root, scene_node);

        let mut a11y_tree = A11yTree::new();
        let a11y_root = a11y_tree.root();
        let container_node = A11yNode {
            role: A11yRole::Group,
            ..Default::default()
        };
        let a11y_id = a11y_tree.add_node(a11y_root, container_node);

        world.insert_resource(scene);
        world.insert_resource(a11y_tree);
        world.insert_resource(A11yBoundsBuffer::default());

        let accessible = AccessibleNode::new(a11y_id, A11yRole::Group);
        world.spawn((SceneNodeRef(node_id), accessible));

        // Run gather + apply
        let mut schedule = Schedule::default();
        schedule.add_systems((
            gather_a11y_bounds_system,
            apply_a11y_bounds_system.after(gather_a11y_bounds_system),
        ));
        schedule.run(&mut world);

        // Verify the a11y tree is marked dirty
        let tree = world.resource::<A11yTree>();
        assert!(tree.is_dirty(a11y_id), "A11y node should be marked dirty");
    }

    #[test]
    fn test_register_action_callbacks() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let mut world = World::new();
        let mut a11y_tree = A11yTree::new();
        let a11y_root = a11y_tree.root();

        let button_node = A11yNode {
            role: A11yRole::Button,
            ..Default::default()
        };
        let a11y_id = a11y_tree.add_node(a11y_root, button_node);

        world.insert_resource(a11y_tree);
        world.insert_resource(ArthropodActionHandler::new());

        let was_called = Arc::new(AtomicBool::new(false));
        let was_called_clone = was_called.clone();

        let accessible = AccessibleNode::new(a11y_id, A11yRole::Button);
        let on_click = OnA11yClick::new(move || {
            was_called_clone.store(true, Ordering::SeqCst);
        });

        world.spawn((accessible, on_click));

        // Run the registration system
        let mut schedule = Schedule::default();
        schedule.add_systems(register_action_callbacks_system);
        schedule.run(&mut world);

        // Verify callback by dispatching via ActionHandler trait
        let mut handler = world.resource_mut::<ArthropodActionHandler>();
        handler.do_action(ActionRequest {
            action: accesskit::Action::Click,
            target_node: AccessKitNodeId(a11y_id.raw()),
            target_tree: TreeId(accesskit::Uuid::nil()),
            data: None,
        });

        assert!(was_called.load(Ordering::SeqCst), "Callback should execute");
    }

    #[test]
    fn test_register_multiple_callbacks() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let mut world = World::new();
        let mut a11y_tree = A11yTree::new();
        let a11y_root = a11y_tree.root();

        let node_a = A11yNode {
            role: A11yRole::Button,
            ..Default::default()
        };
        let id_a = a11y_tree.add_node(a11y_root, node_a);

        let node_b = A11yNode {
            role: A11yRole::Button,
            ..Default::default()
        };
        let id_b = a11y_tree.add_node(a11y_root, node_b);

        world.insert_resource(a11y_tree);
        world.insert_resource(ArthropodActionHandler::new());

        let counter = Arc::new(AtomicU32::new(0));

        let counter_a = counter.clone();
        let acc_a = AccessibleNode::new(id_a, A11yRole::Button);
        let click_a = OnA11yClick::new(move || {
            counter_a.fetch_add(1, Ordering::SeqCst);
        });
        world.spawn((acc_a, click_a));

        let counter_b = counter.clone();
        let acc_b = AccessibleNode::new(id_b, A11yRole::Button);
        let click_b = OnA11yClick::new(move || {
            counter_b.fetch_add(10, Ordering::SeqCst);
        });
        world.spawn((acc_b, click_b));

        let mut schedule = Schedule::default();
        schedule.add_systems(register_action_callbacks_system);
        schedule.run(&mut world);

        let mut handler = world.resource_mut::<ArthropodActionHandler>();
        handler.do_action(ActionRequest {
            action: accesskit::Action::Click,
            target_node: AccessKitNodeId(id_a.raw()),
            target_tree: TreeId(accesskit::Uuid::nil()),
            data: None,
        });
        handler.do_action(ActionRequest {
            action: accesskit::Action::Click,
            target_node: AccessKitNodeId(id_b.raw()),
            target_tree: TreeId(accesskit::Uuid::nil()),
            data: None,
        });

        assert_eq!(counter.load(Ordering::SeqCst), 11);
    }
}
