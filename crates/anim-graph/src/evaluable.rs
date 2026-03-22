//! Evaluable trait — the core abstraction for animation evaluation.
//!
//! An `Evaluable<T>` is a pure function from normalized time (0.0–1.0) to a
//! [`Sample<T>`](crate::Sample). This is the animation analog of Orpheus's
//! `Pattern` trait: composable, queryable, and free of side effects.

use crate::{Animatable, Sample};

/// A pure animation segment that can be evaluated at any phase.
///
/// Phase is always normalized to 0.0–1.0 regardless of the segment's
/// real-time duration. The [`natural_duration`](Evaluable::natural_duration)
/// method reports the segment's preferred duration in seconds, which
/// composition types (e.g., [`Sequence`](crate::sequence::Sequence)) use
/// to allocate proportional time.
///
/// # Implementors
///
/// - [`Keyframe<T>`](crate::keyframe::Keyframe) — tween with easing
/// - [`Hold<T>`](crate::hold::Hold) — constant value (pause)
/// - [`SpringSegment<T>`](crate::spring_segment::SpringSegment) — physics spring with time budget
/// - [`Sequence<T>`](crate::sequence::Sequence) — end-to-end chain
/// - [`TimeWarp<T>`](crate::time_warp::TimeWarp) — phase remapping
/// - [`Stagger<T>`](crate::stagger::Stagger) — offset parallel composition
pub trait Evaluable<T: Animatable>: Send + Sync {
    /// Evaluate the animation at a normalized phase (0.0–1.0).
    ///
    /// Returns a [`Sample`] containing both the current value and its
    /// instantaneous velocity. Velocity is expressed in units of `T`
    /// per second (scaled by real-time duration externally).
    fn evaluate(&self, phase: f32) -> Sample<T>;

    /// The segment's preferred real-time duration in seconds.
    ///
    /// Used by composition types to allocate proportional phase ranges.
    /// For example, a 300ms keyframe and a 200ms hold in a sequence
    /// would get phase ranges 0.0–0.6 and 0.6–1.0 respectively.
    fn natural_duration(&self) -> f32;
}
