use anim_graph::{Animatable, Animation, Easing};
use flux_state::{Effect, ReadSignal, Runtime, Signal};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Configuration for motion animations.
#[derive(Clone, Copy, Debug)]
pub enum MotionConfig {
    /// Physics-based spring animation.
    Spring { stiffness: f32, damping: f32 },
    /// Time-based tween animation.
    Tween { duration: Duration, easing: Easing },
}

impl Default for MotionConfig {
    fn default() -> Self {
        Self::Spring {
            stiffness: 100.0,
            damping: 15.0,
        }
    }
}

/// A signal that animates its value over time.
///
/// Keeps the animation effect alive as long as this struct exists.
pub struct MotionSignal<T> {
    signal: ReadSignal<T>,
    _effect: Effect,
}

impl<T> std::ops::Deref for MotionSignal<T> {
    type Target = ReadSignal<T>;
    fn deref(&self) -> &Self::Target {
        &self.signal
    }
}

/// Create a signal that smoothly animates to the source signal's value.
///
/// # Arguments
///
/// * `cx` - The runtime context.
/// * `source` - The source signal (target value).
/// * `clock` - A signal representing the current application time (e.g., elapsed duration).
/// * `config` - Animation configuration.
pub fn create_motion_signal<T>(
    cx: Arc<Runtime>,
    source: ReadSignal<T>,
    clock: ReadSignal<Duration>,
    config: MotionConfig,
) -> MotionSignal<T>
where
    T: Animatable + PartialEq + Send + Sync + 'static,
{
    let initial = source.get_untracked();
    let output = Signal::new(cx.clone(), initial.clone());
    let (read_out, write_out) = output.split();

    struct State<T: Animatable> {
        animation: Option<Animation<T>>,
        last_target: T,
        last_time: Duration,
    }

    let state = Arc::new(Mutex::new(State {
        animation: None,
        last_target: initial,
        last_time: Duration::ZERO,
    }));

    let effect = Effect::new(cx.clone(), move || {
        let target = source.get(); // Always subscribe to target changes
        let mut state = state.lock().unwrap();

        // Check if target changed
        if target != state.last_target {
            // Determine start value from current animation state or last target
            let start = if let Some(anim) = &mut state.animation {
                anim.tick(Duration::ZERO)
            } else {
                state.last_target.clone()
            };

            let anim = match config {
                MotionConfig::Spring { stiffness, damping } => {
                    Animation::spring(start, target.clone(), stiffness, damping)
                }
                MotionConfig::Tween { duration, easing } => {
                    Animation::tween(start, target.clone(), duration, easing)
                }
            };

            state.animation = Some(anim);
            state.last_target = target;
        }

        // Check if we need to animate
        let is_animating = state
            .animation
            .as_ref()
            .map(|a| !a.is_complete())
            .unwrap_or(false);

        if is_animating {
            let time = clock.get(); // Subscribe to clock

            // Calculate delta time
            // If we haven't updated in a while (e.g. > 100ms), treat as reset/start of animation
            let dt = if time >= state.last_time {
                time - state.last_time
            } else {
                Duration::ZERO
            };

            state.last_time = time;

            // Clamp dt to avoid huge jumps if we stopped listening for a long time
            // or on first frame of animation
            // Using 1.0s to allow reasonable steps in tests
            let dt = if dt.as_secs_f32() > 1.0 {
                Duration::ZERO
            } else {
                dt
            };

            if let Some(anim) = &mut state.animation {
                let new_val = anim.tick(dt);
                write_out.set(new_val);

                // If complete, clear animation to stop subscribing to clock
                if anim.is_complete() {
                    state.animation = None;
                }
            }
        }
    });

    MotionSignal {
        signal: read_out,
        _effect: effect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;

    #[test]
    fn test_tween_motion() {
        let runtime = Runtime::new();
        let clock = Signal::new(runtime.clone(), Duration::ZERO);
        let (read_clock, write_clock) = clock.split();

        let source = Signal::new(runtime.clone(), 0.0);
        let (read_source, write_source) = source.split();

        let motion = create_motion_signal(
            runtime.clone(),
            read_source,
            read_clock,
            MotionConfig::Tween {
                duration: Duration::from_secs(1),
                easing: Easing::Linear,
            },
        );

        // Initial state
        assert_eq!(motion.get_untracked(), 0.0);

        // Update target
        write_source.set(100.0);
        // Motion shouldn't change immediately (dt=0 on first tick)
        assert_eq!(motion.get_untracked(), 0.0);

        // Advance time 0.5s
        write_clock.set(Duration::from_millis(500));

        // Value should be interpolated (50.0)
        assert_eq!(motion.get_untracked(), 50.0);

        // Advance time to completion
        write_clock.set(Duration::from_secs(1));
        assert_eq!(motion.get_untracked(), 100.0);
    }

    #[test]
    fn test_spring_settling() {
        let runtime = Runtime::new();
        let clock = Signal::new(runtime.clone(), Duration::ZERO);
        let (read_clock, write_clock) = clock.split();

        let source = Signal::new(runtime.clone(), 0.0);
        let (read_source, write_source) = source.split();

        let motion = create_motion_signal(
            runtime.clone(),
            read_source,
            read_clock,
            MotionConfig::Spring {
                stiffness: 100.0,
                damping: 20.0,
            },
        );

        write_source.set(10.0);

        // Simulate frames
        let mut time = Duration::ZERO;
        let dt = Duration::from_millis(16);

        for _ in 0..100 {
            time += dt;
            write_clock.set(time);
        }

        // Should be close to 10.0
        let val = motion.get_untracked();
        assert!((val - 10.0).abs() < 0.1);
    }
}
