//! TimeWarp — phase remapping wrapper.
//!
//! Wraps an evaluable and applies a transformation to its phase
//! before delegation. Enables reverse, speed changes, and custom curves.

use crate::evaluable::Evaluable;
use crate::{Animatable, Sample};

/// Wraps an evaluable and remaps its phase through a curve.
///
/// The warp function maps `phase -> phase`, allowing time manipulation:
/// - `|t| 1.0 - t` — reverse
/// - `|t| (t * 2.0).min(1.0)` — double speed (first half only)
/// - `|t| t * t` — quadratic acceleration of time itself
pub struct TimeWarp<T: Animatable> {
    inner: Box<dyn Evaluable<T>>,
    curve: Box<dyn Fn(f32) -> f32 + Send + Sync>,
    duration_override: Option<f32>,
}

impl<T: Animatable> TimeWarp<T> {
    /// Create a time-warped evaluable.
    pub fn new(
        inner: Box<dyn Evaluable<T>>,
        curve: impl Fn(f32) -> f32 + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner,
            curve: Box::new(curve),
            duration_override: None,
        }
    }

    /// Override the natural duration (e.g., for speed changes).
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration_override = Some(duration);
        self
    }

    /// Create a reversed evaluable.
    pub fn reverse(inner: Box<dyn Evaluable<T>>) -> Self {
        Self::new(inner, |t| 1.0 - t)
    }

    /// Evaluate with phase remapping.
    pub fn evaluate(&self, phase: f32) -> Sample<T> {
        let warped = (self.curve)(phase.clamp(0.0, 1.0)).clamp(0.0, 1.0);
        self.inner.evaluate(warped)
    }
}

impl<T: Animatable + Send + Sync> Evaluable<T> for TimeWarp<T> {
    fn evaluate(&self, phase: f32) -> Sample<T> {
        self.evaluate(phase)
    }

    fn natural_duration(&self) -> f32 {
        self.duration_override
            .unwrap_or_else(|| self.inner.natural_duration())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Easing;
    use crate::keyframe::Keyframe;

    #[test]
    fn time_warp_identity_passthrough() {
        let kf = Box::new(Keyframe::new(0.0_f32, 100.0, Easing::Linear, 1.0));
        let warped = TimeWarp::new(kf, |t| t);

        assert!((warped.evaluate(0.5).value - 50.0).abs() < 0.1);
    }

    #[test]
    fn time_warp_reverse() {
        let kf = Box::new(Keyframe::new(0.0_f32, 100.0, Easing::Linear, 1.0));
        let warped = TimeWarp::reverse(kf);

        // Phase 0.0 should map to inner phase 1.0 (value = 100)
        assert!((warped.evaluate(0.0).value - 100.0).abs() < 0.1);
        // Phase 1.0 should map to inner phase 0.0 (value = 0)
        assert!((warped.evaluate(1.0).value - 0.0).abs() < 0.1);
    }

    #[test]
    fn time_warp_double_speed() {
        let kf = Box::new(Keyframe::new(0.0_f32, 100.0, Easing::Linear, 1.0));
        let warped = TimeWarp::new(kf, |t| (t * 2.0).min(1.0));

        // At phase 0.5, inner phase is 1.0 (fully complete)
        assert!((warped.evaluate(0.5).value - 100.0).abs() < 0.1);
    }
}
