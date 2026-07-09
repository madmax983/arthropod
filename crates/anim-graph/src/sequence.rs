//! Sequence — chains evaluables end-to-end.
//!
//! Phase 0.0–1.0 is proportionally distributed across children
//! based on their `natural_duration()` ratios.

use crate::evaluable::Evaluable;
use crate::{Animatable, Sample};

/// A sequential composition of animation segments.
///
/// Each child gets a proportional slice of the 0.0–1.0 phase range
/// based on its `natural_duration()` relative to the total. Evaluating
/// at a given phase delegates to whichever child "owns" that phase region.
///
/// # Boundary continuity
///
/// For smooth transitions, the end value of segment N should equal the
/// start value of segment N+1. This is the caller's responsibility —
/// `Sequence` does not enforce continuity.
pub struct Sequence<T: Animatable> {
    segments: Vec<Box<dyn Evaluable<T>>>,
    /// Precomputed normalized boundaries: `(start, end)` for each segment.
    boundaries: Vec<(f32, f32)>,
    total_duration: f32,
}

impl<T: Animatable> Sequence<T> {
    /// Create a sequence from a list of evaluable segments.
    ///
    pub fn new(segments: Vec<Box<dyn Evaluable<T>>>) -> Self {
        let total_duration: f32 = segments.iter().map(|s| s.natural_duration()).sum();
        let mut boundaries = Vec::with_capacity(segments.len());
        let mut cursor = 0.0_f32;

        for seg in &segments {
            let proportion = if total_duration > f32::EPSILON {
                seg.natural_duration() / total_duration
            } else {
                1.0 / segments.len() as f32
            };
            boundaries.push((cursor, cursor + proportion));
            cursor += proportion;
        }

        Self {
            segments,
            boundaries,
            total_duration,
        }
    }

    /// Evaluate the sequence at a global phase.
    pub fn evaluate(&self, phase: f32) -> Sample<T> {
        if self.segments.is_empty() {
            return Sample::at_rest(T::zero());
        }

        let phase = phase.clamp(0.0, 1.0);

        // Find which segment owns this phase
        for (i, &(start, end)) in self.boundaries.iter().enumerate() {
            if phase < end || i == self.segments.len() - 1 {
                // Compute local phase within this segment
                let span = end - start;
                let local_phase = if span > f32::EPSILON {
                    ((phase - start) / span).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                return self.segments[i].evaluate(local_phase);
            }
        }

        // Fallback: evaluate last segment at end
        if let Some(last) = self.segments.last() {
            last.evaluate(1.0)
        } else {
            Sample::at_rest(T::zero())
        }
    }
}

impl<T: Animatable + Send + Sync> Evaluable<T> for Sequence<T> {
    fn evaluate(&self, phase: f32) -> Sample<T> {
        self.evaluate(phase)
    }

    fn natural_duration(&self) -> f32 {
        self.total_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Easing;
    use crate::hold::Hold;
    use crate::keyframe::Keyframe;

    #[test]
    fn sequence_two_keyframes() {
        let seq = Sequence::new(vec![
            Box::new(Keyframe::new(0.0_f32, 50.0, Easing::Linear, 1.0)),
            Box::new(Keyframe::new(50.0, 100.0, Easing::Linear, 1.0)),
        ]);

        // Phase 0.0 = start of first segment
        assert!((seq.evaluate(0.0).value - 0.0).abs() < 0.1);
        // Phase 0.25 = midpoint of first segment
        assert!((seq.evaluate(0.25).value - 25.0).abs() < 0.1);
        // Phase 0.5 = boundary (start of second segment)
        assert!((seq.evaluate(0.5).value - 50.0).abs() < 0.1);
        // Phase 0.75 = midpoint of second segment
        assert!((seq.evaluate(0.75).value - 75.0).abs() < 0.1);
        // Phase 1.0 = end
        assert!((seq.evaluate(1.0).value - 100.0).abs() < 0.1);
    }

    #[test]
    fn sequence_boundary_continuity() {
        let seq = Sequence::new(vec![
            Box::new(Keyframe::new(0.0_f32, 50.0, Easing::Linear, 0.5)),
            Box::new(Keyframe::new(50.0, 100.0, Easing::Linear, 0.5)),
        ]);

        // Just before and after boundary should be continuous
        let before = seq.evaluate(0.499).value;
        let after = seq.evaluate(0.501).value;
        assert!(
            (before - after).abs() < 1.0,
            "Boundary should be continuous: before={before}, after={after}"
        );
    }

    #[test]
    fn sequence_with_hold() {
        let seq = Sequence::new(vec![
            Box::new(Keyframe::new(0.0_f32, 100.0, Easing::Linear, 0.5)),
            Box::new(Hold::new(100.0_f32, 0.5)),
        ]);

        // Phase 0.75 = midpoint of hold
        let s = seq.evaluate(0.75);
        assert!((s.value - 100.0).abs() < 0.1);
        assert!((s.velocity - 0.0).abs() < 0.1);
    }

    #[test]
    fn sequence_natural_duration_is_sum() {
        let seq = Sequence::new(vec![
            Box::new(Keyframe::new(0.0_f32, 1.0, Easing::Linear, 0.3)),
            Box::new(Hold::new(1.0_f32, 0.2)),
            Box::new(Keyframe::new(1.0, 0.0, Easing::Linear, 0.5)),
        ]);

        assert!((seq.natural_duration() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sequence_single_segment_passthrough() {
        let seq = Sequence::new(vec![Box::new(Keyframe::new(
            0.0_f32,
            100.0,
            Easing::Linear,
            1.0,
        ))]);

        assert!((seq.evaluate(0.5).value - 50.0).abs() < 0.1);
    }
}
