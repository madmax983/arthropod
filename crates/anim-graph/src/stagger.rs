//! Stagger — offset parallel composition.
//!
//! Applies a template animation to multiple indices, each starting
//! at an offset delay. The choreography primitive for staggered
//! list entrances.

use crate::evaluable::Evaluable;
use crate::{Animatable, Sample};

/// Offsets copies of a template evaluable by a fixed delay.
///
/// Each index (0..count) starts at `index * offset` seconds into the
/// total duration. The total natural duration is
/// `template_duration + (count - 1) * offset`.
///
/// # Example
///
/// A stagger of 3 fade-ins with 50ms offset:
/// - Item 0: starts at 0ms
/// - Item 1: starts at 50ms
/// - Item 2: starts at 100ms
/// - Total duration: template_duration + 100ms
pub struct Stagger<T: Animatable + Send + Sync> {
    segments: Vec<Box<dyn Evaluable<T>>>,
    /// Offset between each child's start time in seconds.
    offset: f32,
    total_duration: f32,
}

impl<T: Animatable + Send + Sync> Stagger<T> {
    /// Create a stagger from a list of evaluable segments with a fixed offset.
    pub fn new(segments: Vec<Box<dyn Evaluable<T>>>, offset: f32) -> Self {
        let count = segments.len();
        let child_duration = segments
            .first()
            .map(|s| s.natural_duration())
            .unwrap_or(0.0);
        let total_duration = if count > 0 {
            child_duration + (count as f32 - 1.0) * offset
        } else {
            0.0
        };

        Self {
            segments,
            offset,
            total_duration,
        }
    }

    /// Evaluate a specific index at the given global phase.
    ///
    /// Returns `None` if the index hasn't started yet (phase before its offset).
    pub fn evaluate_at(&self, index: usize, phase: f32) -> Option<Sample<T>> {
        let seg = self.segments.get(index)?;
        let child_duration = seg.natural_duration();

        // This child's start time as a phase
        let start_phase = if self.total_duration > f32::EPSILON {
            (index as f32 * self.offset) / self.total_duration
        } else {
            0.0
        };

        // This child's end phase
        let end_phase = if self.total_duration > f32::EPSILON {
            start_phase + child_duration / self.total_duration
        } else {
            1.0
        };

        let phase = phase.clamp(0.0, 1.0);

        if phase < start_phase {
            // Not started yet — return start value
            return Some(seg.evaluate(0.0));
        }

        if phase >= end_phase {
            // Already finished — return end value
            return Some(seg.evaluate(1.0));
        }

        // Active: compute local phase
        let span = end_phase - start_phase;
        let local_phase = if span > f32::EPSILON {
            (phase - start_phase) / span
        } else {
            1.0
        };

        Some(seg.evaluate(local_phase.clamp(0.0, 1.0)))
    }
}

impl<T: Animatable + Send + Sync> Evaluable<T> for Stagger<T> {
    /// Evaluates the first segment at the given phase.
    ///
    /// For multi-target stagger evaluation, use `evaluate_at(index, phase)`.
    fn evaluate(&self, phase: f32) -> Sample<T> {
        self.evaluate_at(0, phase)
            .unwrap_or(Sample::at_rest(T::zero()))
    }

    fn natural_duration(&self) -> f32 {
        self.total_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Easing;
    use crate::keyframe::Keyframe;

    fn make_stagger() -> Stagger<f32> {
        Stagger::new(
            vec![
                Box::new(Keyframe::new(0.0_f32, 100.0, Easing::Linear, 0.3)),
                Box::new(Keyframe::new(0.0_f32, 100.0, Easing::Linear, 0.3)),
                Box::new(Keyframe::new(0.0_f32, 100.0, Easing::Linear, 0.3)),
            ],
            0.1, // 100ms offset between each
        )
    }

    #[test]
    fn stagger_first_starts_immediately() {
        let s = make_stagger();
        let sample = s.evaluate_at(0, 0.0).unwrap();
        assert!((sample.value - 0.0).abs() < 0.1);
    }

    #[test]
    fn stagger_second_starts_after_offset() {
        let s = make_stagger();
        // Total duration = 0.3 + 2 * 0.1 = 0.5
        // Second child starts at 0.1 / 0.5 = 0.2 phase
        let before = s.evaluate_at(1, 0.15).unwrap();
        // Should still be at start (not yet active)
        assert!(
            (before.value - 0.0).abs() < 0.1,
            "Second child shouldn't have started yet, got {}",
            before.value
        );

        let during = s.evaluate_at(1, 0.5).unwrap();
        // Should be mid-animation
        assert!(
            during.value > 10.0,
            "Second child should be animating at phase 0.5, got {}",
            during.value
        );
    }

    #[test]
    fn stagger_all_complete_at_end() {
        let s = make_stagger();
        for i in 0..3 {
            let sample = s.evaluate_at(i, 1.0).unwrap();
            assert!(
                (sample.value - 100.0).abs() < 0.1,
                "Child {i} should be complete at phase 1.0, got {}",
                sample.value
            );
        }
    }

    #[test]
    fn stagger_natural_duration() {
        let s = make_stagger();
        // 0.3 + 2 * 0.1 = 0.5
        assert!((s.total_duration - 0.5).abs() < 1e-6);
    }
}
