//! Glass Overlay System
//!
//! An experimental feature that renders a "frosted glass" overlay effect
//! over specific nodes or the whole screen, connecting `flux-state` reactivity
//! with the `render-engine` blur primitives.
//!
//! Useful for modals, focus modes, or stylized UI layers.

#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};
#[cfg(feature = "nova")]
use style_engine::{BackgroundBlur, Effect, VisualStyle};

/// Configuration for the Glass Overlay system.
#[cfg(feature = "nova")]
#[derive(Resource)]
pub struct GlassOverlayConfig {
    /// Whether the glass overlay system is active.
    pub enabled: bool,
    /// The color tint of the frosted glass (e.g., slightly white/transparent).
    pub tint: Color,
    /// The blur radius for the frosted glass effect.
    pub blur_radius: f32,
}

#[cfg(feature = "nova")]
impl Default for GlassOverlayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tint: Color::rgba(1.0, 1.0, 1.0, 0.1),
            blur_radius: 10.0,
        }
    }
}

/// The target bounds or node of the glass overlay.
#[cfg(feature = "nova")]
#[derive(Resource, Default)]
pub struct GlassOverlayTarget {
    /// The specific node to overlay with frosted glass.
    /// If None, can be used for fullscreen.
    pub target_id: Option<NodeId>,
}

/// Internal state for the glass overlay system.
#[cfg(feature = "nova")]
#[derive(Resource, Default)]
pub struct GlassOverlayState {
    /// The ID of the overlay node we created in the scene.
    pub overlay_node_id: Option<NodeId>,
}

/// System that maintains the glass overlay.
#[cfg(feature = "nova")]
pub fn update_glass_overlay(
    mut scene: ResMut<Scene>,
    config: Res<GlassOverlayConfig>,
    target: Res<GlassOverlayTarget>,
    mut state: ResMut<GlassOverlayState>,
) {
    if !config.enabled || target.target_id.is_none() {
        if let Some(overlay_id) = state.overlay_node_id.take()
            && scene.get_node(overlay_id).is_some()
        {
            scene.remove_node(overlay_id);
        }
        return;
    }

    let target_id = target.target_id.unwrap();

    let target_bounds = match scene.get_node(target_id) {
        Some(node) => node.bounds,
        None => {
            if let Some(overlay_id) = state.overlay_node_id.take()
                && scene.get_node(overlay_id).is_some()
            {
                scene.remove_node(overlay_id);
            }
            return;
        }
    };

    let mut style = VisualStyle::new();
    style.fills = vec![style_engine::Paint::Solid(config.tint.0)];
    style.effects = vec![Effect::BackgroundBlur(BackgroundBlur::new(
        config.blur_radius,
    ))];

    if let Some(overlay_id) = state.overlay_node_id {
        if let Some(overlay_node) = scene.get_node_mut(overlay_id) {
            overlay_node.content = NodeContent::Styled {
                style: Box::new(style),
            };
            overlay_node.bounds = target_bounds;
        }
    } else {
        let root = scene.root();
        let mut overlay_node = SceneNode::new(NodeContent::Styled {
            style: Box::new(style),
        });
        overlay_node.bounds = target_bounds;
        // Append at root so it renders on top
        let id = scene.add_node(root, overlay_node);
        state.overlay_node_id = Some(id);
    }
}

#[cfg(feature = "nova")]
pub fn register_glass_overlay(app: &mut crate::App) {
    app.world_mut().init_resource::<GlassOverlayConfig>();
    app.world_mut().init_resource::<GlassOverlayTarget>();
    app.world_mut().init_resource::<GlassOverlayState>();
    app.add_update_system(update_glass_overlay);
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use render_engine::{NodeContent, Scene, SceneNode};

    #[test]
    fn test_glass_overlay_creation_and_removal() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut target_node = SceneNode::new(NodeContent::Empty);
        target_node.bounds = plat_core::Rect::new(10.0, 10.0, 100.0, 100.0);
        let target_id = scene.add_node(root, target_node);

        let mut world = World::new();
        world.insert_resource(scene);
        world.insert_resource(GlassOverlayConfig {
            enabled: true,
            tint: Color::WHITE,
            blur_radius: 5.0,
        });
        world.insert_resource(GlassOverlayTarget {
            target_id: Some(target_id),
        });
        world.insert_resource(GlassOverlayState::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(update_glass_overlay);

        schedule.run(&mut world);

        let state = world.resource::<GlassOverlayState>();
        assert!(state.overlay_node_id.is_some());
        let overlay_id = state.overlay_node_id.unwrap();

        let scene = world.resource::<Scene>();
        let overlay_node = scene.get_node(overlay_id).unwrap();
        assert_eq!(overlay_node.bounds.x, 10.0);
        assert_eq!(overlay_node.bounds.width, 100.0);

        world.resource_mut::<GlassOverlayConfig>().enabled = false;
        schedule.run(&mut world);

        let state = world.resource::<GlassOverlayState>();
        assert!(state.overlay_node_id.is_none());
    }
}
