//! Hold — a constant-value segment for pauses and delays.
//!
//! Useful in sequences to create pauses between animated segments.
//! Always returns the same value with zero velocity.

use crate::evaluable::Evaluable;
use crate::{Animatable, Sample};

/// A segment that holds a constant value for a given duration.
///
/// Velocity is always zero. This is the animation equivalent of a rest
/// note in music — silence between movements.
///
/// # Example
///
/// ```
/// use anim_graph::hold::Hold;
/// use anim_graph::Sample;
///
/// let h = Hold::new(42.0_f32, 0.5);
/// let s = h.evaluate(0.5);
/// assert_eq!(s.value, 42.0);
/// assert_eq!(s.velocity, 0.0);
/// ```
pub struct Hold<T: Animatable> {
    pub value: T,
    /// Duration in seconds.
    pub duration: f32,
}

impl<T: Animatable> Hold<T> {
    /// Create a new hold segment.
    pub fn new(value: T, duration: f32) -> Self {
        Self { value, duration }
    }

    /// Evaluate the hold (always returns the same value).
    pub fn evaluate(&self, _phase: f32) -> Sample<T> {
        Sample::at_rest(self.value.clone())
    }
}

impl<T: Animatable + Send + Sync> Evaluable<T> for Hold<T> {
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
    use crate::evaluable::Evaluable;

    #[test]
    fn hold_returns_constant_value() {
        let h = Hold::new(42.0_f32, 1.0);
        assert_eq!(h.evaluate(0.0).value, 42.0);
        assert_eq!(h.evaluate(0.5).value, 42.0);
        assert_eq!(h.evaluate(1.0).value, 42.0);
    }

    #[test]
    fn hold_velocity_is_zero() {
        let h = Hold::new(100.0_f32, 0.5);
        assert_eq!(h.evaluate(0.5).velocity, 0.0);
    }

    #[test]
    fn hold_natural_duration() {
        let h = Hold::new(0.0_f32, 0.75);
        assert!((Evaluable::<f32>::natural_duration(&h) - 0.75).abs() < 1e-6);
    }
}
