#[cfg(feature = "nova")]
use arthropod_ecs::components::{InteractionState, MousePosition, SceneNodeRef};
#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use render_engine::{Scene, Transform2D, Vec2};

/// Component to make a UI node draggable by the mouse.
#[cfg(feature = "nova")]
#[derive(Component, Debug, Clone)]
pub struct DraggableNode {
    /// Whether the node is currently being dragged.
    pub is_dragging: bool,
    /// The offset from the node's origin to the mouse cursor when dragging started.
    pub drag_offset: Vec2,
    /// Whether dragging is enabled.
    pub enabled: bool,
}

#[cfg(feature = "nova")]
impl Default for DraggableNode {
    fn default() -> Self {
        Self {
            is_dragging: false,
            drag_offset: Vec2::ZERO,
            enabled: true,
        }
    }
}

/// System that handles dragging of nodes with DraggableNode.
#[cfg(feature = "nova")]
pub fn update_draggable_nodes(
    mut scene: ResMut<Scene>,
    mouse_pos: Res<MousePosition>,
    mut query: Query<(&mut DraggableNode, &InteractionState, &SceneNodeRef)>,
) {
    for (mut draggable, interaction, node_ref) in query.iter_mut() {
        if !draggable.enabled {
            draggable.is_dragging = false;
            continue;
        }

        let mouse_vec = mouse_pos.0;

        if let Some(node) = scene.get_node_mut(node_ref.0) {
            if interaction.active && !draggable.is_dragging {
                // Just started dragging
                draggable.is_dragging = true;
                // Calculate offset from node's current position to mouse
                let node_pos = node.transform.translation();
                draggable.drag_offset = mouse_vec - node_pos;
            } else if !interaction.active && draggable.is_dragging {
                // Stopped dragging
                draggable.is_dragging = false;
            }

            if draggable.is_dragging {
                // Update position based on mouse position and initial offset
                let new_pos = mouse_vec - draggable.drag_offset;
                let mut affine = node.transform.as_affine2();
                affine.translation = new_pos;
                node.transform = Transform2D::from_affine2(affine);
            }
        }
    }
}

#[cfg(feature = "nova")]
pub fn register_draggable(app: &mut crate::App) {
    app.add_update_system(update_draggable_nodes);
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use render_engine::{NodeContent, SceneNode};

    #[test]
    fn test_draggable_node() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut target_node = SceneNode::new(NodeContent::Empty);
        target_node.transform = Transform2D::translate(100.0, 100.0);
        let target_id = scene.add_node(root, target_node);

        let mut world = World::new();

        world.insert_resource(MousePosition(Vec2::new(150.0, 150.0)));

        let mut entity_mut = world.spawn_empty();
        entity_mut.insert(DraggableNode::default());
        entity_mut.insert(SceneNodeRef(target_id));

        let interaction = InteractionState {
            active: true,
            ..Default::default()
        }; // Simulate mouse down
        entity_mut.insert(interaction);

        world.insert_resource(scene);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_draggable_nodes);

        // Run system - should start dragging and calculate offset
        schedule.run(&mut world);

        let draggable = world.query::<&DraggableNode>().single(&world);
        assert!(draggable.is_dragging);
        assert_eq!(draggable.drag_offset, Vec2::new(50.0, 50.0));

        // Move mouse
        world.insert_resource(MousePosition(Vec2::new(200.0, 200.0)));
        schedule.run(&mut world);

        let scene = world.resource::<Scene>();
        let node = scene.get_node(target_id).unwrap();
        let translation = node.transform.translation();

        // Expected pos: mouse (200, 200) - offset (50, 50) = (150, 150)
        assert_eq!(translation.x, 150.0);
        assert_eq!(translation.y, 150.0);

        // Release mouse
        let mut interaction = world
            .query::<&mut InteractionState>()
            .single_mut(&mut world);
        interaction.active = false;

        schedule.run(&mut world);

        let draggable = world.query::<&DraggableNode>().single(&world);
        assert!(!draggable.is_dragging);
    }
}
