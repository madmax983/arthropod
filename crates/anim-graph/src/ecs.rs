//! ECS integration for the animation system.
//!
//! Provides [`TimelineDriver`] and [`timeline_system`] for driving
//! [`Timeline<T>`](crate::timeline::Timeline) animations via bevy_ecs.

use crate::Animatable;
use crate::timeline::Timeline;
use bevy_ecs::prelude::*;
use flux_state::WriteSignal;
use std::time::Duration;

/// Resource to provide delta time to the animation system.
#[derive(Resource, Default)]
pub struct TimeResource {
    pub delta: Duration,
}

impl TimeResource {
    pub fn new(delta: Duration) -> Self {
        Self { delta }
    }

    pub fn set_delta(&mut self, delta: Duration) {
        self.delta = delta;
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }
}

/// ECS component that drives a [`Timeline`] and writes output to a reactive signal.
///
/// # Example
///
/// ```no_run
/// use anim_graph::ecs::TimelineDriver;
/// use anim_graph::timeline::Timeline;
/// use std::time::Duration;
/// # use flux_state::{Runtime, Signal};
///
/// # let runtime = Runtime::new();
/// # let signal = Signal::new(runtime, 0.0_f32);
/// # let (_, write) = signal.split();
/// let driver = TimelineDriver::new(
///     Timeline::tween(0.0_f32, 1.0, Duration::from_millis(300)),
///     write,
/// );
/// ```
#[derive(Component)]
pub struct TimelineDriver<T: Animatable + Send + Sync + 'static> {
    pub timeline: Timeline<T>,
    pub target: WriteSignal<T>,
}

impl<T: Animatable + Send + Sync + 'static> TimelineDriver<T> {
    /// Create a new timeline driver.
    pub fn new(timeline: Timeline<T>, target: WriteSignal<T>) -> Self {
        Self { timeline, target }
    }
}

/// System that ticks all active [`TimelineDriver`]s.
///
/// Each frame:
/// 1. Advances the timeline by delta time
/// 2. Writes the sampled value to the reactive signal
/// 3. Removes the driver when the timeline completes
pub fn timeline_system<T: Animatable + Send + Sync + 'static>(
    time: Res<TimeResource>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut TimelineDriver<T>)>,
) {
    let dt = time.delta().as_secs_f32();

    for (entity, mut driver) in query.iter_mut() {
        let sample = driver.timeline.tick(dt);
        driver.target.set(sample.value);

        if driver.timeline.is_completed() {
            commands.entity(entity).remove::<TimelineDriver<T>>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};

    #[test]
    fn timeline_driver_writes_to_signal() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);
        let (read, write) = signal.split();

        let mut world = World::new();
        world.insert_resource(TimeResource::new(Duration::from_millis(50)));
        world.spawn(TimelineDriver::new(
            Timeline::tween(0.0, 100.0, Duration::from_millis(80)),
            write,
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(timeline_system::<f32>);
        schedule.run(&mut world);

        let val = read.get_untracked();
        assert!(val > 0.0, "Signal should have been updated, got {val}");
    }

    #[test]
    fn timeline_driver_removed_on_completion() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);
        let (_, write) = signal.split();

        let mut world = World::new();
        world.insert_resource(TimeResource::new(Duration::from_millis(100)));
        let entity = world
            .spawn(TimelineDriver::new(
                Timeline::tween(0.0, 100.0, Duration::from_millis(50)),
                write,
            ))
            .id();

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(timeline_system::<f32>);
        schedule.run(&mut world);

        // Apply deferred commands
        world.flush();

        assert!(
            world.get::<TimelineDriver<f32>>(entity).is_none(),
            "Driver should be removed after completion"
        );
    }

    #[test]
    fn timeline_driver_loops_not_removed() {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, 0.0_f32);
        let (_, write) = signal.split();

        let mut world = World::new();
        world.insert_resource(TimeResource::new(Duration::from_millis(100)));
        let entity = world
            .spawn(TimelineDriver::new(
                Timeline::tween(0.0, 100.0, Duration::from_millis(50)).loop_forever(),
                write,
            ))
            .id();

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(timeline_system::<f32>);
        schedule.run(&mut world);

        world.flush();

        assert!(
            world.get::<TimelineDriver<f32>>(entity).is_some(),
            "Looping driver should NOT be removed"
        );
    }
}
