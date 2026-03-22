//! AnimationClock — drift-free two-component time model.
//!
//! Inspired by AletheiaDB's hybrid logical clock `(wallclock, logical)`,
//! the animation clock uses `(cycle: u64, phase: f32)` to eliminate
//! floating-point drift in looping animations.
//!
//! - `cycle` is an integer — exact after hours of looping.
//! - `phase` is 0.0–1.0 within the current cycle — error never compounds.
//! - Delta is clamped (self-healing) to gracefully handle tab backgrounding.

/// Maximum delta time per tick (100ms).
/// Larger deltas (tab backgrounding, debugger pauses) are clamped to this
/// value, preventing animations from jumping to completion on resume.
const MAX_DELTA_SECS: f32 = 0.1;

/// What happens at cycle boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackMode {
    /// Play once, then stop at phase 1.0.
    Once,
    /// Loop forever (cycle increments, phase resets).
    Loop,
    /// Alternate forward/reverse on each cycle.
    PingPong,
    /// Loop exactly `n` times, then stop.
    Count(u32),
}

/// Event emitted by [`AnimationClock::tick`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockEvent {
    /// Normal phase advance within a cycle.
    Normal,
    /// One or more cycle boundaries were crossed.
    CycleBoundary {
        /// Number of complete cycles crossed in this tick.
        completed: u64,
    },
}

/// Two-component animation clock: `cycle` (integer) + `phase` (0.0–1.0).
///
/// # Drift resistance
///
/// The `cycle` counter is a `u64` — no floating-point accumulation, ever.
/// `phase` stays within 0.0–1.0 and resets each cycle, so error never
/// compounds across loops.
///
/// # Self-healing
///
/// Delta time is clamped to [`MAX_DELTA_SECS`] to prevent time bombs when
/// the application resumes from a background state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationClock {
    /// Completed cycle count (monotonically increasing).
    cycle: u64,
    /// Position within current cycle, normalized to 0.0–1.0.
    phase: f32,
}

impl AnimationClock {
    /// Create a new clock at the beginning (cycle 0, phase 0.0).
    pub fn new() -> Self {
        Self {
            cycle: 0,
            phase: 0.0,
        }
    }

    /// Advance the clock by a real-time delta.
    ///
    /// `duration` is the total duration of one cycle in seconds.
    ///
    /// Returns a [`ClockEvent`] indicating whether a cycle boundary was crossed.
    /// The caller (typically [`Timeline`](crate::Timeline)) uses this event to
    /// decide loop behavior.
    pub fn tick(&mut self, delta_secs: f32, duration: f32) -> ClockEvent {
        debug_assert!(duration > 0.0, "Clock duration must be positive");

        // Self-healing: clamp delta to prevent time bombs
        let clamped = delta_secs.clamp(0.0, MAX_DELTA_SECS);
        let phase_advance = clamped / duration;

        let new_phase = self.phase + phase_advance;
        if new_phase >= 1.0 {
            let whole_cycles = new_phase as u64;
            self.cycle += whole_cycles;
            self.phase = new_phase.fract();
            // Handle exact 1.0 case (fract returns 0.0)
            if self.phase == 0.0 && new_phase > 0.0 {
                // Landed exactly on a boundary — phase stays at 0.0 (start of new cycle)
            }
            ClockEvent::CycleBoundary {
                completed: whole_cycles,
            }
        } else {
            self.phase = new_phase;
            ClockEvent::Normal
        }
    }

    /// Current phase within the cycle (0.0–1.0).
    pub fn phase(&self) -> f32 {
        self.phase
    }

    /// Current cycle count.
    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    /// Effective phase accounting for playback mode.
    ///
    /// For [`PlaybackMode::PingPong`], odd cycles evaluate in reverse.
    /// For all other modes, returns the raw phase.
    pub fn effective_phase(&self, mode: &PlaybackMode) -> f32 {
        match mode {
            PlaybackMode::PingPong if self.cycle % 2 == 1 => 1.0 - self.phase,
            _ => self.phase,
        }
    }

    /// Check if the clock has finished based on the playback mode.
    pub fn is_finished(&self, mode: &PlaybackMode) -> bool {
        match mode {
            PlaybackMode::Once => self.cycle >= 1,
            PlaybackMode::Count(n) => self.cycle >= u64::from(*n),
            PlaybackMode::Loop | PlaybackMode::PingPong => false,
        }
    }

    /// Reset to the beginning (cycle 0, phase 0.0).
    pub fn reset(&mut self) {
        self.cycle = 0;
        self.phase = 0.0;
    }
}

impl Default for AnimationClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_linear_advance() {
        let mut clock = AnimationClock::new();
        // 1 second duration, advance 0.05s (well under MAX_DELTA)
        let event = clock.tick(0.05, 1.0);
        assert_eq!(event, ClockEvent::Normal);
        assert!((clock.phase() - 0.05).abs() < 1e-6);
        assert_eq!(clock.cycle(), 0);
    }

    #[test]
    fn clock_completes_once() {
        let mut clock = AnimationClock::new();
        // Duration 0.05s, tick 0.05s — completes exactly one cycle within MAX_DELTA
        let event = clock.tick(0.05, 0.05);
        assert!(matches!(event, ClockEvent::CycleBoundary { completed: 1 }));
        assert!(clock.is_finished(&PlaybackMode::Once));
    }

    #[test]
    fn clock_loop_wraps() {
        let mut clock = AnimationClock::new();
        // Advance past one full cycle
        let event = clock.tick(0.1, 0.08); // 0.1 / 0.08 = 1.25 cycles
        assert!(matches!(event, ClockEvent::CycleBoundary { .. }));
        assert_eq!(clock.cycle(), 1);
        assert!((clock.phase() - 0.25).abs() < 1e-6);
        // Loop mode is never finished
        assert!(!clock.is_finished(&PlaybackMode::Loop));
    }

    #[test]
    fn clock_ping_pong_reverses() {
        let mut clock = AnimationClock::new();
        // First cycle: normal direction
        clock.tick(0.05, 0.1); // phase = 0.5
        assert!((clock.effective_phase(&PlaybackMode::PingPong) - 0.5).abs() < 1e-6);

        // Cross into second cycle
        clock.tick(0.08, 0.1); // phase wraps, now in cycle 1
        assert_eq!(clock.cycle(), 1);
        // Odd cycle: reversed
        let eff = clock.effective_phase(&PlaybackMode::PingPong);
        assert!(
            eff > 0.5,
            "PingPong odd cycle should reverse: effective_phase={eff}"
        );
    }

    #[test]
    fn clock_count_n_finishes_after_n_cycles() {
        let mut clock = AnimationClock::new();
        let mode = PlaybackMode::Count(3);

        // 3 cycles
        for _ in 0..3 {
            clock.tick(0.1, 0.1);
        }
        assert!(clock.is_finished(&mode));
    }

    #[test]
    fn clock_delta_clamping() {
        let mut clock = AnimationClock::new();
        // 5 seconds delta (simulates tab backgrounding), 1 second duration
        // Should be clamped to MAX_DELTA_SECS (0.1s)
        let _event = clock.tick(5.0, 1.0);
        // Phase should be ~0.1, not 5.0
        assert!(
            clock.phase() <= MAX_DELTA_SECS + 0.01,
            "Delta should be clamped: phase={}",
            clock.phase()
        );
    }

    #[test]
    fn clock_cycle_boundary_event() {
        let mut clock = AnimationClock::new();
        // Advance exactly one cycle
        let event = clock.tick(0.1, 0.1);
        match event {
            ClockEvent::CycleBoundary { completed } => {
                assert_eq!(completed, 1);
            }
            ClockEvent::Normal => panic!("Expected CycleBoundary"),
        }
    }

    #[test]
    fn clock_zero_delta_no_change() {
        let mut clock = AnimationClock::new();
        let event = clock.tick(0.0, 1.0);
        assert_eq!(event, ClockEvent::Normal);
        assert_eq!(clock.phase(), 0.0);
        assert_eq!(clock.cycle(), 0);
    }

    #[test]
    fn clock_reset() {
        let mut clock = AnimationClock::new();
        clock.tick(0.5, 1.0);
        assert!(clock.phase() > 0.0);
        clock.reset();
        assert_eq!(clock.phase(), 0.0);
        assert_eq!(clock.cycle(), 0);
    }

    #[test]
    fn clock_negative_delta_treated_as_zero() {
        let mut clock = AnimationClock::new();
        let event = clock.tick(-1.0, 1.0);
        assert_eq!(event, ClockEvent::Normal);
        assert_eq!(clock.phase(), 0.0);
    }
}
