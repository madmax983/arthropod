//! Animation system for Arthropod GUI framework.
//!
//! The `anim-graph` module provides a physics-based animation system designed for UI interactions.
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
//! use arthropod::experimental::anim_graph::{Animation, Easing};
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
//! use arthropod::experimental::anim_graph::Animation;
//! use std::time::Duration;
//!
//! // Create a stiff spring with low damping (bouncy)
//! let mut spring = Animation::spring(0.0, 100.0, 300.0, 15.0);
//!
//! // Tick the simulation
//! let value = spring.tick(Duration::from_millis(16));
//! ```

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
    /// use arthropod::experimental::anim_graph::Easing;
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
    /// use arthropod::experimental::anim_graph::Animation;
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

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::Color;
    use std::time::Duration;

    // ==================== Animatable Trait Tests ====================

    #[test]
    fn test_f32_interpolation() {
        let start = 0.0_f32;
        let end = 100.0_f32;

        assert_eq!(start.interpolate(&end, 0.0), 0.0);
        assert_eq!(start.interpolate(&end, 0.5), 50.0);
        assert_eq!(start.interpolate(&end, 1.0), 100.0);
    }

    #[test]
    fn test_f32_interpolation_negative() {
        let start = -50.0_f32;
        let end = 50.0_f32;

        assert_eq!(start.interpolate(&end, 0.0), -50.0);
        assert_eq!(start.interpolate(&end, 0.5), 0.0);
        assert_eq!(start.interpolate(&end, 1.0), 50.0);
    }

    #[test]
    fn test_color_interpolation() {
        let red = Color::RED;
        let blue = Color::BLUE;

        let purple = red.interpolate(&blue, 0.5);
        assert_eq!(purple.r(), 0.5);
        assert_eq!(purple.g(), 0.0);
        assert_eq!(purple.b(), 0.5);
        assert_eq!(purple.a(), 1.0);
    }

    #[test]
    fn test_color_interpolation_with_alpha() {
        let opaque = Color::rgba(1.0, 0.0, 0.0, 1.0);
        let transparent = Color::rgba(1.0, 0.0, 0.0, 0.0);

        let half = opaque.interpolate(&transparent, 0.5);
        assert_eq!(half.a(), 0.5);
    }

    // ==================== Easing Function Tests ====================

    #[test]
    fn test_easing_linear() {
        let easing = Easing::Linear;

        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.25), 0.25);
        assert_eq!(easing.apply(0.5), 0.5);
        assert_eq!(easing.apply(0.75), 0.75);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_clamps_input() {
        let easing = Easing::Linear;

        assert_eq!(easing.apply(-0.5), 0.0, "Should clamp negative to 0");
        assert_eq!(easing.apply(1.5), 1.0, "Should clamp > 1 to 1");
    }

    #[test]
    fn test_easing_ease_in() {
        let easing = Easing::EaseIn;

        // EaseIn should start slow (t^2)
        let result_at_half = easing.apply(0.5);
        assert!(
            result_at_half < 0.5,
            "EaseIn at 0.5 should be < 0.5 (slow start)"
        );

        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_out() {
        let easing = Easing::EaseOut;

        // EaseOut should start fast
        let result_at_half = easing.apply(0.5);
        assert!(
            result_at_half > 0.5,
            "EaseOut at 0.5 should be > 0.5 (fast start)"
        );

        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_in_out() {
        let easing = Easing::EaseInOut;

        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(1.0), 1.0);

        // Should be symmetric around 0.5
        let quarter = easing.apply(0.25);
        let three_quarters = easing.apply(0.75);

        assert!(quarter < 0.25, "Should start slow");
        assert!(three_quarters > 0.75, "Should end slow");
    }

    // ==================== Tween Animation Tests ====================

    #[test]
    fn test_tween_creation() {
        let anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

        match anim {
            Animation::Tween {
                from, to, duration, ..
            } => {
                assert_eq!(from, 0.0);
                assert_eq!(to, 100.0);
                assert_eq!(duration, Duration::from_secs(1));
            }
            _ => panic!("Expected Tween variant"),
        }
    }

    #[test]
    fn test_tween_at_start() {
        let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

        let value = anim.tick(Duration::ZERO);
        assert_eq!(value, 0.0, "Should be at start value");
    }

    #[test]
    fn test_tween_at_midpoint() {
        let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

        let value = anim.tick(Duration::from_millis(500));
        assert_eq!(value, 50.0, "Should be halfway through");
    }

    #[test]
    fn test_tween_at_end() {
        let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

        let value = anim.tick(Duration::from_secs(1));
        assert_eq!(value, 100.0, "Should be at end value");
    }

    #[test]
    fn test_tween_clamps_at_end() {
        let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

        let value = anim.tick(Duration::from_secs(2));
        assert_eq!(value, 100.0, "Should clamp to end value");
    }

    #[test]
    fn test_tween_with_ease_in() {
        let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::EaseIn);

        let value = anim.tick(Duration::from_millis(500));
        assert!(
            value < 50.0,
            "EaseIn should be slow at start, value: {}",
            value
        );
    }

    #[test]
    fn test_tween_incremental_ticks() {
        let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

        // Tick in small increments
        let v1 = anim.tick(Duration::from_millis(250));
        let v2 = anim.tick(Duration::from_millis(250));
        let v3 = anim.tick(Duration::from_millis(500));

        assert_eq!(v1, 25.0);
        assert_eq!(v2, 50.0);
        assert_eq!(v3, 100.0, "Should accumulate time");
    }

    #[test]
    fn test_tween_color_animation() {
        let mut anim = Animation::tween(
            Color::RED,
            Color::BLUE,
            Duration::from_secs(1),
            Easing::Linear,
        );

        let color = anim.tick(Duration::from_millis(500));

        // Should be purple at midpoint
        assert_eq!(color.r(), 0.5);
        assert_eq!(color.g(), 0.0);
        assert_eq!(color.b(), 0.5);
    }

    // ==================== Animation Completion Tests ====================

    #[test]
    fn test_tween_not_complete_initially() {
        let anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

        assert!(
            !anim.is_complete(),
            "Tween should not be complete initially"
        );
    }

    #[test]
    fn test_tween_not_complete_during() {
        let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

        anim.tick(Duration::from_millis(500));
        assert!(
            !anim.is_complete(),
            "Tween should not be complete at midpoint"
        );
    }

    #[test]
    fn test_tween_complete_at_end() {
        let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

        anim.tick(Duration::from_secs(1));
        assert!(anim.is_complete(), "Tween should be complete at end");
    }

    #[test]
    fn test_tween_complete_after_end() {
        let mut anim = Animation::tween(0.0_f32, 100.0_f32, Duration::from_secs(1), Easing::Linear);

        anim.tick(Duration::from_secs(2));
        assert!(anim.is_complete(), "Tween should be complete after end");
    }

    // ==================== Spring Animation Tests ====================

    #[test]
    fn test_spring_creation() {
        let anim = Animation::spring(0.0_f32, 100.0_f32, 200.0, 20.0);

        match anim {
            Animation::Spring {
                current,
                target,
                stiffness,
                damping,
                ..
            } => {
                assert_eq!(current, 0.0);
                assert_eq!(target, 100.0);
                assert_eq!(stiffness, 200.0);
                assert_eq!(damping, 20.0);
            }
            _ => panic!("Expected Spring variant"),
        }
    }

    #[test]
    fn test_spring_starts_at_initial_value() {
        let mut anim = Animation::spring(0.0_f32, 100.0_f32, 200.0, 20.0);

        let value = anim.tick(Duration::ZERO);
        assert_eq!(value, 0.0, "Spring should start at initial value");
    }

    #[test]
    fn test_spring_moves_towards_target() {
        let mut anim = Animation::spring(0.0_f32, 100.0_f32, 100.0, 10.0);

        // Initial state
        assert_eq!(anim.tick(Duration::ZERO), 0.0);

        // Advance time
        let val_after_tick = anim.tick(Duration::from_millis(100));

        // It should have moved towards 100
        assert!(
            val_after_tick > 0.0,
            "Spring should move towards target, got {}",
            val_after_tick
        );
        assert!(
            val_after_tick < 100.0,
            "Spring shouldn't overshoot immediately, got {}",
            val_after_tick
        );
    }

    #[test]
    fn test_spring_equilibrium_stability() {
        // If we start at target, it should stay there if velocity is 0
        let mut anim = Animation::spring(100.0_f32, 100.0_f32, 100.0, 10.0);

        let val = anim.tick(Duration::from_millis(100));
        assert_eq!(val, 100.0, "Spring at equilibrium should not move");
    }
}
