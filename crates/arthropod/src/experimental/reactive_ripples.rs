//! Reactive Ripples Feedback Effect
//!
//! An experimental feature that connects `flux-state` reactivity with visual feedback
//! in the scene graph. When a watched signal triggers, expanding "ripple" rings
//! (similar to water drops or material design touch feedback) are spawned and animated.

#[cfg(feature = "nova")]
use arthropod_ecs::components::SceneNodeRef;
#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use flux_state::ReadSignal;
#[cfg(feature = "nova")]
use plat_core::Rect;
#[cfg(feature = "nova")]
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};
#[cfg(feature = "nova")]
use style_engine::{CornerRadii, Paint, StrokeAlign, StrokeStyle, VisualStyle};

/// Component to emit ripple effects when a signal becomes true.
#[cfg(feature = "nova")]
#[derive(Component, Clone)]
pub struct ReactiveRippleEmitter {
    /// The signal that triggers a ripple when it transitions to true.
    pub trigger: ReadSignal<bool>,
    /// The color of the ripple.
    pub color: Color,
    /// Maximum radius the ripple will reach.
    pub max_radius: f32,
    /// Duration of the ripple animation in ticks/frames.
    pub duration_frames: u32,
    /// The stroke width of the ripple.
    pub stroke_width: f32,
    /// The last read state of the trigger to detect edge transitions.
    pub last_state: bool,
}

#[cfg(feature = "nova")]
impl ReactiveRippleEmitter {
    /// Creates a new ripple emitter bound to the given signal.
    pub fn new(trigger: ReadSignal<bool>) -> Self {
        let initial_state = trigger.get_untracked();
        Self {
            trigger,
            color: Color::rgba(0.0, 0.5, 1.0, 0.8), // Default to a nice cyan/blue
            max_radius: 100.0,
            duration_frames: 60, // roughly 1 second at 60fps
            stroke_width: 2.0,
            last_state: initial_state,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_radius(mut self, max_radius: f32) -> Self {
        self.max_radius = max_radius;
        self
    }
}

/// Internal component tracking an active, expanding ripple node.
#[cfg(feature = "nova")]
#[derive(Component)]
pub struct ActiveRipple {
    pub node_id: NodeId,
    pub max_radius: f32,
    pub current_frame: u32,
    pub duration_frames: u32,
}

/// System to poll the emitters and spawn new ripples on trigger.
#[cfg(feature = "nova")]
pub fn update_reactive_ripples(
    mut commands: Commands,
    mut scene: ResMut<Scene>,
    mut emitters: Query<(&mut ReactiveRippleEmitter, &SceneNodeRef)>,
    mut active_ripples: Query<(Entity, &mut ActiveRipple)>,
) {
    // 1. Process emitters and spawn new ripples
    for (mut emitter, node_ref) in emitters.iter_mut() {
        let current_state = emitter.trigger.get_untracked();

        if !current_state || emitter.last_state {
            emitter.last_state = current_state;
            continue;
        }

        if let Some(parent_node) = scene.get_node(node_ref.0) {
            let center_x = parent_node.bounds.width / 2.0;
            let center_y = parent_node.bounds.height / 2.0;

            let style = VisualStyle::new()
                .stroke(StrokeStyle::solid(
                    Paint::solid(emitter.color.as_vec4()),
                    emitter.stroke_width,
                    StrokeAlign::Center,
                ))
                .corner_radius(0.0);

            let mut ripple_node = SceneNode::new(NodeContent::Styled {
                style: Box::new(style),
            });
            ripple_node.bounds = Rect::new(center_x, center_y, 0.0, 0.0);

            let ripple_id = scene.add_node(node_ref.0, ripple_node);

            commands.spawn(ActiveRipple {
                node_id: ripple_id,
                max_radius: emitter.max_radius,
                current_frame: 0,
                duration_frames: emitter.duration_frames,
            });
        }

        emitter.last_state = current_state;
    }

    // 2. Animate and cleanup active ripples
    for (entity, mut ripple) in active_ripples.iter_mut() {
        ripple.current_frame += 1;
        let progress = ripple.current_frame as f32 / ripple.duration_frames as f32;

        if progress >= 1.0 {
            if scene.get_node(ripple.node_id).is_some() {
                scene.remove_node(ripple.node_id);
            }
            commands.entity(entity).despawn();
            continue;
        }

        let ease_out_quart = 1.0 - (1.0 - progress).powi(4);
        let current_radius = ripple.max_radius * ease_out_quart;
        let diameter = current_radius * 2.0;

        let opacity = 1.0 - progress;

        if let Some(node) = scene.get_node_mut(ripple.node_id) {
            let center_x = node.bounds.x + node.bounds.width / 2.0;
            let center_y = node.bounds.y + node.bounds.height / 2.0;

            node.bounds = Rect::new(
                center_x - current_radius,
                center_y - current_radius,
                diameter,
                diameter,
            );
            node.opacity = opacity;

            if let NodeContent::Styled { ref mut style } = node.content {
                style.corner_radii = CornerRadii::uniform(current_radius);
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
}

/// Registers the reactive ripples system.
#[cfg(feature = "nova")]
pub fn register_reactive_ripples(app: &mut crate::App) {
    app.add_update_system(update_reactive_ripples);
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};
    use render_engine::Scene;

    #[test]
    fn test_reactive_ripple_spawn_and_animate() {
        let mut world = World::new();
        let mut scene = Scene::new();
        let root = scene.root();

        // Create a target node
        let mut target_node = SceneNode::new(NodeContent::Empty);
        target_node.bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let target_id = scene.add_node(root, target_node);
        world.insert_resource(scene);

        let runtime = Runtime::new();
        let (read_sig, write_sig) = Signal::new(runtime, false).split();

        // Spawn emitter
        world.spawn((
            ReactiveRippleEmitter::new(read_sig),
            SceneNodeRef(target_id),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_reactive_ripples);

        // Run before trigger, no ripples should exist
        schedule.run(&mut world);
        assert_eq!(world.query::<&ActiveRipple>().iter(&world).count(), 0);

        // Trigger the signal
        write_sig.set(true);
        schedule.run(&mut world);

        // We expect one ripple to be spawned and be at frame 0 (since the spawning frame it wasn't processed by the animation loop because queries evaluate separately)
        // Actually, depending on system iteration order, maybe frame 0 or 1.
        let active_ripples: Vec<_> = world.query::<&ActiveRipple>().iter(&world).collect();
        assert_eq!(active_ripples.len(), 1);
        let ripple_node_id = active_ripples[0].node_id;

        // Verify the node exists in the scene
        let scene = world.resource::<Scene>();
        assert!(scene.get_node(ripple_node_id).is_some());

        // Run another frame to animate
        schedule.run(&mut world);
        let scene = world.resource::<Scene>();
        let node = scene.get_node(ripple_node_id).unwrap();

        // It should have expanded slightly
        assert!(node.bounds.width > 0.0);
        assert!(node.opacity < 1.0);
    }
}
