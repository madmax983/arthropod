//! Keyframe — a single tween segment with easing.
//!
//! Interpolates from value A to value B over normalized time,
//! with an easing curve controlling the rate of change.

use crate::evaluable::Evaluable;
use crate::{Animatable, Easing, Sample};

/// A single tween segment that interpolates between two values.
///
/// The velocity is derived analytically from the easing function's
/// derivative, scaled by the distance `(to - from)` and divided by
/// the segment's natural duration.
///
/// # Example
///
/// ```
/// use anim_graph::keyframe::Keyframe;
/// use anim_graph::{Easing, Sample};
///
/// let kf = Keyframe::new(0.0_f32, 100.0, Easing::Linear, 1.0);
/// let sample = kf.evaluate(0.5);
/// assert!((sample.value - 50.0).abs() < 0.01);
/// ```
pub struct Keyframe<T: Animatable> {
    pub from: T,
    pub to: T,
    pub easing: Easing,
    /// Natural duration in seconds.
    pub duration: f32,
}

impl<T: Animatable> Keyframe<T> {
    /// Create a new keyframe.
    ///
    /// - `from`/`to`: start and end values.
    /// - `easing`: the curve controlling interpolation rate.
    /// - `duration`: preferred real-time duration in seconds.
    pub fn new(from: T, to: T, easing: Easing, duration: f32) -> Self {
        Self {
            from,
            to,
            easing,
            duration,
        }
    }

    /// Evaluate the keyframe, returning value and velocity.
    ///
    /// Public method that implements the trait contract. Also usable
    /// directly without trait objects.
    pub fn evaluate(&self, phase: f32) -> Sample<T> {
        let phase = phase.clamp(0.0, 1.0);
        let eased = self.easing.apply(phase);
        let value = self.from.interpolate(&self.to, eased);

        // Velocity = easing'(t) * (to - from) / duration
        let deriv = self.easing.derivative(phase);
        let delta = self.to.sub(&self.from);
        let velocity = if self.duration > f32::EPSILON {
            delta.scale(deriv / self.duration)
        } else {
            T::zero()
        };

        Sample::new(value, velocity)
    }
}

impl<T: Animatable + Send + Sync> Evaluable<T> for Keyframe<T> {
    fn evaluate(&self, phase: f32) -> Sample<T> {
        self.evaluate(phase)
    }

    fn natural_duration(&self) -> f32 {
        self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyframe_at_start_returns_from() {
        let kf = Keyframe::new(0.0_f32, 100.0, Easing::Linear, 1.0);
        let s = kf.evaluate(0.0);
        assert!((s.value - 0.0).abs() < 1e-4);
    }

    #[test]
    fn keyframe_at_end_returns_to() {
        let kf = Keyframe::new(0.0_f32, 100.0, Easing::Linear, 1.0);
        let s = kf.evaluate(1.0);
        assert!((s.value - 100.0).abs() < 1e-4);
    }

    #[test]
    fn keyframe_midpoint_linear() {
        let kf = Keyframe::new(0.0_f32, 100.0, Easing::Linear, 1.0);
        let s = kf.evaluate(0.5);
        assert!((s.value - 50.0).abs() < 1e-4);
        // Linear velocity: (100 - 0) * 1.0 / 1.0 = 100 units/sec
        assert!((s.velocity - 100.0).abs() < 1e-2);
    }

    #[test]
    fn keyframe_ease_in_velocity_starts_slow() {
        let kf = Keyframe::new(0.0_f32, 100.0, Easing::EaseIn, 1.0);
        let s_start = kf.evaluate(0.0);
        let s_end = kf.evaluate(1.0);
        // EaseIn: velocity at start should be 0, at end should be max
        assert!(
            s_start.velocity.abs() < 1e-4,
            "EaseIn starts with zero velocity"
        );
        assert!(s_end.velocity > 100.0, "EaseIn ends with high velocity");
    }

    #[test]
    fn keyframe_color_interpolation() {
        use render_engine::Color;
        let kf = Keyframe::new(Color::RED, Color::BLUE, Easing::Linear, 0.5);
        let s = kf.evaluate(0.5);
        // Midpoint between RED and BLUE
        assert!((s.value.r() - 0.5).abs() < 0.01);
        assert!((s.value.b() - 0.5).abs() < 0.01);
    }

    #[test]
    fn keyframe_natural_duration() {
        let kf = Keyframe::new(0.0_f32, 1.0, Easing::Linear, 0.3);
        assert!((Evaluable::<f32>::natural_duration(&kf) - 0.3).abs() < 1e-6);
    }
}
