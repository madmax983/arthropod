//! Parallax UI System
//!
//! An experimental feature that gives UI elements depth by making them move slightly
//! in opposition to the mouse cursor, similar to the Apple TV parallax effect.

#[cfg(feature = "nova")]
use arthropod_ecs::components::{LayoutConstraintsResource, MousePosition, SceneNodeRef};
#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use render_engine::{Scene, Transform2D};

/// Component to enable parallax effect on a UI node.
#[cfg(feature = "nova")]
#[derive(Component, Debug, Clone)]
pub struct ParallaxNode {
    /// The strength of the parallax effect.
    /// Higher values mean more movement. Default is 0.05.
    pub strength: f32,
    /// If true, moves in the same direction as the mouse instead of opposite.
    pub invert: bool,
    /// The original translation of the node before parallax is applied.
    /// This is used as the baseline for the offset.
    pub original_translation: Option<render_engine::Vec2>,
}

#[cfg(feature = "nova")]
impl Default for ParallaxNode {
    fn default() -> Self {
        Self {
            strength: 0.05,
            invert: false,
            original_translation: None,
        }
    }
}

/// System that updates the transform of nodes with ParallaxNode based on mouse position.
#[cfg(feature = "nova")]
pub fn update_parallax(
    mut scene: ResMut<Scene>,
    mouse_pos: Res<MousePosition>,
    constraints: Option<Res<LayoutConstraintsResource>>,
    mut query: Query<(&mut ParallaxNode, &SceneNodeRef)>,
) {
    let (window_width, window_height) = if let Some(c) = constraints {
        (
            c.0.max_width.unwrap_or(800.0),
            c.0.max_height.unwrap_or(600.0),
        )
    } else {
        (800.0, 600.0)
    };

    let center_x = window_width / 2.0;
    let center_y = window_height / 2.0;

    // Calculate normalized mouse offset from center (-1.0 to 1.0)
    // Prevent divide-by-zero if constraints are 0
    let offset_x = if center_x > 0.0001 {
        (mouse_pos.0.x - center_x) / center_x
    } else {
        0.0
    };
    let offset_y = if center_y > 0.0001 {
        (mouse_pos.0.y - center_y) / center_y
    } else {
        0.0
    };

    for (mut parallax, node_ref) in query.iter_mut() {
        if let Some(node) = scene.get_node_mut(node_ref.0) {
            // Capture the original translation on the first run
            let base_translation = parallax.original_translation.unwrap_or_else(|| {
                let trans = node.transform.translation();
                parallax.original_translation = Some(trans);
                trans
            });

            // Calculate parallax shift
            let mut shift_x = offset_x * window_width * parallax.strength;
            let mut shift_y = offset_y * window_height * parallax.strength;

            if parallax.invert {
                shift_x = -shift_x;
                shift_y = -shift_y;
            }

            // Apply the shift relative to the original translation
            // Using translation and adding offset.
            let mut affine = node.transform.as_affine2();
            affine.translation = base_translation + render_engine::Vec2::new(-shift_x, -shift_y);
            node.transform = Transform2D::from_affine2(affine);
        }
    }
}

#[cfg(feature = "nova")]
pub fn register_parallax(app: &mut crate::App) {
    app.add_update_system(update_parallax);
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use layout_engine::LayoutConstraints;
    use render_engine::{NodeContent, SceneNode};

    #[test]
    fn test_parallax_movement() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Create a target node at (100, 100)
        let mut target_node = SceneNode::new(NodeContent::Empty);
        target_node.transform = Transform2D::translate(100.0, 100.0);
        let target_id = scene.add_node(root, target_node);

        let mut world = World::new();

        // Setup Window Constraints: 1000x1000
        let constraints = LayoutConstraints {
            max_width: Some(1000.0),
            max_height: Some(1000.0),
            ..Default::default()
        };
        world.insert_resource(LayoutConstraintsResource(constraints));

        // Setup Mouse Position: (750, 750) -> Top Right quadrant
        // Normalized offset from center (500,500) will be (+0.5, +0.5)
        world.insert_resource(MousePosition(render_engine::Vec2::new(750.0, 750.0)));

        let mut entity_mut = world.spawn_empty();
        entity_mut.insert(ParallaxNode {
            strength: 0.1,
            invert: false,
            original_translation: None,
        });
        entity_mut.insert(SceneNodeRef(target_id));

        world.insert_resource(scene);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_parallax);

        // Run system
        schedule.run(&mut world);

        let scene = world.resource::<Scene>();
        let node = scene.get_node(target_id).unwrap();

        let translation = node.transform.translation();

        // Expected shift:
        // offset_x = 0.5
        // shift_x = 0.5 * 1000.0 * 0.1 = 50.0
        // Because invert is false, the code applies -shift_x: -50.0
        // Original pos: 100.0 -> New pos: 50.0

        assert_eq!(translation.x, 50.0);
        assert_eq!(translation.y, 50.0);
    }

    #[test]
    fn test_parallax_movement_inverted() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Create a target node at (100, 100)
        let mut target_node = SceneNode::new(NodeContent::Empty);
        target_node.transform = Transform2D::translate(100.0, 100.0);
        let target_id = scene.add_node(root, target_node);

        let mut world = World::new();

        // Setup Window Constraints: 1000x1000
        let constraints = LayoutConstraints {
            max_width: Some(1000.0),
            max_height: Some(1000.0),
            ..Default::default()
        };
        world.insert_resource(LayoutConstraintsResource(constraints));

        // Setup Mouse Position: (750, 750) -> Top Right quadrant
        world.insert_resource(MousePosition(render_engine::Vec2::new(750.0, 750.0)));

        let mut entity_mut = world.spawn_empty();
        entity_mut.insert(ParallaxNode {
            strength: 0.1,
            invert: true, // INVERTED
            original_translation: None,
        });
        entity_mut.insert(SceneNodeRef(target_id));

        world.insert_resource(scene);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_parallax);

        // Run system
        schedule.run(&mut world);

        let scene = world.resource::<Scene>();
        let node = scene.get_node(target_id).unwrap();

        let translation = node.transform.translation();

        // Expected shift inverted:
        // offset_x = 0.5 -> shift_x = 50.0 -> invert = -50.0
        // applied: -(-50.0) = +50.0
        // Original pos: 100.0 -> New pos: 150.0

        assert_eq!(translation.x, 150.0);
        assert_eq!(translation.y, 150.0);
    }
}
