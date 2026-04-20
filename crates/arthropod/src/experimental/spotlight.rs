//! Spotlight Overlay System
//!
//! An experimental feature that visually highlights a specific node
//! by dimming the rest of the screen. Useful for tutorials or onboarding.

#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};

/// Configuration for the Spotlight system.
#[cfg(feature = "nova")]
#[derive(Resource)]
pub struct SpotlightConfig {
    /// Whether the spotlight system is active.
    pub enabled: bool,
    /// The color of the backdrop overlay (typically a translucent black).
    pub backdrop_color: Color,
}

#[cfg(feature = "nova")]
impl Default for SpotlightConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backdrop_color: Color::rgba(0.0, 0.0, 0.0, 0.7),
        }
    }
}

/// The target of the spotlight.
#[cfg(feature = "nova")]
#[derive(Resource, Default)]
pub struct SpotlightTarget {
    /// The specific node to highlight.
    pub target_id: Option<NodeId>,
}

/// Internal state for the spotlight system.
#[cfg(feature = "nova")]
#[derive(Resource, Default)]
pub struct SpotlightState {
    /// The ID of the overlay node we created in the scene.
    pub overlay_node_id: Option<NodeId>,
}

/// System that maintains the spotlight overlay.
#[cfg(feature = "nova")]
pub fn update_spotlight(
    mut scene: ResMut<Scene>,
    config: Res<SpotlightConfig>,
    target: Res<SpotlightTarget>,
    mut state: ResMut<SpotlightState>,
) {
    if !config.enabled || target.target_id.is_none() {
        // Clear the spotlight if it's disabled or has no target
        if let Some(overlay_id) = state.overlay_node_id.take()
            && scene.get_node(overlay_id).is_some()
        {
            scene.remove_node(overlay_id);
        }
        return;
    }

    let target_id = target.target_id.unwrap();

    // Check if the target node exists and get its bounds
    let target_bounds = match scene.get_node(target_id) {
        Some(node) => node.bounds,
        None => {
            // Target disappeared, clean up the overlay
            if let Some(overlay_id) = state.overlay_node_id.take()
                && scene.get_node(overlay_id).is_some()
            {
                scene.remove_node(overlay_id);
            }
            return;
        }
    };

    // For the overlay, we create a full screen rect using a massive size.
    // A more robust implementation would hook into the window size, but for the scene graph
    // a very large rect will cover the screen. We use a hollow path (a rect with a hole).
    // For simplicity in this experiment, we'll draw four solid color rects around the target.

    // The "hole punch" logic can be done with style_engine::VectorPath.
    use render_engine::Vec2;
    use style_engine::VectorPath;
    use style_engine::{Paint, VisualStyle};

    // Path for the full screen (assuming standard massive bounds like 10000x10000 centered at origin)
    let screen_w = 10000.0;
    let screen_h = 10000.0;
    let mut background_path = VectorPath::new();
    // Clockwise full screen
    background_path.move_to(Vec2::new(0.0, 0.0));
    background_path.line_to(Vec2::new(screen_w, 0.0));
    background_path.line_to(Vec2::new(screen_w, screen_h));
    background_path.line_to(Vec2::new(0.0, screen_h));
    background_path.close();

    // Calculate bounds corners
    let min_x = target_bounds.x;
    let min_y = target_bounds.y;
    let max_x = target_bounds.x + target_bounds.width;
    let max_y = target_bounds.y + target_bounds.height;

    // Counter-clockwise hole punch (EvenOdd winding rule will make this a hole)
    background_path.move_to(Vec2::new(min_x, min_y));
    background_path.line_to(Vec2::new(min_x, max_y));
    background_path.line_to(Vec2::new(max_x, max_y));
    background_path.line_to(Vec2::new(max_x, min_y));
    background_path.close();

    let mut style = VisualStyle::new();
    style.fills = vec![Paint::Solid(config.backdrop_color.0)];
    style.fill_geometry = Some(vec![background_path]);

    if let Some(overlay_id) = state.overlay_node_id {
        // Update existing
        if let Some(overlay_node) = scene.get_node_mut(overlay_id) {
            overlay_node.content = NodeContent::Styled {
                style: Box::new(style),
            };
            overlay_node.bounds = plat_core::Rect::new(0.0, 0.0, screen_w, screen_h);
        }
    } else {
        // Create new
        let root = scene.root();
        let mut overlay_node = SceneNode::new(NodeContent::Styled {
            style: Box::new(style),
        });
        overlay_node.bounds = plat_core::Rect::new(0.0, 0.0, screen_w, screen_h);
        // Put it at the root level, but appended so it renders on top
        let id = scene.add_node(root, overlay_node);
        state.overlay_node_id = Some(id);
    }
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use render_engine::{NodeContent, Scene, SceneNode};

    #[test]
    fn test_spotlight_adds_and_removes_overlay() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Create a target node
        let target_node = SceneNode::new(NodeContent::Empty);
        let target_id = scene.add_node(root, target_node);

        // Setup resources
        let config = SpotlightConfig {
            enabled: true,
            backdrop_color: Color::BLACK,
        };
        let target = SpotlightTarget {
            target_id: Some(target_id),
        };
        let state = SpotlightState::default();

        // Run system
        // We mock ResMut by passing mutable references (in tests we often just pass the inner data, but here we can't easily mock ResMut without a full World. Let's just create a World).

        let mut world = World::new();
        world.insert_resource(scene);
        world.insert_resource(config);
        world.insert_resource(target);
        world.insert_resource(state);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_spotlight);

        // Run once: should create overlay
        schedule.run(&mut world);

        let scene = world.resource::<Scene>();
        let state = world.resource::<SpotlightState>();

        assert!(state.overlay_node_id.is_some(), "Overlay should be created");
        let overlay_id = state.overlay_node_id.unwrap();
        assert!(
            scene.get_node(overlay_id).is_some(),
            "Overlay node should be in scene"
        );

        // Run again with disabled config: should remove overlay
        world.resource_mut::<SpotlightConfig>().enabled = false;
        schedule.run(&mut world);

        let scene = world.resource::<Scene>();
        let state = world.resource::<SpotlightState>();

        assert!(
            state.overlay_node_id.is_none(),
            "Overlay state should be cleared"
        );
        assert!(
            scene.get_node(overlay_id).is_none(),
            "Overlay node should be removed from scene"
        );
    }
}
