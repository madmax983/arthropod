//! Gesture Feedback System
//!
//! An experimental feature that provides visual feedback for mouse gestures.
//! It tracks right-click mouse movements using `input_engine::StrokeMatcher`
//! and draws a trail of nodes in the scene.

#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use input_engine::{InputPattern, StrokeMatcher};
#[cfg(feature = "nova")]
use plat_core::{MouseButton, WindowEvent};
#[cfg(feature = "nova")]
use render_engine::{Color, NodeContent, NodeId, Rect, Scene, SceneNode};
#[cfg(feature = "nova")]
use style_engine::{Paint, StrokeAlign, StrokeStyle, VisualStyle};

/// Input resource for gesture feedback.
#[cfg(feature = "nova")]
#[derive(Resource, Default)]
pub struct GestureFeedbackInput {
    pub events: Vec<WindowEvent>,
}

/// Internal state for tracking gesture feedback.
#[cfg(feature = "nova")]
#[derive(Resource)]
pub struct GestureFeedbackState {
    pub matcher: StrokeMatcher,
    pub trail_nodes: Vec<NodeId>,
}

#[cfg(feature = "nova")]
impl Default for GestureFeedbackState {
    fn default() -> Self {
        Self {
            matcher: StrokeMatcher::new(MouseButton::Right),
            trail_nodes: Vec::new(),
        }
    }
}

/// Configuration for the Gesture Feedback system.
#[cfg(feature = "nova")]
#[derive(Resource)]
pub struct GestureFeedbackConfig {
    pub enabled: bool,
    pub trail_color: Color,
    pub trail_thickness: f32,
    pub max_trail_points: usize,
}

#[cfg(feature = "nova")]
impl Default for GestureFeedbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            trail_color: Color::rgba(0.0, 1.0, 1.0, 0.8), // Cyan
            trail_thickness: 4.0,
            max_trail_points: 100,
        }
    }
}

/// System that maintains the gesture feedback trail.
#[cfg(feature = "nova")]
pub fn update_gesture_feedback(
    mut scene: ResMut<Scene>,
    config: Res<GestureFeedbackConfig>,
    mut input: ResMut<GestureFeedbackInput>,
    mut state: ResMut<GestureFeedbackState>,
) {
    if !config.enabled {
        clear_trail(&mut scene, &mut state);
        return;
    }

    let mut is_tracking = state.matcher.is_tracking();

    for event in input.events.drain(..) {
        let _gesture = state.matcher.update(&event);
        is_tracking = state.matcher.is_tracking();

        // If tracking just started or is ongoing, add a node for the new point
        if is_tracking {
            if let WindowEvent::CursorMoved { position } = event {
                add_trail_point(
                    &mut scene,
                    &mut state,
                    &config,
                    position.x as f32,
                    position.y as f32,
                );
            } else if let WindowEvent::MouseInput(mouse_input) = event {
                // Initial press point
                if mouse_input.state == plat_core::ElementState::Pressed
                    && mouse_input.button == MouseButton::Right
                {
                    add_trail_point(
                        &mut scene,
                        &mut state,
                        &config,
                        mouse_input.position.x as f32,
                        mouse_input.position.y as f32,
                    );
                }
            }
        }
    }

    if !is_tracking {
        clear_trail(&mut scene, &mut state);
    }
}

#[cfg(feature = "nova")]
fn add_trail_point(
    scene: &mut Scene,
    state: &mut GestureFeedbackState,
    config: &GestureFeedbackConfig,
    x: f32,
    y: f32,
) {
    let style = VisualStyle::new()
        .stroke(StrokeStyle::solid(
            Paint::solid(config.trail_color.as_vec4()),
            config.trail_thickness,
            StrokeAlign::Center,
        ))
        .solid_fill(config.trail_color.as_vec4()); // Make it visible even if stroke fails

    let size = config.trail_thickness * 2.0;
    let mut node = SceneNode::new(NodeContent::Styled {
        style: Box::new(style),
    });
    node.bounds = Rect::new(x - size / 2.0, y - size / 2.0, size, size);

    let root = scene.root();
    let id = scene.add_node(root, node);
    state.trail_nodes.push(id);

    // Limit trail length
    if state.trail_nodes.len() > config.max_trail_points {
        let old_id = state.trail_nodes.remove(0);
        if scene.get_node(old_id).is_some() {
            scene.remove_node(old_id);
        }
    }
}

#[cfg(feature = "nova")]
fn clear_trail(scene: &mut Scene, state: &mut GestureFeedbackState) {
    for &id in &state.trail_nodes {
        if scene.get_node(id).is_some() {
            scene.remove_node(id);
        }
    }
    state.trail_nodes.clear();
}

#[cfg(feature = "nova")]
pub fn register_gesture_feedback(app: &mut crate::App) {
    app.world_mut().init_resource::<GestureFeedbackConfig>();
    app.world_mut().init_resource::<GestureFeedbackInput>();
    app.world_mut().init_resource::<GestureFeedbackState>();
    app.add_update_system(update_gesture_feedback);
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use plat_core::{ElementState, Modifiers, MouseInput, Point};

    #[test]
    fn test_gesture_feedback_trail() {
        let mut world = World::new();
        let scene = Scene::new();
        world.insert_resource(scene);
        world.insert_resource(GestureFeedbackConfig::default());
        world.insert_resource(GestureFeedbackState::default());
        world.insert_resource(GestureFeedbackInput::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(update_gesture_feedback);

        let points = [
            Point::new(10.0, 10.0),
            Point::new(20.0, 10.0),
            Point::new(30.0, 10.0),
        ];

        let mut events = Vec::new();
        // Press
        events.push(WindowEvent::MouseInput(MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Right,
            position: points[0],
            modifiers: Modifiers::default(),
        }));
        // Move
        for p in points.iter().skip(1) {
            events.push(WindowEvent::CursorMoved { position: *p });
        }

        world.resource_mut::<GestureFeedbackInput>().events = events;

        schedule.run(&mut world);

        let state = world.resource::<GestureFeedbackState>();
        let scene = world.resource::<Scene>();

        // We expect 3 nodes created (1 for press, 2 for move)
        assert_eq!(state.trail_nodes.len(), 3);

        for &id in &state.trail_nodes {
            assert!(scene.get_node(id).is_some());
        }

        // Release to clear
        world.resource_mut::<GestureFeedbackInput>().events =
            vec![WindowEvent::MouseInput(MouseInput {
                state: ElementState::Released,
                button: MouseButton::Right,
                position: *points.last().unwrap(),
                modifiers: Modifiers::default(),
            })];

        schedule.run(&mut world);

        let state = world.resource::<GestureFeedbackState>();
        assert!(state.trail_nodes.is_empty());
    }
}
