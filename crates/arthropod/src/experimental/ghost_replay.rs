use arthropod_ecs::components::SceneNodeRef;
use bevy_ecs::prelude::*;
use plat_core::{ElementState, Event, MouseButton, WindowEvent};
use render_engine::{Color, NodeContent, NodeId, Paint, Scene, SceneNode, Vec2, VisualStyle};
use std::time::{Duration, Instant};

// =============================================================================
// Data Structures
// =============================================================================

#[derive(Debug, Clone)]
pub struct GhostEvent {
    pub timestamp: Duration,
    pub kind: GhostEventKind,
}

#[derive(Debug, Clone)]
pub enum GhostEventKind {
    CursorMoved(Vec2),
    Click(MouseButton),
    Release(MouseButton),
}

/// Resource to store recorded events.
#[derive(Resource, Default)]
pub struct GhostRecorder {
    pub events: Vec<GhostEvent>,
    pub start_time: Option<Instant>,
    pub recording: bool,
}

impl GhostRecorder {
    pub fn start(&mut self) {
        self.events.clear();
        self.start_time = Some(Instant::now());
        self.recording = true;
    }

    pub fn stop(&mut self) {
        self.recording = false;
        // Keep events for replay
    }
}

/// Resource to manage playback state.
#[derive(Resource)]
pub struct GhostReplayer {
    pub events: Vec<GhostEvent>,
    pub start_time: Option<Instant>,
    pub playing: bool,
    pub current_index: usize,
    pub speed: f32,
    /// Track the visual cursor node ID to avoid ECS query lag
    pub cursor_node: Option<NodeId>,
}

impl Default for GhostReplayer {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            start_time: None,
            playing: false,
            current_index: 0,
            speed: 1.0,
            cursor_node: None,
        }
    }
}

impl GhostReplayer {
    pub fn play(&mut self, events: Vec<GhostEvent>) {
        self.events = events;
        self.start_time = Some(Instant::now());
        self.playing = true;
        self.current_index = 0;
        self.cursor_node = None; // Reset cursor
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.start_time = None;
        self.cursor_node = None;
    }
}

/// Component for the visual cursor entity.
#[derive(Component)]
pub struct GhostCursor;

// =============================================================================
// Helper Functions
// =============================================================================

/// Helper to record an event.
/// Called from `WidgetApp::on_event`.
pub fn record_event(world: &mut World, event: &Event) {
    // Check if recorder exists and is recording
    // We use a scope to limit borrow of world
    let (should_record, start_time) = if let Some(recorder) = world.get_resource::<GhostRecorder>()
    {
        (recorder.recording, recorder.start_time)
    } else {
        (false, None)
    };

    if !should_record || start_time.is_none() {
        return;
    }

    let start = start_time.unwrap();
    let timestamp = start.elapsed();

    let kind = match event {
        Event::Window {
            event: WindowEvent::CursorMoved { position },
            ..
        } => Some(GhostEventKind::CursorMoved(Vec2::new(
            position.x as f32,
            position.y as f32,
        ))),
        Event::Window {
            event: WindowEvent::MouseInput(input),
            ..
        } => match input.state {
            ElementState::Pressed => Some(GhostEventKind::Click(input.button)),
            ElementState::Released => Some(GhostEventKind::Release(input.button)),
        },
        _ => None,
    };

    if let Some(kind) = kind {
        // Now get mut access to push event
        if let Some(mut recorder) = world.get_resource_mut::<GhostRecorder>() {
            recorder.events.push(GhostEvent { timestamp, kind });
        }
    }
}

// =============================================================================
// Systems
// =============================================================================

pub fn update_ghost_replay(
    mut commands: Commands,
    mut replayer: ResMut<GhostReplayer>,
    mut scene: ResMut<Scene>,
    // We use query to cleanup entities if needed.
    cursor_query: Query<(Entity, &SceneNodeRef), With<GhostCursor>>,
) {
    if !replayer.playing || replayer.start_time.is_none() {
        return;
    }

    let elapsed_secs = replayer.start_time.unwrap().elapsed().as_secs_f32() * replayer.speed;
    let elapsed = Duration::from_secs_f32(elapsed_secs);

    let mut cursor_node_id = replayer.cursor_node;

    // Process all events up to current elapsed time
    while replayer.current_index < replayer.events.len() {
        // Clone event to avoid borrowing replayer while mutating it later
        let event = replayer.events[replayer.current_index].clone();

        if event.timestamp > elapsed {
            break;
        }

        // --- Apply Event ---
        match event.kind {
            GhostEventKind::CursorMoved(pos) => {
                // Ensure cursor exists
                if cursor_node_id.is_none() {
                    // Create cursor
                    let root = scene.root();
                    let node = SceneNode::new(NodeContent::Styled {
                        style: Box::new(
                            VisualStyle::new()
                                .solid_fill(Color::rgba(0.5, 0.0, 1.0, 0.5).as_vec4()),
                        ), // Purple ghost
                    });

                    let id = scene.add_node(root, node);
                    cursor_node_id = Some(id);
                    replayer.cursor_node = Some(id);

                    // Spawn entity to track it
                    commands.spawn((GhostCursor, SceneNodeRef(id)));
                }

                // Update position
                if let Some(node) = cursor_node_id.and_then(|id| scene.get_node_mut(id)) {
                    node.bounds = plat_core::Rect::new(pos.x - 10.0, pos.y - 10.0, 20.0, 20.0);
                }
            }
            GhostEventKind::Click(_) => {
                if let Some(NodeContent::Styled { style }) = cursor_node_id
                    .and_then(|id| scene.get_node_mut(id))
                    .map(|n| &mut n.content)
                {
                    // Flash Red
                    if !style.fills.is_empty() {
                        style.fills[0] = Paint::Solid(Color::rgba(1.0, 0.0, 0.0, 0.8).as_vec4());
                    }
                }
            }
            GhostEventKind::Release(_) => {
                if let Some(NodeContent::Styled { style }) = cursor_node_id
                    .and_then(|id| scene.get_node_mut(id))
                    .map(|n| &mut n.content)
                {
                    // Back to Purple
                    if !style.fills.is_empty() {
                        style.fills[0] = Paint::Solid(Color::rgba(0.5, 0.0, 1.0, 0.5).as_vec4());
                    }
                }
            }
        }

        replayer.current_index += 1;
    }

    // Check if finished
    if replayer.current_index >= replayer.events.len() {
        // Stop
        replayer.playing = false;
        replayer.start_time = None;
        replayer.cursor_node = None;

        // Cleanup visual cursor
        if let Some(id) = cursor_node_id {
            scene.remove_node(id);
        }

        // Cleanup ECS entities
        for (entity, node_ref) in cursor_query.iter() {
            // Double check if we missed any (though scene.remove_node handles children)
            if cursor_node_id == Some(node_ref.0) {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Initialize the ghost replay system.
pub fn init_ghost_replay(app: &mut crate::App) {
    app.world_mut().insert_resource(GhostRecorder::default());
    app.world_mut().insert_resource(GhostReplayer::default());
    app.add_update_system(update_ghost_replay);
}

#[cfg(test)]
mod tests {
    use super::*;

    use plat_core::{
        ElementState, Event, Modifiers, MouseButton, MouseInput, Point, WindowEvent, WindowId,
    };

    #[test]
    fn test_ghost_recorder_lifecycle() {
        let mut recorder = GhostRecorder::default();
        assert!(!recorder.recording);
        assert!(recorder.start_time.is_none());

        recorder.start();
        assert!(recorder.recording);
        assert!(recorder.start_time.is_some());
        assert!(recorder.events.is_empty());

        recorder.stop();
        assert!(!recorder.recording);
    }

    #[test]
    fn test_ghost_replayer_lifecycle() {
        let mut replayer = GhostReplayer::default();
        assert!(!replayer.playing);

        let events = vec![GhostEvent {
            timestamp: Duration::from_secs(1),
            kind: GhostEventKind::Click(MouseButton::Left),
        }];
        replayer.play(events);
        assert!(replayer.playing);
        assert_eq!(replayer.events.len(), 1);

        replayer.stop();
        assert!(!replayer.playing);
    }

    #[test]
    fn test_record_event_adds_to_recorder() {
        let mut world = World::new();
        world.insert_resource(GhostRecorder::default());

        // Start recording
        {
            let mut recorder = world.resource_mut::<GhostRecorder>();
            recorder.start();
        }

        // Simulate event
        let event = Event::Window {
            window_id: WindowId::default(),
            event: WindowEvent::MouseInput(MouseInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
                position: Point::default(),
                modifiers: Modifiers::default(),
            }),
        };

        record_event(&mut world, &event);

        // Verify event recorded
        let recorder = world.resource::<GhostRecorder>();
        assert_eq!(recorder.events.len(), 1);
        if let GhostEventKind::Click(btn) = recorder.events[0].kind {
            assert_eq!(btn, MouseButton::Left);
        } else {
            panic!("Expected Click event");
        }
    }
}
