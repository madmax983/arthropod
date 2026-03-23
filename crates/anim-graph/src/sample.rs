//! Sample — a value paired with its instantaneous velocity.
//!
//! Every [`Evaluable`](crate::Evaluable) returns a `Sample` so that the timeline
//! system always knows both *where* and *how fast* an animation is moving.
//! This is the contract that makes spring handoff seamless: interrupting
//! a running animation just reads the current sample and seeds a spring
//! with its velocity.

use crate::Animatable;

/// A sampled animation value together with its first derivative (velocity).
///
/// Velocity is expressed in units of `T` per second, scaled by the
/// segment's real-time duration at evaluation time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample<T: Animatable> {
    /// The current value at the sampled phase.
    pub value: T,
    /// Instantaneous velocity (units of T per second).
    pub velocity: T,
}

impl<T: Animatable> Sample<T> {
    /// Create a sample with explicit value and velocity.
    pub fn new(value: T, velocity: T) -> Self {
        Self { value, velocity }
    }

    /// Create a sample at rest (velocity = zero).
    ///
    /// Useful for hold segments, completed animations, or initial states.
    pub fn at_rest(value: T) -> Self {
        Self {
            value,
            velocity: T::zero(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_new_stores_value_and_velocity() {
        let s = Sample::new(10.0_f32, 5.0);
        assert_eq!(s.value, 10.0);
        assert_eq!(s.velocity, 5.0);
    }

    #[test]
    fn sample_at_rest_has_zero_velocity() {
        let s = Sample::<f32>::at_rest(42.0);
        assert_eq!(s.value, 42.0);
        assert_eq!(s.velocity, 0.0);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn sample_clone_produces_equal_copy() {
        let s = Sample::new(3.14_f32, -1.0);
        let s2 = s;
        assert_eq!(s, s2);
    }
}
