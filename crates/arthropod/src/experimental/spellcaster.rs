//! Spellcaster: Gesture-Based Action Dispatcher
//!
//! "Spellcaster" is an experimental input system that maps complex mouse gestures
//! (from `input-engine::StrokeMatcher`) to arbitrary actions (Signals).
//!
//! It enables "casting spells" by drawing shapes on the screen.
//!
//! # Concepts
//!
//! - **Spell**: A mapping from a `MouseGesture` (e.g. Circle, Swipe) to an effect.
//! - **Grimoire**: A registry of known spells.
//! - **Wand**: The component that tracks input and casts spells.

use crate::experimental::particles::ParticleEmitter;
use bevy_ecs::prelude::*;
use input_engine::{InputPattern, MouseGesture, StrokeMatcher};
use plat_core::{MouseButton, WindowEvent};
use render_engine::Vec2;
use std::collections::HashMap;
use std::sync::Arc;

/// A registry of spells (gesture -> callback).
///
/// We use `Arc<dyn Fn() + Send + Sync>` for callbacks to allow arbitrary effects.
#[derive(Resource, Default, Clone)]
pub struct Grimoire {
    spells: HashMap<MouseGesture, Arc<dyn Fn() + Send + Sync>>,
}

impl Grimoire {
    /// Learn a new spell.
    ///
    /// # Example
    ///
    /// ```
    /// # use arthropod::experimental::spellcaster::Grimoire;
    /// # use input_engine::MouseGesture;
    /// # use std::sync::Arc;
    /// let mut grimoire = Grimoire::default();
    /// grimoire.learn(MouseGesture::CircleClockwise, || println!("Fireball!"));
    /// ```
    pub fn learn<F>(&mut self, gesture: MouseGesture, effect: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.spells.insert(gesture, Arc::new(effect));
    }

    /// Cast a spell matching the gesture.
    pub fn cast(&self, gesture: MouseGesture) {
        if let Some(effect) = self.spells.get(&gesture) {
            effect();
        }
    }
}

/// Component for the "Wand" - the entity that tracks gestures.
#[derive(Component)]
pub struct Wand {
    matcher: StrokeMatcher,
    pub active: bool,
    /// Optional visual feedback (particle emitter entity)
    pub emitter_entity: Option<Entity>,
}

impl Default for Wand {
    fn default() -> Self {
        Self {
            matcher: StrokeMatcher::new(MouseButton::Right),
            active: true,
            emitter_entity: None,
        }
    }
}

/// Resource to receive window events for the wand.
///
/// In a real integration, this would come from `plat-core` or `input-engine`.
/// For now, we inject it manually in systems or tests.
#[derive(Resource, Default)]
pub struct WandInput {
    pub events: Vec<WindowEvent>,
}

/// System to update wands and cast spells.
pub fn wand_system(
    mut wands: Query<&mut Wand>,
    mut input: ResMut<WandInput>,
    grimoire: Res<Grimoire>,
    // Optional: Update particle emitter position if tracking
    mut emitters: Query<&mut ParticleEmitter>,
) {
    for event in input.events.drain(..) {
        for mut wand in &mut wands {
            if !wand.active {
                continue;
            }

            if let Some(gesture) = wand.matcher.update(&event) {
                // Gesture recognized! Cast spell.
                grimoire.cast(gesture);
            }

            // Visual feedback: Update emitter position to follow cursor
            if let Some(emitter_entity) = wand.emitter_entity {
                // Clippy suggestion: collapse nested if block
                #[allow(clippy::collapsible_if)]
                if let Ok(mut emitter) = emitters.get_mut(emitter_entity) {
                    if let WindowEvent::CursorMoved { position } = event {
                        emitter.position = Vec2::new(position.x as f32, position.y as f32);
                    }
                    // Only emit while tracking (button down)
                    emitter.active = wand.matcher.is_tracking();
                }
            }
        }
    }
}

/// Helper to register Spellcaster features.
pub fn register_spellcaster(app: &mut crate::App) {
    app.world_mut().init_resource::<Grimoire>();
    app.world_mut().init_resource::<WandInput>();
    app.add_update_system(wand_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use input_engine::MouseGesture;
    use plat_core::{ElementState, Modifiers, MouseInput, Point};
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_spell_casting() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(wand_system);

        let mut grimoire = Grimoire::default();
        let cast_success = Arc::new(AtomicBool::new(false));
        let cast_success_clone = cast_success.clone();

        grimoire.learn(MouseGesture::SwipeRight, move || {
            cast_success_clone.store(true, Ordering::SeqCst);
        });

        world.insert_resource(grimoire);
        world.insert_resource(WandInput::default());

        // Spawn a Wand
        world.spawn(Wand::default());

        // Simulate Swipe Right
        let points = [
            Point::new(0.0, 0.0),
            Point::new(20.0, 0.0),
            Point::new(40.0, 0.0),
            Point::new(60.0, 0.0),
            Point::new(80.0, 0.0),
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
        // Release
        events.push(WindowEvent::MouseInput(MouseInput {
            state: ElementState::Released,
            button: MouseButton::Right,
            position: *points.last().unwrap(),
            modifiers: Modifiers::default(),
        }));

        // Inject events
        world.resource_mut::<WandInput>().events = events;

        // Run system
        schedule.run(&mut world);

        // Verify spell was cast
        assert!(cast_success.load(Ordering::SeqCst));
    }
}
