#[cfg(feature = "nova")]
use anim_graph::{Animatable, Animation};
#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use flux_state::{ReadSignal, Runtime, Signal, WriteSignal};
#[cfg(feature = "nova")]
use std::sync::{Arc, Mutex, Weak};
#[cfg(feature = "nova")]
use std::time::{Duration, Instant};

#[cfg(feature = "nova")]
/// Internal state for an elastic signal.
struct ElasticState<T: Animatable> {
    animation: Animation<T>,
    target_signal: WriteSignal<T>,
}

#[cfg(feature = "nova")]
/// A handle to an elastic signal that can be ticked.
trait Tickable: Send + Sync {
    /// Update the animation state by dt. Returns true if active, false if can be dropped.
    fn tick(&self, dt: Duration) -> bool;
}

#[cfg(feature = "nova")]
impl<T: Animatable + Send + Sync + 'static> Tickable for Mutex<ElasticState<T>> {
    fn tick(&self, dt: Duration) -> bool {
        let (new_value, target_signal) = {
            let mut guard = match self.lock() {
                Ok(g) => g,
                Err(_) => return false, // Poisoned
            };

            // Tick the animation
            let new_value = guard.animation.tick(dt);
            (new_value, guard.target_signal.clone())
        };

        // Update the signal (this triggers effects)
        // We do this outside the lock to avoid deadlocks if subscribers call back into us
        target_signal.set(new_value);

        // Keep alive as long as we exist (Weak ref controls lifetime in registry)
        true
    }
}

#[cfg(feature = "nova")]
/// A reactive signal that animates towards its target value using spring physics.
///
/// Wraps a `flux_state::Signal` and an `anim_graph::Animation`.
#[derive(Clone)]
pub struct ElasticSignal<T: Animatable + Send + Sync + 'static> {
    read: ReadSignal<T>,
    #[allow(dead_code)]
    write: WriteSignal<T>,
    state: Arc<Mutex<ElasticState<T>>>,
}

#[cfg(feature = "nova")]
impl<T: Animatable + Send + Sync + 'static> ElasticSignal<T> {
    /// Create a new elastic signal with an initial value.
    ///
    /// The signal will start at `initial` and will animate towards any new value set via `.set()`.
    ///
    /// # Arguments
    ///
    /// * `runtime` - The reactive runtime.
    /// * `initial` - The initial value.
    /// * `registry` - The global registry to register this signal for ticking.
    pub fn new(runtime: Arc<Runtime>, initial: T, registry: &ElasticRegistry) -> Self {
        let (read, write) = Signal::new(runtime, initial.clone()).split();

        let state = Arc::new(Mutex::new(ElasticState {
            animation: Animation::Spring {
                current: initial.clone(),
                target: initial,
                velocity: T::zero(),
                stiffness: 200.0,
                damping: 20.0,
            },
            target_signal: write.clone(),
        }));

        let signal = Self { read, write, state };

        // Register immediately
        registry.register(&signal);

        signal
    }

    /// Set a new target value for the signal.
    ///
    /// The signal will animate towards this value using spring physics.
    pub fn set(&self, target: T) {
        if let Ok(mut state) = self.state.lock() {
            if let Animation::Spring {
                target: anim_target,
                ..
            } = &mut state.animation
            {
                *anim_target = target;
            } else if let Animation::Tween {
                to: anim_to,
                elapsed: anim_elapsed,
                ..
            } = &mut state.animation
            {
                // If we were tweening, reset to new target (implementation detail: we only support spring for now)
                *anim_to = target;
                *anim_elapsed = Duration::ZERO;
            }
        }
    }

    /// Get the current interpolated value of the signal.
    pub fn get(&self) -> T {
        self.read.get()
    }

    /// Get the underlying ReadSignal for use in computations/effects.
    pub fn read_signal(&self) -> ReadSignal<T> {
        self.read.clone()
    }
}

#[cfg(feature = "nova")]
/// Resource to manage active elastic signals.
#[derive(Resource, Default, Clone)]
pub struct ElasticRegistry {
    signals: Arc<Mutex<Vec<Weak<dyn Tickable>>>>,
}

#[cfg(feature = "nova")]
impl ElasticRegistry {
    pub fn register<T: Animatable + Send + Sync + 'static>(&self, signal: &ElasticSignal<T>) {
        if let Ok(mut signals) = self.signals.lock() {
            // Coerce Arc<Mutex<ElasticState<T>>> to Weak<dyn Tickable>
            // Use Arc::downgrade and type erasure
            let weak = Arc::downgrade(&signal.state) as Weak<dyn Tickable>;
            signals.push(weak);
        }
    }
}

#[cfg(feature = "nova")]
/// Resource to track time for elastic animations.
#[derive(Resource)]
pub struct ElasticTime {
    last_update: Instant,
    pub delta: Duration,
}

#[cfg(feature = "nova")]
impl Default for ElasticTime {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
            delta: Duration::from_millis(16), // Default to 60fps if not updated
        }
    }
}

#[cfg(feature = "nova")]
/// System to update the ElasticTime resource.
pub fn update_elastic_time_system(mut time: ResMut<ElasticTime>) {
    let now = Instant::now();
    let delta = now.duration_since(time.last_update);
    // Clamp delta to avoid huge jumps if thread sleeps (max 100ms)
    time.delta = delta.min(Duration::from_millis(100));
    time.last_update = now;
}

#[cfg(feature = "nova")]
/// System to tick all registered elastic signals.
pub fn elastic_tick_system(registry: Res<ElasticRegistry>, time: Res<ElasticTime>) {
    if let Ok(mut signals) = registry.signals.lock() {
        let dt = time.delta;

        // Retain only signals that are still alive (returns true)
        signals.retain(|weak| {
            if let Some(arc) = weak.upgrade() {
                arc.tick(dt)
            } else {
                false // Dropped signal, remove from registry
            }
        });
    }
}

#[cfg(feature = "nova")]
/// Helper to register elastic systems and resources to an App.
///
/// Usage:
/// ```
/// use arthropod::prelude::*;
/// use arthropod::experimental::elastic::register_elastic_feature;
///
/// let mut app = AppBuilder::new().build_headless().unwrap();
/// register_elastic_feature(app.world_mut());
/// app.add_update_system(arthropod::experimental::elastic::update_elastic_time_system);
/// app.add_update_system(arthropod::experimental::elastic::elastic_tick_system);
/// ```
pub fn register_elastic_feature(world: &mut World) {
    if world.get_resource::<ElasticRegistry>().is_none() {
        world.insert_resource(ElasticRegistry::default());
    }
    if world.get_resource::<ElasticTime>().is_none() {
        world.insert_resource(ElasticTime::default());
    }
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;

    #[test]
    fn test_elastic_signal_interpolation() {
        let runtime = Runtime::new();
        let registry = ElasticRegistry::default();

        // Initial value 0.0
        let signal = ElasticSignal::new(runtime.clone(), 0.0, &registry);

        // Set target to 100.0
        signal.set(100.0);

        // Verify initial state
        assert_eq!(signal.get(), 0.0);

        // Create a time resource
        let time = ElasticTime {
            last_update: Instant::now(),
            delta: Duration::from_millis(16),
        };

        // Tick manually via system logic
        // (In a real app, systems run via ECS)
        {
            let signals = registry.signals.lock().unwrap();
            for weak in signals.iter() {
                if let Some(arc) = weak.upgrade() {
                    arc.tick(time.delta);
                }
            }
        }

        // Value should have moved from 0.0 towards 100.0
        let val1 = signal.get();
        assert!(val1 > 0.0);
        assert!(val1 < 100.0);

        // Tick again
        {
            let signals = registry.signals.lock().unwrap();
            for weak in signals.iter() {
                if let Some(arc) = weak.upgrade() {
                    arc.tick(time.delta);
                }
            }
        }

        let val2 = signal.get();
        assert!(val2 > val1);
        assert!(val2 < 100.0);
    }

    #[test]
    fn test_elastic_registry_cleanup() {
        let runtime = Runtime::new();
        let registry = ElasticRegistry::default();

        {
            let _signal = ElasticSignal::new(runtime.clone(), 0.0, &registry);
            assert_eq!(registry.signals.lock().unwrap().len(), 1);
        }
        // _signal is dropped here

        // Run cleanup logic (retain)
        {
            let mut signals = registry.signals.lock().unwrap();
            signals.retain(|weak| weak.upgrade().is_some());
        }

        assert_eq!(registry.signals.lock().unwrap().len(), 0);
    }
}
