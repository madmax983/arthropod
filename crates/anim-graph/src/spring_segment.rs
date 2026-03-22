//! SpringSegment — a physics spring with a time budget.
//!
//! Unlike a free-running spring, a `SpringSegment` has a finite duration
//! (estimated from physics parameters). The timeline treats it like any
//! other segment — when its phase reaches 1.0, the next segment begins,
//! whether or not the spring has fully settled.

use crate::evaluable::Evaluable;
use crate::{Animatable, Sample};

/// Maximum simulation step size for numerical stability.
const MAX_STEP_SECS: f32 = 0.001; // 1ms steps

/// A damped harmonic oscillator segment with a time budget.
///
/// The spring simulation runs for `phase * budget` seconds, subdivided
/// into small fixed steps for stability. The budget is the segment's
/// natural duration, estimated from the settling time of the spring.
///
/// # Velocity preservation
///
/// `initial_velocity` allows interruption chains: when a timeline is
/// interrupted, the current velocity feeds into the next spring segment's
/// initial conditions, preserving momentum.
pub struct SpringSegment<T: Animatable> {
    pub from: T,
    pub to: T,
    pub initial_velocity: T,
    pub stiffness: f32,
    pub damping: f32,
    /// Time budget in seconds (the segment's natural duration).
    pub budget: f32,
}

impl<T: Animatable> SpringSegment<T> {
    /// Create a new spring segment with default zero initial velocity.
    pub fn new(from: T, to: T, stiffness: f32, damping: f32) -> Self {
        // Estimate settling time: ~4 time constants for 98% settling
        // Time constant ≈ 2 * mass / damping (mass = 1 for UI springs)
        let budget = if damping > f32::EPSILON {
            (4.0 * 2.0 / damping).max(0.1)
        } else {
            2.0 // Undamped spring — arbitrary budget
        };

        Self {
            from,
            to,
            initial_velocity: T::zero(),
            stiffness,
            damping,
            budget,
        }
    }

    /// Set initial velocity (for interruption chains).
    pub fn with_initial_velocity(mut self, velocity: T) -> Self {
        self.initial_velocity = velocity;
        self
    }

    /// Set an explicit time budget override.
    pub fn with_budget(mut self, budget: f32) -> Self {
        self.budget = budget;
        self
    }

    /// Evaluate the spring at a given phase by running the simulation.
    ///
    /// Runs the spring physics from `t=0` to `t = phase * budget`,
    /// subdivided into small fixed steps.
    pub fn evaluate(&self, phase: f32) -> Sample<T> {
        let phase = phase.clamp(0.0, 1.0);
        let target_time = phase * self.budget;

        let mut current = self.from.clone();
        let mut velocity = self.initial_velocity.clone();
        let mut time = 0.0_f32;

        while time < target_time {
            let dt = (target_time - time).min(MAX_STEP_SECS);

            // Force = -k * (x - target) - c * v
            let displacement = current.sub(&self.to);
            let spring_force = displacement.scale(-self.stiffness);
            let damping_force = velocity.scale(-self.damping);
            let acceleration = spring_force.add(&damping_force);

            // Semi-implicit Euler integration
            velocity = velocity.add(&acceleration.scale(dt));
            current = current.add(&velocity.scale(dt));

            time += dt;
        }

        // Velocity is already in T/second (physics simulation uses seconds)
        Sample::new(current, velocity)
    }
}

impl<T: Animatable + Send + Sync> Evaluable<T> for SpringSegment<T> {
    fn evaluate(&self, phase: f32) -> Sample<T> {
        self.evaluate(phase)
    }

    fn natural_duration(&self) -> f32 {
        self.budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_segment_starts_at_from() {
        let s = SpringSegment::new(0.0_f32, 100.0, 300.0, 30.0);
        let sample = s.evaluate(0.0);
        assert!((sample.value - 0.0).abs() < 1e-4);
    }

    #[test]
    fn spring_segment_approaches_target() {
        let s = SpringSegment::new(0.0_f32, 100.0, 300.0, 30.0);
        let sample = s.evaluate(1.0);
        // Should be very close to target after full budget
        assert!(
            (sample.value - 100.0).abs() < 2.0,
            "Spring should approach target, got {}",
            sample.value
        );
    }

    #[test]
    fn spring_segment_produces_velocity() {
        let s = SpringSegment::new(0.0_f32, 100.0, 300.0, 30.0);
        let sample = s.evaluate(0.1);
        // Early in the animation, velocity should be positive (moving toward target)
        assert!(
            sample.velocity > 0.0,
            "Early spring should have positive velocity, got {}",
            sample.velocity
        );
    }

    #[test]
    fn spring_segment_natural_duration() {
        let s = SpringSegment::new(0.0_f32, 100.0, 300.0, 30.0);
        assert!(s.budget > 0.0);
        assert!((Evaluable::<f32>::natural_duration(&s) - s.budget).abs() < 1e-6);
    }

    #[test]
    fn spring_segment_with_initial_velocity() {
        let s = SpringSegment::new(0.0_f32, 100.0, 300.0, 30.0).with_initial_velocity(500.0);
        let sample = s.evaluate(0.05);
        // With high initial velocity toward target, should advance faster
        let s_no_vel = SpringSegment::new(0.0_f32, 100.0, 300.0, 30.0);
        let sample_no_vel = s_no_vel.evaluate(0.05);
        assert!(
            sample.value > sample_no_vel.value,
            "Initial velocity should accelerate toward target"
        );
    }
}
