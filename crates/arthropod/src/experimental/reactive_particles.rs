use super::particles::ParticleEmitter;
use bevy_ecs::prelude::*;
use flux_state::ReadSignal;
use render_engine::{Color, Vec2};

/// Component to drive particle emitter properties from reactive signals.
///
/// This component bridges `flux-state` reactivity with `arthropod-ecs` particle systems.
/// By attaching this component to an entity with a `ParticleEmitter`, you can bind
/// particle properties (rate, color, etc.) to reactive signals.
///
/// # Example
///
/// ```
/// # use bevy_ecs::prelude::*;
/// # use arthropod::experimental::reactive_particles::ReactiveParticleEmitter;
/// # use flux_state::{Runtime, Signal};
/// # use render_engine::Color;
/// # let runtime = Runtime::new();
/// let color_signal = Signal::new(runtime.clone(), Color::RED);
/// let (read_color, write_color) = color_signal.split();
///
/// // Entity will emit RED particles, then BLUE when signal updates
/// let reactive = ReactiveParticleEmitter {
///     color: Some(read_color),
///     ..Default::default()
/// };
/// ```
#[derive(Component, Default, Clone)]
pub struct ReactiveParticleEmitter {
    /// Bind emission rate (particles/sec) to a signal.
    pub rate: Option<ReadSignal<f32>>,
    /// Bind particle color to a signal.
    pub color: Option<ReadSignal<Color>>,
    /// Bind particle size to a signal.
    pub size: Option<ReadSignal<f32>>,
    /// Bind velocity spread to a signal.
    pub spread: Option<ReadSignal<Vec2>>,
    /// Bind active state to a signal.
    pub active: Option<ReadSignal<bool>>,
}

/// System to sync reactive signals to particle emitters.
///
/// This runs every frame and updates the `ParticleEmitter` component
/// if the corresponding signal exists in `ReactiveParticleEmitter`.
pub fn sync_reactive_emitters(
    mut emitters: Query<(&mut ParticleEmitter, &ReactiveParticleEmitter)>,
) {
    for (mut emitter, reactive) in &mut emitters {
        if let Some(rate) = &reactive.rate {
            emitter.rate = rate.get();
        }
        if let Some(color) = &reactive.color {
            emitter.color = color.get();
        }
        if let Some(size) = &reactive.size {
            emitter.size = size.get();
        }
        if let Some(spread) = &reactive.spread {
            emitter.spread = spread.get();
        }
        if let Some(active) = &reactive.active {
            emitter.active = active.get();
        }
    }
}

/// Registers the reactive particle system.
pub fn register_reactive_particles(app: &mut crate::App) {
    app.add_update_system(sync_reactive_emitters);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::particles::ParticleEmitter;
    use flux_state::{Runtime, Signal};

    #[test]
    fn test_reactive_emitter_sync() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(sync_reactive_emitters);

        let runtime = Runtime::new();

        // Create signals
        let (read_rate, write_rate) = Signal::new(runtime.clone(), 10.0).split();
        let (read_active, write_active) = Signal::new(runtime.clone(), true).split();

        // Spawn entity with Emitter + Reactive
        let entity = world
            .spawn((
                ParticleEmitter {
                    rate: 0.0, // Initial static value (should be overwritten)
                    active: false,
                    ..Default::default()
                },
                ReactiveParticleEmitter {
                    rate: Some(read_rate),
                    active: Some(read_active),
                    ..Default::default()
                },
            ))
            .id();

        // Run system
        schedule.run(&mut world);

        // Verify sync
        let emitter = world.entity(entity).get::<ParticleEmitter>().unwrap();
        assert_eq!(emitter.rate, 10.0);
        assert_eq!(emitter.active, true);

        // Update signals
        write_rate.set(50.0);
        write_active.set(false);

        // Run system again
        schedule.run(&mut world);

        // Verify update
        let emitter = world.entity(entity).get::<ParticleEmitter>().unwrap();
        assert_eq!(emitter.rate, 50.0);
        assert_eq!(emitter.active, false);
    }
}
