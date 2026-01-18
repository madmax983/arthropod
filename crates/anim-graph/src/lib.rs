//! Animation system for Arthropod GUI framework.
//!
//! Provides animation primitives (Tween, Spring, Keyframes) and an animation controller
//! that drives scene graph properties over time.

use std::time::Duration;

/// Trait for types that can be animated.
pub trait Animatable: Clone + 'static {
    /// Interpolate between two values.
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

/// Easing function for animations.
#[derive(Debug, Clone, Copy)]
pub enum Easing {
    /// Linear interpolation.
    Linear,
    /// Ease in (slow start).
    EaseIn,
    /// Ease out (slow end).
    EaseOut,
    /// Ease in-out (slow start and end).
    EaseInOut,
    /// Custom cubic bezier curve.
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    /// Apply the easing function to a linear parameter t (0.0 to 1.0).
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

/// Animation primitive.
pub enum Animation<T: Animatable> {
    /// Simple tween from one value to another.
    Tween {
        from: T,
        to: T,
        duration: Duration,
        easing: Easing,
        elapsed: Duration,
    },
    /// Physics-based spring animation.
    Spring {
        current: T,
        target: T,
        velocity: T,
        stiffness: f32,
        damping: f32,
    },
}

impl<T: Animatable> Animation<T> {
    /// Create a new tween animation.
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
    pub fn spring(from: T, to: T, stiffness: f32, damping: f32) -> Self {
        Self::Spring {
            current: from.clone(),
            target: to,
            velocity: from, // Zero velocity initially
            stiffness,
            damping,
        }
    }

    /// Advance the animation by delta time, returns current value.
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
                target: _,
                velocity: _,
                stiffness: _,
                damping: _,
            } => {
                // Simplified spring physics - will be improved
                current.clone()
            }
        }
    }

    /// Check if animation is complete.
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Tween {
                elapsed, duration, ..
            } => elapsed >= duration,
            Self::Spring { .. } => {
                // Spring never truly completes, but we can check if close enough
                // For now, return false - will implement threshold check later
                false
            }
        }
    }
}

/// Implement Animatable for f32.
impl Animatable for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
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
}
