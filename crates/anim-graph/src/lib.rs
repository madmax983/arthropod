//! Animation system for Arthropod GUI framework.
//!
//! The `anim-graph` crate provides a physics-based animation system designed for UI interactions.
//! It supports both traditional tweening (time-based) and spring physics (force-based) animations.
//!
//! # Key Concepts
//!
//! - **[`Animation`]:** The core enum representing an active animation state (Tween or Spring).
//! - **[`Animatable`]:** A trait implemented by types that can be animated (e.g., `f32`, `Color`).
//! - **[`Easing`]:** Functions that control the rate of change for tweens.
//!
//! # Examples
//!
//! ## Simple Tween
//!
//! Create a linear interpolation between two values over 1 second:
//!
//! ```
//! use anim_graph::{Animation, Easing};
//! use std::time::Duration;
//!
//! let mut anim = Animation::tween(0.0, 100.0, Duration::from_secs(1), Easing::Linear);
//!
//! // Advance time by 0.5 seconds
//! let value = anim.tick(Duration::from_millis(500));
//! assert_eq!(value, 50.0);
//! ```
//!
//! ## Physics Spring
//!
//! Create a spring animation that naturally settles at a target:
//!
//! ```
//! use anim_graph::Animation;
//! use std::time::Duration;
//!
//! // Create a stiff spring with low damping (bouncy)
//! let mut spring = Animation::spring(0.0, 100.0, 300.0, 15.0);
//!
// Ticking the simulation
//! let value = spring.tick(Duration::from_millis(16));
//! ```

pub mod clock;
pub mod evaluable;
pub mod hold;
pub mod keyframe;
pub mod sample;
pub mod sequence;
pub mod spring_segment;
pub mod stagger;
pub mod time_warp;
pub mod timeline;

// Re-exports
pub use clock::{AnimationClock, ClockEvent, PlaybackMode};
pub use evaluable::Evaluable;
pub use sample::Sample;

use std::time::Duration;

/// Trait for types that can be animated.
///
/// This trait extends `Clone` and requires implementation of interpolation
/// and basic vector arithmetic operations needed for physics simulations.
pub trait Animatable: Clone + 'static {
    /// Interpolate between two values.
    ///
    /// `t` is a value between 0.0 and 1.0 (clamped).
    fn interpolate(&self, other: &Self, t: f32) -> Self;

    /// Scale the value by a scalar factor.
    fn scale(&self, scalar: f32) -> Self;

    /// Add another value to this one.
    fn add(&self, other: &Self) -> Self;

    /// Subtract another value from this one.
    fn sub(&self, other: &Self) -> Self;

    /// Return the zero value (additive identity).
    ///
    /// For `f32`, this is `0.0`. For `Color`, this is a transparent zero vector.
    fn zero() -> Self;

    /// Calculate the squared distance between two values.
    /// Used for determining when an animation has settled.
    fn distance_squared(&self, other: &Self) -> f32;
}

/// Easing function for animations.
///
/// Easing functions specify the rate of change of a parameter over time.
///
/// # Visualizations
///
/// - **Linear:** Constant speed. `f(t) = t`
/// - **EaseIn:** Starts slow, speeds up. `f(t) = t²`
/// - **EaseOut:** Starts fast, slows down. `f(t) = t(2-t)`
/// - **EaseInOut:** Slow start and end. `f(t) = 2t²` if t<0.5 else `...`
#[derive(Debug, Clone, Copy)]
pub enum Easing {
    /// Linear interpolation (no easing).
    Linear,
    /// Quadratic ease-in (slow start).
    EaseIn,
    /// Quadratic ease-out (slow end).
    EaseOut,
    /// Quadratic ease-in-out (slow start and end).
    EaseInOut,
    /// Custom cubic bezier curve defined by two control points (x1, y1) and (x2, y2).
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    /// Apply the easing function to a linear parameter t (0.0 to 1.0).
    ///
    /// Returns the eased value, which is usually between 0.0 and 1.0,
    /// but may overshoot for elastic functions (not yet implemented).
    ///
    /// # Example
    ///
    /// ```
    /// use anim_graph::Easing;
    ///
    /// let linear = Easing::Linear;
    /// assert_eq!(linear.apply(0.5), 0.5);
    ///
    /// let ease_in = Easing::EaseIn;
    /// assert_eq!(ease_in.apply(0.5), 0.25); // 0.5 * 0.5
    /// ```
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Easing::CubicBezier(_x1, y1, _x2, y2) => {
                // Simplified cubic bezier - full implementation would use iterative solver
                let t2 = t * t;
                let t3 = t2 * t;
                3.0 * (1.0 - t) * (1.0 - t) * t * y1 + 3.0 * (1.0 - t) * t2 * y2 + t3
            }
        }
    }

    /// Compute the instantaneous rate of change (derivative) of the easing function.
    ///
    /// Used by [`Keyframe`](crate::keyframe::Keyframe) to produce velocity in [`Sample`].
    /// The derivative tells us how fast the easing curve is changing at time `t`,
    /// which translates to the animation's velocity when scaled by `(to - from) / duration`.
    ///
    /// # Mathematical basis
    ///
    /// - **Linear**: `f(t) = t` → `f'(t) = 1`
    /// - **EaseIn**: `f(t) = t²` → `f'(t) = 2t`
    /// - **EaseOut**: `f(t) = t(2-t)` → `f'(t) = 2 - 2t`
    /// - **EaseInOut**: piecewise `f'(t) = 4t` or `f'(t) = 4 - 4t`
    /// - **CubicBezier**: numerical finite difference
    pub fn derivative(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => 1.0,
            Easing::EaseIn => 2.0 * t,
            Easing::EaseOut => 2.0 - 2.0 * t,
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t
                } else {
                    4.0 - 4.0 * t
                }
            }
            Easing::CubicBezier(..) => {
                // Numerical derivative via central finite difference
                let h = 0.0001;
                let t0 = (t - h).max(0.0);
                let t1 = (t + h).min(1.0);
                let dt = t1 - t0;
                if dt < f32::EPSILON {
                    return 0.0;
                }
                (self.apply(t1) - self.apply(t0)) / dt
            }
        }
    }
}

/// Animation primitive representing an active animation state.
pub enum Animation<T: Animatable> {
    /// Simple tween from one value to another over a fixed duration.
    Tween {
        /// Starting value.
        from: T,
        /// Target value.
        to: T,
        /// Total duration of the animation.
        duration: Duration,
        /// Easing curve to apply.
        easing: Easing,
        /// Time elapsed since start.
        elapsed: Duration,
    },
    /// Physics-based spring animation.
    ///
    /// Uses a damped harmonic oscillator simulation.
    Spring {
        /// Current value.
        current: T,
        /// Target equilibrium value.
        target: T,
        /// Current velocity.
        velocity: T,
        /// Spring stiffness (k). Higher values mean stiffer/faster spring.
        stiffness: f32,
        /// Damping coefficient (c). Higher values mean less oscillation/slower settling.
        damping: f32,
    },
}

impl<T: Animatable> Animation<T> {
    /// Create a new tween animation.
    ///
    /// # Arguments
    ///
    /// * `from` - Starting value.
    /// * `to` - Target value.
    /// * `duration` - Time to complete the animation.
    /// * `easing` - Easing curve to use.
    pub fn tween(from: T, to: T, duration: Duration, easing: Easing) -> Self {
        Self::Tween {
            from,
            to,
            duration,
            easing,
            elapsed: Duration::ZERO,
        }
    }

    /// Create a spring animation.
    ///
    /// # Arguments
    ///
    /// * `from` - Starting value.
    /// * `to` - Target value.
    /// * `stiffness` - Spring stiffness constant (k). Typical values: 100.0 - 500.0.
    /// * `damping` - Damping coefficient (c). Typical values: 10.0 - 30.0.
    ///
    /// # Example
    ///
    /// ```
    /// use anim_graph::Animation;
    /// let spring = Animation::spring(0.0, 100.0, 200.0, 20.0);
    /// ```
    pub fn spring(from: T, to: T, stiffness: f32, damping: f32) -> Self {
        Self::Spring {
            current: from.clone(),
            target: to,
            velocity: T::zero(),
            stiffness,
            damping,
        }
    }

    /// Advance the animation by delta time, returns current value.
    ///
    /// For springs, this runs the physics simulation step.
    /// For tweens, this advances the elapsed time.
    pub fn tick(&mut self, dt: Duration) -> T {
        match self {
            Self::Tween {
                from,
                to,
                duration,
                easing,
                elapsed,
            } => {
                *elapsed += dt;
                let t = (elapsed.as_secs_f32() / duration.as_secs_f32()).min(1.0);
                let eased_t = easing.apply(t);
                from.interpolate(to, eased_t)
            }
            Self::Spring {
                current,
                target,
                velocity,
                stiffness,
                damping,
            } => {
                let dt_secs = dt.as_secs_f32();
                // Avoid instability with large time steps by clamping
                // Max 50ms per tick for stability
                let dt_secs = dt_secs.min(0.05);

                // Force = -k * (x - target) - d * v
                // displacement = current - target
                let displacement = current.sub(target);
                let spring_force = displacement.scale(-*stiffness);
                let damping_force = velocity.scale(-*damping);

                let acceleration = spring_force.add(&damping_force);

                // v += a * dt
                *velocity = velocity.add(&acceleration.scale(dt_secs));

                // x += v * dt
                *current = current.add(&velocity.scale(dt_secs));

                current.clone()
            }
        }
    }

    /// Check if animation is complete.
    ///
    /// - **Tween:** Returns true if `elapsed >= duration`.
    /// - **Spring:** Returns true if the spring has settled (velocity and displacement are negligible).
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Tween {
                elapsed, duration, ..
            } => elapsed >= duration,
            Self::Spring {
                current,
                target,
                velocity,
                ..
            } => {
                let dist_sq = current.distance_squared(target);
                let vel_sq = velocity.distance_squared(&T::zero());
                // Thresholds: small position error (sqrt(0.0001) = 0.01)
                // and small velocity (sqrt(0.0001) = 0.01 units/sec)
                dist_sq < 0.0001 && vel_sq < 0.0001
            }
        }
    }
}

/// Implement Animatable for f32.
impl Animatable for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }

    fn scale(&self, scalar: f32) -> Self {
        self * scalar
    }

    fn add(&self, other: &Self) -> Self {
        self + other
    }

    fn sub(&self, other: &Self) -> Self {
        self - other
    }

    fn zero() -> Self {
        0.0
    }

    fn distance_squared(&self, other: &Self) -> f32 {
        (self - other).powi(2)
    }
}

/// Implement Animatable for Color using SIMD-accelerated glam.
impl Animatable for render_engine::Color {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        // Use glam for SIMD-accelerated linear interpolation (4x faster)
        let v1 = self.as_vec4();
        let v2 = other.as_vec4();
        let result = v1.lerp(v2, t);
        render_engine::Color::from_vec4(result)
    }

    fn scale(&self, scalar: f32) -> Self {
        render_engine::Color::from_vec4(self.as_vec4() * scalar)
    }

    fn add(&self, other: &Self) -> Self {
        render_engine::Color::from_vec4(self.as_vec4() + other.as_vec4())
    }

    fn sub(&self, other: &Self) -> Self {
        render_engine::Color::from_vec4(self.as_vec4() - other.as_vec4())
    }

    fn zero() -> Self {
        render_engine::Color::from_vec4(render_engine::Vec4::ZERO)
    }

    fn distance_squared(&self, other: &Self) -> f32 {
        self.as_vec4().distance_squared(other.as_vec4())
    }
}
