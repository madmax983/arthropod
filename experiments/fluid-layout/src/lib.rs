//! Fluid Layout - Physics-based layout interpolation
//!
//! This crate provides a reactive primitive `create_fluid_layout_signal` that transforms
//! discrete layout updates (from `layout-engine`) into smooth, continuous animations.
//!
//! It bridges `flux-state` (reactivity), `layout-engine` (data), and `anim-graph` (physics).

use anim_graph::{Animatable, Animation, Easing};
use flux_state::{Effect, ReadSignal, Runtime, Signal};
use glam::Vec4;
use layout_engine::ComputedLayout;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Configuration for fluid layout animations
#[derive(Clone, Copy, Debug)]
pub enum FluidLayoutConfig {
    /// Physics-based spring animation.
    /// Good for interactive UI that needs to feel organic.
    Spring {
        /// Controls how strongly the layout snaps to the target.
        stiffness: f32,
        /// Controls how quickly the layout settles.
        damping: f32,
    },
    /// Time-based tween animation.
    /// Good for predictable transitions.
    Tween {
        /// The total time the layout transition should take.
        duration: Duration,
        /// The curve controlling the rate of the transition.
        easing: Easing,
    },
}

impl Default for FluidLayoutConfig {
    fn default() -> Self {
        Self::Spring {
            stiffness: 150.0,
            damping: 20.0,
        }
    }
}

/// A layout rectangle that can be animated (Animatable)
///
/// Wraps x, y, width, height in a Vec4 for SIMD operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FluidRect(pub Vec4);

impl FluidRect {
    /// Create a new FluidRect from raw coordinates and dimensions.
    ///
    /// ## Examples
    ///
    /// ```
    /// use fluid_layout::FluidRect;
    ///
    /// let rect = FluidRect::new(10.0, 20.0, 100.0, 50.0);
    /// assert_eq!(rect.x(), 10.0);
    /// assert_eq!(rect.width(), 100.0);
    /// ```
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self(Vec4::new(x, y, w, h))
    }

    /// Extracts the horizontal position from the packed SIMD vector.
    pub fn x(&self) -> f32 {
        self.0.x
    }

    /// Extracts the vertical position from the packed SIMD vector.
    pub fn y(&self) -> f32 {
        self.0.y
    }

    /// Extracts the layout width from the packed SIMD vector.
    pub fn width(&self) -> f32 {
        self.0.z
    }

    /// Extracts the layout height from the packed SIMD vector.
    pub fn height(&self) -> f32 {
        self.0.w
    }
}

impl From<ComputedLayout> for FluidRect {
    fn from(layout: ComputedLayout) -> Self {
        Self::new(layout.x, layout.y, layout.width, layout.height)
    }
}

impl From<FluidRect> for ComputedLayout {
    fn from(rect: FluidRect) -> Self {
        ComputedLayout {
            x: rect.x(),
            y: rect.y(),
            width: rect.width(),
            height: rect.height(),
        }
    }
}

impl Animatable for FluidRect {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self(self.0.lerp(other.0, t))
    }

    fn scale(&self, scalar: f32) -> Self {
        Self(self.0 * scalar)
    }

    fn add(&self, other: &Self) -> Self {
        Self(self.0 + other.0)
    }

    fn sub(&self, other: &Self) -> Self {
        Self(self.0 - other.0)
    }

    fn zero() -> Self {
        Self(Vec4::ZERO)
    }

    fn distance_squared(&self, other: &Self) -> f32 {
        self.0.distance_squared(other.0)
    }
}

/// A signal that holds the fluid layout animation state.
///
/// Wraps the underlying signal and keeps the animation effect alive.
pub struct FluidLayoutSignal {
    signal: ReadSignal<ComputedLayout>,
    _effect: Effect,
}

impl std::ops::Deref for FluidLayoutSignal {
    type Target = ReadSignal<ComputedLayout>;
    fn deref(&self) -> &Self::Target {
        &self.signal
    }
}

/// Create a signal that smoothly animates discrete layout changes.
///
/// This function creates a reactive bridge between a source of layout updates
/// (e.g. from `LayoutEngine`) and a consumer (e.g. `RenderEngine`).
///
/// # Arguments
///
/// * `cx` - The runtime context.
/// * `source` - The source signal providing `ComputedLayout` updates.
/// * `clock` - A signal providing the current application time (as `Duration`).
/// * `config` - Animation configuration (Spring or Tween).
pub fn create_fluid_layout_signal(
    cx: Arc<Runtime>,
    source: ReadSignal<ComputedLayout>,
    clock: ReadSignal<Duration>,
    config: FluidLayoutConfig,
) -> FluidLayoutSignal {
    let initial: FluidRect = source.get_untracked().into();

    // We output ComputedLayout directly for convenience
    let output = Signal::new(cx.clone(), ComputedLayout::from(initial));
    let (read_out, write_out) = output.split();

    struct State {
        animation: Option<Animation<FluidRect>>,
        last_target: FluidRect,
        last_time: Duration,
    }

    let state = Arc::new(Mutex::new(State {
        animation: None,
        last_target: initial,
        last_time: Duration::ZERO,
    }));

    // Create an effect that runs when source changes OR clock ticks
    let effect = Effect::new(cx, move || {
        let target_layout = source.get(); // Subscribe to source
        let target: FluidRect = target_layout.into();

        let mut state = state.lock().unwrap();

        // Check for target change
        if target != state.last_target {
            // Determine start value from current animation state or last target
            let start = if let Some(anim) = &mut state.animation {
                // If animating, start from current interpolated value
                anim.tick(Duration::ZERO)
            } else {
                state.last_target
            };

            let anim = match config {
                FluidLayoutConfig::Spring { stiffness, damping } => {
                    Animation::spring(start, target, stiffness, damping)
                }
                FluidLayoutConfig::Tween { duration, easing } => {
                    Animation::tween(start, target, duration, easing)
                }
            };

            state.animation = Some(anim);
            state.last_target = target;
        }

        // Check if animating
        let is_animating = state
            .animation
            .as_ref()
            .map(|a| !a.is_complete())
            .unwrap_or(false);

        if is_animating {
            let time = clock.get(); // Subscribe to clock

            // Calculate dt
            let dt = if time >= state.last_time {
                time - state.last_time
            } else {
                Duration::ZERO
            };

            state.last_time = time;

            // Clamp huge dt (e.g. initial start or long pause)
            // Using 0.1s (100ms) as threshold for "huge jump"
            let dt = if dt.as_secs_f32() > 0.1 {
                Duration::ZERO
            } else {
                dt
            };

            if let Some(anim) = &mut state.animation {
                let new_rect = anim.tick(dt);
                write_out.set(new_rect.into());

                if anim.is_complete() {
                    state.animation = None;
                }
            }
        } else {
            // Update last_time even if not animating, so we don't have a huge dt when we start
            let time = clock.get_untracked();
            state.last_time = time;
        }
    });

    FluidLayoutSignal {
        signal: read_out,
        _effect: effect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;

    #[test]
    fn test_layout_tween() {
        let runtime = Runtime::new();
        let clock = Signal::new(runtime.clone(), Duration::ZERO);
        let (read_clock, write_clock) = clock.split();

        let initial_layout = ComputedLayout {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let source = Signal::new(runtime.clone(), initial_layout);
        let (read_source, write_source) = source.split();

        let fluid = create_fluid_layout_signal(
            runtime.clone(),
            read_source,
            read_clock,
            FluidLayoutConfig::Tween {
                duration: Duration::from_secs(1),
                easing: Easing::Linear,
            },
        );

        // Initial check
        let current = fluid.get_untracked();
        assert_eq!(current.x, 0.0);

        // Change target
        let target_layout = ComputedLayout {
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 200.0,
        };
        write_source.set(target_layout);

        // Advance time by 0.5s in small steps
        // The signal logic clamps large dt (>100ms) to 0, so we must simulate realistic frames
        let mut time = Duration::ZERO;
        for _ in 0..50 {
            time += Duration::from_millis(10);
            write_clock.set(time);
        }

        // Should be halfway
        let current = fluid.get_untracked();
        assert!((current.x - 50.0).abs() < 0.1);
        assert!((current.width - 150.0).abs() < 0.1);

        // Advance to completion
        for _ in 0..50 {
            time += Duration::from_millis(10);
            write_clock.set(time);
        }

        let current = fluid.get_untracked();
        assert!((current.x - 100.0).abs() < 0.1);
    }
}
