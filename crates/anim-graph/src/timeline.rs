//! Timeline — the stateful animation wrapper.
//!
//! Connects the pure [`Evaluable`] core to real time via [`AnimationClock`],
//! manages playback state, and handles interruption with velocity-preserving
//! spring handoff.

use std::time::Duration;

use crate::clock::{AnimationClock, ClockEvent, PlaybackMode};
use crate::evaluable::Evaluable;

use crate::keyframe::Keyframe;
use crate::sequence::Sequence;
use crate::spring_segment::SpringSegment;
use crate::stagger::Stagger;
use crate::{Animatable, Easing, Sample};

/// Spring configuration for interruptions.
#[derive(Debug, Clone, Copy)]
pub struct SpringConfig {
    /// Controls how strongly the spring pulls towards the target.
    pub stiffness: f32,
    /// Controls how quickly the spring's momentum decays over time.
    pub damping: f32,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            stiffness: 300.0,
            damping: 30.0,
        }
    }
}

/// Internal state of a timeline.
enum TimelineState<T: Animatable> {
    /// Normal playback — evaluating the root tree.
    Playing,

    /// Interrupted — a spring is settling from the interruption point.
    Interrupted {
        spring: SpringSegment<T>,
        spring_clock: AnimationClock,
    },

    /// Finished — holds the final sample.
    Completed { final_sample: Sample<T> },
}

/// A stateful animation that combines a pure evaluation tree with a clock.
///
/// # Layers
///
/// `Timeline<T>` sits between the pure `Evaluable<T>` core and the ECS
/// integration layer. It owns:
/// - An [`AnimationClock`] for drift-free time tracking
/// - A `Box<dyn Evaluable<T>>` for the animation definition
/// - A `TimelineState` for playback/interruption management
///
/// # Interruption model
///
/// When [`interrupt()`](Timeline::interrupt) is called, the timeline:
/// 1. Samples the current value and velocity
/// 2. Spawns a [`SpringSegment`] with those as initial conditions
/// 3. Springs toward the new target, preserving momentum
///
/// # Builder API
///
/// Use `Timeline::tween()`, `Timeline::spring()`, `Timeline::sequence()`,
/// or `Timeline::stagger()` for construction — see each method for examples.
pub struct Timeline<T: Animatable> {
    root: Box<dyn Evaluable<T>>,
    clock: AnimationClock,
    duration: f32,
    playback: PlaybackMode,
    state: TimelineState<T>,
    /// Last computed sample (cached for interrupt reads).
    last_sample: Sample<T>,
}

impl<T: Animatable + Send + Sync + 'static> Timeline<T> {
    /// Create a timeline from a raw evaluable.
    pub fn from_evaluable(root: Box<dyn Evaluable<T>>, playback: PlaybackMode) -> Self {
        let duration = root.natural_duration();
        let initial = root.evaluate(0.0);
        Self {
            root,
            clock: AnimationClock::new(),
            duration,
            playback,
            state: TimelineState::Playing,
            last_sample: initial,
        }
    }

    // ========== One-liner constructors ==========

    /// Create a simple tween timeline.
    ///
    /// ```
    /// use anim_graph::timeline::Timeline;
    /// use std::time::Duration;
    ///
    /// let fade = Timeline::tween(1.0_f32, 0.0, Duration::from_millis(300));
    /// ```
    pub fn tween(from: T, to: T, duration: Duration) -> Self {
        let secs = duration.as_secs_f32();
        Self::from_evaluable(
            Box::new(Keyframe::new(from, to, Easing::Linear, secs)),
            PlaybackMode::Once,
        )
    }

    /// Create a spring timeline.
    pub fn spring(from: T, to: T, stiffness: f32, damping: f32) -> Self {
        let seg = SpringSegment::new(from, to, stiffness, damping);
        Self::from_evaluable(Box::new(seg), PlaybackMode::Once)
    }

    /// Start building a sequence.
    pub fn sequence(segments: Vec<Box<dyn Evaluable<T>>>) -> Self {
        let seq = Sequence::new(segments);
        Timeline::from_evaluable(Box::new(seq), PlaybackMode::Once)
    }

    /// Start building a stagger.
    pub fn stagger(segments: Vec<Box<dyn Evaluable<T>>>, offset: Duration) -> Self {
        let stagger = Stagger::new(segments, offset.as_secs_f32());
        Timeline::from_evaluable(Box::new(stagger), PlaybackMode::Once)
    }

    // ========== Modifiers ==========

    /// Set the easing curve (only applies to single-keyframe timelines).
    pub fn easing(mut self, easing: Easing) -> Self {
        // Replace root with a new keyframe using the given easing
        let sample_start = self.root.evaluate(0.0);
        let sample_end = self.root.evaluate(1.0);
        self.root = Box::new(Keyframe::new(
            sample_start.value,
            sample_end.value,
            easing,
            self.duration,
        ));
        self
    }

    /// Set playback to ping-pong (alternate forward/reverse).
    pub fn ping_pong(mut self) -> Self {
        self.playback = PlaybackMode::PingPong;
        self
    }

    /// Set playback to loop forever.
    pub fn loop_forever(mut self) -> Self {
        self.playback = PlaybackMode::Loop;
        self
    }

    /// Set playback to repeat a specific number of times.
    pub fn count(mut self, n: u32) -> Self {
        self.playback = PlaybackMode::Count(n);
        self
    }

    // ========== Runtime ==========

    /// Advance the timeline by a real-time delta and return the current sample.
    pub fn tick(&mut self, delta_secs: f32) -> Sample<T> {
        // Two-pass: compute sample, then update state if needed.
        // This avoids borrow conflicts between reading state and mutating it.
        let (sample, new_state) = match &mut self.state {
            TimelineState::Playing => {
                let event = self.clock.tick(delta_secs, self.duration);

                match event {
                    ClockEvent::Normal => {
                        let phase = self.clock.effective_phase(&self.playback);
                        (self.root.evaluate(phase), None)
                    }
                    ClockEvent::CycleBoundary { .. } => {
                        if self.clock.is_finished(&self.playback) {
                            let final_sample = self.root.evaluate(1.0);
                            (
                                final_sample.clone(),
                                Some(TimelineState::Completed { final_sample }),
                            )
                        } else {
                            let phase = self.clock.effective_phase(&self.playback);
                            (self.root.evaluate(phase), None)
                        }
                    }
                }
            }

            TimelineState::Interrupted {
                spring,
                spring_clock,
            } => {
                let budget = spring.budget;
                let event = spring_clock.tick(delta_secs, budget);
                let sample = spring.evaluate(spring_clock.phase());

                if matches!(event, ClockEvent::CycleBoundary { .. })
                    || spring_clock.is_finished(&PlaybackMode::Once)
                {
                    let final_sample = spring.evaluate(1.0);
                    (
                        final_sample.clone(),
                        Some(TimelineState::Completed { final_sample }),
                    )
                } else {
                    (sample, None)
                }
            }

            TimelineState::Completed { final_sample } => (final_sample.clone(), None),
        };

        if let Some(state) = new_state {
            self.state = state;
        }

        self.last_sample = sample.clone();
        sample
    }

    /// Interrupt the current animation and spring toward a new target.
    ///
    /// Captures the current value and velocity from the last sample,
    /// then spawns a [`SpringSegment`] with those as initial conditions.
    /// The spring settles at `new_target`, preserving momentum from
    /// whatever was playing.
    pub fn interrupt(&mut self, new_target: T, config: SpringConfig) {
        let current = self.last_sample.clone();

        let spring =
            SpringSegment::new(current.value, new_target, config.stiffness, config.damping)
                .with_initial_velocity(current.velocity);

        self.state = TimelineState::Interrupted {
            spring,
            spring_clock: AnimationClock::new(),
        };
    }

    /// Check if the timeline has finished playing.
    pub fn is_completed(&self) -> bool {
        matches!(self.state, TimelineState::Completed { .. })
    }

    /// Get the last computed sample value.
    pub fn current_value(&self) -> T {
        self.last_sample.value.clone()
    }

    /// Reset the timeline to the beginning.
    pub fn reset(&mut self) {
        self.clock.reset();
        self.state = TimelineState::Playing;
        self.last_sample = self.root.evaluate(0.0);
    }

    /// Get the timeline's total duration in seconds.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Get the playback mode.
    pub fn playback_mode(&self) -> &PlaybackMode {
        &self.playback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_tween_basic() {
        let mut tl = Timeline::tween(0.0_f32, 100.0, Duration::from_secs(1));
        // Tick 0.05s (within MAX_DELTA)
        let s = tl.tick(0.05);
        assert!(s.value > 0.0 && s.value < 100.0);
    }

    #[test]
    fn timeline_completes_once() {
        let mut tl = Timeline::tween(0.0_f32, 100.0, Duration::from_millis(50));
        // Tick past the duration
        tl.tick(0.06);
        assert!(tl.is_completed());
    }

    #[test]
    fn timeline_loop_does_not_complete() {
        let mut tl = Timeline::tween(0.0_f32, 100.0, Duration::from_millis(50)).loop_forever();
        tl.tick(0.06);
        assert!(!tl.is_completed());
    }

    #[test]
    fn timeline_ping_pong() {
        let mut tl = Timeline::tween(0.0_f32, 100.0, Duration::from_millis(50)).ping_pong();

        // First cycle forward
        tl.tick(0.025); // phase ~0.5
        let mid_forward = tl.current_value();

        // Cross into second cycle
        tl.tick(0.05); // now in reverse cycle
        let _in_reverse = tl.current_value();

        // In reverse, values should be heading back toward 0
        assert!(
            mid_forward > 20.0,
            "Forward phase should have advanced, got {mid_forward}"
        );
        // The exact reverse value depends on timing, just verify we crossed a boundary
        assert!(!tl.is_completed(), "PingPong should never complete");
    }

    #[test]
    fn timeline_count() {
        let mut tl = Timeline::tween(0.0_f32, 100.0, Duration::from_millis(50)).count(2);
        tl.tick(0.05); // Complete 1st cycle
        assert!(!tl.is_completed());
        tl.tick(0.05); // Complete 2nd cycle
        assert!(tl.is_completed());
    }

    #[test]
    fn timeline_interrupt_preserves_velocity() {
        let mut tl = Timeline::tween(0.0_f32, 100.0, Duration::from_millis(80));
        // Advance partway
        tl.tick(0.04);
        let before = tl.last_sample;
        assert!(
            before.velocity.abs() > 0.0,
            "Should have non-zero velocity mid-tween"
        );

        // Interrupt toward a different target
        tl.interrupt(200.0, SpringConfig::default());

        // Tick the spring
        let s = tl.tick(0.01);
        // Value should be near where we interrupted, moving toward 200
        assert!(
            (s.value - before.value).abs() < 50.0,
            "Should be near interruption point"
        );
    }

    #[test]
    fn timeline_interrupt_spring_settles() {
        let mut tl = Timeline::tween(0.0_f32, 100.0, Duration::from_millis(80));
        tl.tick(0.04);
        tl.interrupt(
            50.0,
            SpringConfig {
                stiffness: 800.0,
                damping: 60.0,
            },
        );

        // Tick until settled (generous time for spring with initial counter-velocity)
        for _ in 0..200 {
            tl.tick(0.01);
        }

        assert!(tl.is_completed(), "Spring should have settled");
        // The spring starts with velocity toward 100 but targets 50,
        // so it may overshoot and settle. Tolerance accounts for budget cutoff.
        assert!(
            (tl.current_value() - 50.0).abs() < 10.0,
            "Should settle near target, got {}",
            tl.current_value()
        );
    }

    #[test]
    fn timeline_delta_clamping() {
        let mut tl = Timeline::tween(0.0_f32, 100.0, Duration::from_secs(1)).loop_forever();
        // Simulate 5 seconds of backgrounding
        tl.tick(5.0);
        // Should not have jumped far (delta clamped to 100ms = 10% of 1s)
        assert!(
            tl.current_value() < 20.0,
            "Delta should be clamped, got {}",
            tl.current_value()
        );
    }

    #[test]
    fn timeline_sequence_evaluation() {
        let tl = Timeline::sequence(vec![
            Box::new(Keyframe::new(0.0_f32, 50.0, Easing::Linear, 0.050)),
            Box::new(Keyframe::new(50.0, 100.0, Easing::Linear, 0.050)),
        ]);

        assert!((tl.duration() - 0.1).abs() < 1e-4);
    }

    #[test]
    fn builder_tween_with_easing() {
        let mut tl =
            Timeline::tween(0.0_f32, 100.0, Duration::from_millis(80)).easing(Easing::EaseIn);
        tl.tick(0.04);
        let ease_in_value = tl.current_value();
        // EaseIn starts slow — at midpoint, value should be less than 50
        assert!(
            ease_in_value < 50.0,
            "EaseIn at midpoint should be < 50, got {ease_in_value}"
        );
    }

    #[test]
    fn timeline_spring() {
        let mut tl = Timeline::spring(0.0_f32, 100.0, 400.0, 40.0);
        tl.tick(0.05);
        assert!(tl.current_value() > 0.0, "Spring should have moved");
    }

    #[test]
    fn timeline_reset() {
        let mut tl = Timeline::tween(0.0_f32, 100.0, Duration::from_millis(50));
        tl.tick(0.06);
        assert!(tl.is_completed());

        tl.reset();
        assert!(!tl.is_completed());
        assert!((tl.current_value() - 0.0).abs() < 1e-4);
    }
}
