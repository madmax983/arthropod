//! Mouse Trail System
//!
//! An experimental feature that creates a trail of visual nodes that follow
//! the mouse cursor, fading out over time.

#[cfg(feature = "nova")]
use arthropod_ecs::components::MousePosition;
#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode, Vec2};
#[cfg(feature = "nova")]
use style_engine::VisualStyle;

/// Configuration for the Mouse Trail system.
#[cfg(feature = "nova")]
#[derive(Resource)]
pub struct MouseTrailConfig {
    /// Whether the mouse trail system is active.
    pub enabled: bool,
    /// The color of the trail.
    pub color: Color,
    /// The size (width/height) of the trail nodes.
    pub size: f32,
    /// How much opacity is lost per tick.
    pub fade_speed: f32,
}

#[cfg(feature = "nova")]
impl Default for MouseTrailConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: Color::WHITE,
            size: 10.0,
            fade_speed: 0.05,
        }
    }
}

/// Internal state for the mouse trail system.
#[cfg(feature = "nova")]
#[derive(Resource, Default)]
pub struct MouseTrailState {
    /// The IDs of the overlay nodes we created in the scene.
    pub trail_nodes: Vec<NodeId>,
    pub last_mouse_pos: Option<Vec2>,
}

/// System that maintains the mouse trail.
#[cfg(feature = "nova")]
pub fn update_mouse_trail(
    mut scene: ResMut<Scene>,
    config: Res<MouseTrailConfig>,
    mouse_pos: Res<MousePosition>,
    mut state: ResMut<MouseTrailState>,
) {
    if !config.enabled {
        // Clear the trail if disabled
        for &id in &state.trail_nodes {
            if scene.get_node(id).is_some() {
                scene.remove_node(id);
            }
        }
        state.trail_nodes.clear();
        state.last_mouse_pos = None;
        return;
    }

    let current_pos = mouse_pos.0;

    // Fade out existing nodes and remove them if they are invisible
    state.trail_nodes.retain(|&id| {
        if let Some(node) = scene.get_node_mut(id) {
            node.opacity -= config.fade_speed;
            if node.opacity <= 0.0 {
                scene.remove_node(id);
                false
            } else {
                true
            }
        } else {
            false
        }
    });

    // Spawn a new node if the mouse has moved
    let should_spawn = match state.last_mouse_pos {
        Some(last_pos) => (current_pos - last_pos).length_squared() > 1.0,
        None => true,
    };

    if should_spawn {
        let root = scene.root();
        let mut style = VisualStyle::new();
        style.fills = vec![style_engine::Paint::Solid(config.color.0)];

        // Use a simple circle shape if we can, but a rectangle is easier for now.
        // The Arthropod VisualStyle corner radius can make it circular:
        style.corner_radii = style_engine::CornerRadii::uniform(config.size / 2.0);

        let mut new_node = SceneNode::new(NodeContent::Styled {
            style: Box::new(style),
        });

        new_node.bounds = plat_core::Rect::new(
            current_pos.x - config.size / 2.0,
            current_pos.y - config.size / 2.0,
            config.size,
            config.size,
        );
        new_node.opacity = 1.0;

        let id = scene.add_node(root, new_node);
        state.trail_nodes.push(id);
        state.last_mouse_pos = Some(current_pos);
    }
}

#[cfg(feature = "nova")]
pub fn register_mouse_trail(app: &mut crate::App) {
    app.world_mut().insert_resource(MouseTrailConfig::default());
    app.world_mut().insert_resource(MouseTrailState::default());
    app.add_update_system(update_mouse_trail);
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use render_engine::Scene;

    #[test]
    fn test_mouse_trail_spawns_and_fades_nodes() {
        let scene = Scene::new();

        let config = MouseTrailConfig {
            enabled: true,
            color: Color::WHITE,
            size: 10.0,
            fade_speed: 0.5,
        };
        let mouse_pos = MousePosition(Vec2::new(100.0, 100.0));
        let state = MouseTrailState::default();

        let mut world = World::new();
        world.insert_resource(scene);
        world.insert_resource(config);
        world.insert_resource(mouse_pos);
        world.insert_resource(state);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_mouse_trail);

        // Run once: should spawn a new trail node
        schedule.run(&mut world);

        {
            let state = world.resource::<MouseTrailState>();
            assert_eq!(state.trail_nodes.len(), 1);
            assert_eq!(state.last_mouse_pos, Some(Vec2::new(100.0, 100.0)));
            let scene = world.resource::<Scene>();
            let node = scene.get_node(state.trail_nodes[0]).unwrap();
            assert_eq!(node.opacity, 1.0);
        }

        // Run again with mouse at same position: no new node, existing node fades
        schedule.run(&mut world);

        {
            let state = world.resource::<MouseTrailState>();
            assert_eq!(state.trail_nodes.len(), 1);
            let scene = world.resource::<Scene>();
            let node = scene.get_node(state.trail_nodes[0]).unwrap();
            assert_eq!(node.opacity, 0.5);
        }

        // Move mouse
        world.resource_mut::<MousePosition>().0 = Vec2::new(150.0, 150.0);

        // Run again: spawn new node, old node fades to 0 and is removed
        schedule.run(&mut world);

        {
            let state = world.resource::<MouseTrailState>();
            // The old node fades to 0 and is removed, and a new one is added
            assert_eq!(state.trail_nodes.len(), 1);
            let scene = world.resource::<Scene>();
            let node = scene.get_node(state.trail_nodes[0]).unwrap();
            assert_eq!(node.opacity, 1.0);
            assert_eq!(node.bounds.x, 145.0); // 150 - 5
        }
    }
}
