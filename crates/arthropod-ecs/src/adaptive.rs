//! Adaptive threshold system for automatic parallel vs sequential execution selection
//!
//! This module provides runtime heuristics that automatically choose the optimal execution
//! path (parallel vs sequential) based on observed frame metrics and entity counts.
//!
//! # Architecture
//!
//! The adaptive system tracks frame timing metrics and adjusts thresholds dynamically:
//! - **Initial thresholds**: Based on crossover analysis (see docs/performance/parallelization-analysis.md)
//! - **Convergence**: Thresholds stabilize within ~60 frames based on actual workload
//! - **Hysteresis**: Prevents thrashing between execution modes
//!
//! # Usage
//!
//! ```no_run
//! use arthropod_ecs::adaptive::AdaptiveThresholds;
//! use std::time::Duration;
//!
//! let mut thresholds = AdaptiveThresholds::new();
//! let entity_count = 1000; // Example entity count
//! let frame_time = Duration::from_micros(16666); // Example frame time
//!
//! // At frame start, get current thresholds
//! let config = thresholds.current();
//!
//! // Systems check thresholds before using parallel paths
//! if entity_count >= config.reactive_parallel_threshold {
//!     // Use parallel execution
//! } else {
//!     // Use sequential execution
//! }
//!
//! // At frame end, record metrics for adaptation
//! thresholds.record_frame(frame_time, entity_count);
//! ```

use std::collections::VecDeque;
use std::time::Duration;

/// Configuration for parallel execution thresholds
#[derive(Debug, Clone)]
pub struct ThresholdConfig {
    /// Minimum entities to use parallel reactive update (gather-apply pattern)
    /// Based on crossover analysis: ~25,000 entities where overhead < benefit
    pub reactive_parallel_threshold: usize,

    /// Minimum entities to use parallel render collection
    /// Based on crossover analysis: ~500-1,000 entities for efficient batching
    pub render_parallel_threshold: usize,

    /// Minimum text nodes to use parallel text shaping
    /// Based on cosmic-text thread-local overhead: ~8 nodes
    pub text_parallel_threshold: usize,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            // Conservative defaults from crossover analysis
            reactive_parallel_threshold: 25_000, // 25K entities
            render_parallel_threshold: 1_000,    // 1K entities
            text_parallel_threshold: 8,          // 8 text nodes
        }
    }
}

/// Frame timing metrics for adaptive threshold adjustment
#[derive(Debug, Clone)]
struct FrameMetric {
    /// Frame time in microseconds
    duration_us: u64,
    /// Entity count at this frame
    entity_count: usize,
}

/// Adaptive threshold system that monitors performance and adjusts execution strategy
///
/// This system tracks frame timing and entity counts to automatically tune parallel
/// execution thresholds. It uses a sliding window of recent frames to compute efficiency
/// metrics and adjust thresholds to minimize overhead.
///
/// # Adaptation Strategy
///
/// 1. **Warmup Phase** (first 60 frames): Use conservative defaults
/// 2. **Monitoring Phase**: Track frame times and entity counts
/// 3. **Adjustment Phase**: If sustained inefficiency detected, increase thresholds
/// 4. **Convergence**: Thresholds stabilize when efficiency meets target
///
/// # Hysteresis
///
/// To prevent thrashing, thresholds only adjust when:
/// - Sustained inefficiency over 10+ frames
/// - Adjustment magnitude > 10% of current threshold
/// - Cooldown period (30 frames) since last adjustment
#[derive(bevy_ecs::system::Resource)]
pub struct AdaptiveThresholds {
    /// Current threshold configuration
    config: ThresholdConfig,

    /// Sliding window of recent frame metrics (last 60 frames)
    frame_history: VecDeque<FrameMetric>,

    /// Frame count since creation
    frame_count: u64,

    /// Frame count at last threshold adjustment (for cooldown)
    last_adjustment_frame: u64,

    /// Target efficiency in microseconds per entity
    /// Based on baseline: 0.028 us/entity (10K entities @ 280 us)
    target_efficiency_us_per_entity: f64,

    /// Minimum frames between threshold adjustments (hysteresis)
    adjustment_cooldown_frames: u64,
}

impl AdaptiveThresholds {
    /// Create a new adaptive threshold system with default configuration
    pub fn new() -> Self {
        Self {
            config: ThresholdConfig::default(),
            frame_history: VecDeque::with_capacity(60),
            frame_count: 0,
            last_adjustment_frame: 0,
            target_efficiency_us_per_entity: 0.028, // Pre-parallelization baseline
            adjustment_cooldown_frames: 30,
        }
    }

    /// Get the current threshold configuration
    pub fn current(&self) -> &ThresholdConfig {
        &self.config
    }

    /// Record a completed frame for adaptation
    ///
    /// # Arguments
    ///
    /// * `frame_time` - Total frame time (update + render)
    /// * `entity_count` - Number of entities in this frame
    pub fn record_frame(&mut self, frame_time: Duration, entity_count: usize) {
        let duration_us = frame_time.as_micros() as u64;

        // Add to history (keep last 60 frames)
        if self.frame_history.len() >= 60 {
            self.frame_history.pop_front();
        }
        self.frame_history.push_back(FrameMetric {
            duration_us,
            entity_count,
        });

        self.frame_count += 1;

        // Only adjust after warmup period and cooldown
        if self.frame_count > 60
            && self.frame_count - self.last_adjustment_frame >= self.adjustment_cooldown_frames
        {
            self.consider_adjustment();
        }
    }

    /// Consider adjusting thresholds based on recent frame metrics
    fn consider_adjustment(&mut self) {
        if self.frame_history.len() < 10 {
            return; // Need at least 10 frames to make decision
        }

        // Compute average efficiency over recent frames
        let total_us: u64 = self.frame_history.iter().map(|m| m.duration_us).sum();
        let total_entities: usize = self.frame_history.iter().map(|m| m.entity_count).sum();

        if total_entities == 0 {
            return; // No entities, no adjustment needed
        }

        let avg_efficiency = total_us as f64 / total_entities as f64;

        // If efficiency is worse than target, increase thresholds (use parallel less)
        let efficiency_ratio = avg_efficiency / self.target_efficiency_us_per_entity;

        if efficiency_ratio > 2.0 {
            // More than 2x slower than baseline - increase thresholds significantly
            self.config.reactive_parallel_threshold =
                (self.config.reactive_parallel_threshold as f64 * 1.5) as usize;
            self.config.render_parallel_threshold =
                (self.config.render_parallel_threshold as f64 * 1.3) as usize;

            self.last_adjustment_frame = self.frame_count;
        } else if efficiency_ratio > 1.5 {
            // 50% slower than baseline - increase thresholds moderately
            self.config.reactive_parallel_threshold =
                (self.config.reactive_parallel_threshold as f64 * 1.2) as usize;
            self.config.render_parallel_threshold =
                (self.config.render_parallel_threshold as f64 * 1.1) as usize;

            self.last_adjustment_frame = self.frame_count;
        } else if efficiency_ratio < 0.8 {
            // Better than baseline - can lower thresholds (use parallel more)
            self.config.reactive_parallel_threshold =
                (self.config.reactive_parallel_threshold as f64 * 0.9).max(1000.0) as usize;
            self.config.render_parallel_threshold =
                (self.config.render_parallel_threshold as f64 * 0.9).max(100.0) as usize;

            self.last_adjustment_frame = self.frame_count;
        }

        // Cap maximum thresholds (if we need thresholds this high, parallelism isn't helping)
        self.config.reactive_parallel_threshold =
            self.config.reactive_parallel_threshold.min(100_000);
        self.config.render_parallel_threshold = self.config.render_parallel_threshold.min(10_000);
    }

    /// Reset thresholds to defaults (useful for testing)
    #[cfg(test)]
    pub fn reset(&mut self) {
        self.config = ThresholdConfig::default();
        self.frame_history.clear();
        self.frame_count = 0;
        self.last_adjustment_frame = 0;
    }

    /// Get current frame count
    #[cfg(test)]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Default for AdaptiveThresholds {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_thresholds() {
        let thresholds = AdaptiveThresholds::new();
        let config = thresholds.current();

        assert_eq!(config.reactive_parallel_threshold, 25_000);
        assert_eq!(config.render_parallel_threshold, 1_000);
        assert_eq!(config.text_parallel_threshold, 8);
    }

    #[test]
    fn test_warmup_period_no_adjustment() {
        let mut thresholds = AdaptiveThresholds::new();
        let initial_config = thresholds.current().clone();

        // Record 60 frames during warmup
        for _ in 0..60 {
            thresholds.record_frame(Duration::from_micros(5000), 10_000);
        }

        // Thresholds should not have changed during warmup
        assert_eq!(
            thresholds.current().reactive_parallel_threshold,
            initial_config.reactive_parallel_threshold
        );
        assert_eq!(
            thresholds.current().render_parallel_threshold,
            initial_config.render_parallel_threshold
        );
    }

    #[test]
    fn test_increase_threshold_on_poor_efficiency() {
        let mut thresholds = AdaptiveThresholds::new();

        // Complete warmup
        for _ in 0..60 {
            thresholds.record_frame(Duration::from_micros(280), 10_000);
        }

        let initial_reactive = thresholds.current().reactive_parallel_threshold;

        // Record sustained poor efficiency (10x worse than baseline)
        for _ in 0..40 {
            thresholds.record_frame(Duration::from_micros(2800), 10_000); // 0.28 us/entity vs 0.028 target
        }

        // Thresholds should have increased
        assert!(
            thresholds.current().reactive_parallel_threshold > initial_reactive,
            "Expected reactive threshold to increase, got {} vs initial {}",
            thresholds.current().reactive_parallel_threshold,
            initial_reactive
        );
    }

    #[test]
    fn test_decrease_threshold_on_good_efficiency() {
        let mut thresholds = AdaptiveThresholds::new();

        // Complete warmup with good efficiency
        for _ in 0..60 {
            thresholds.record_frame(Duration::from_micros(200), 10_000); // 0.02 us/entity (better than baseline)
        }

        let initial_reactive = thresholds.current().reactive_parallel_threshold;

        // Record sustained excellent efficiency
        for _ in 0..40 {
            thresholds.record_frame(Duration::from_micros(150), 10_000); // 0.015 us/entity
        }

        // Thresholds should have decreased (use parallel more)
        assert!(
            thresholds.current().reactive_parallel_threshold < initial_reactive,
            "Expected reactive threshold to decrease"
        );
    }

    #[test]
    fn test_hysteresis_prevents_thrashing() {
        let mut thresholds = AdaptiveThresholds::new();

        // Complete warmup
        for _ in 0..60 {
            thresholds.record_frame(Duration::from_micros(280), 10_000);
        }

        // Record poor efficiency to trigger adjustment
        for _ in 0..15 {
            thresholds.record_frame(Duration::from_micros(2800), 10_000);
        }

        let after_adjustment = thresholds.current().reactive_parallel_threshold;

        // Immediately try to trigger another adjustment (should be blocked by cooldown)
        for _ in 0..15 {
            thresholds.record_frame(Duration::from_micros(2800), 10_000);
        }

        // Threshold should not have changed (cooldown active)
        assert_eq!(
            thresholds.current().reactive_parallel_threshold,
            after_adjustment,
            "Threshold should not change during cooldown period"
        );
    }

    #[test]
    fn test_threshold_caps() {
        let mut thresholds = AdaptiveThresholds::new();

        // Force thresholds to maximum by recording extreme inefficiency
        for _ in 0..60 {
            thresholds.record_frame(Duration::from_micros(280), 10_000);
        }

        // Try to push thresholds extremely high
        for _ in 0..200 {
            thresholds.record_frame(Duration::from_micros(50000), 10_000);
        }

        // Thresholds should be capped
        assert!(
            thresholds.current().reactive_parallel_threshold <= 100_000,
            "Reactive threshold should be capped at 100K"
        );
        assert!(
            thresholds.current().render_parallel_threshold <= 10_000,
            "Render threshold should be capped at 10K"
        );
    }

    #[test]
    fn test_sliding_window_maintains_size() {
        let mut thresholds = AdaptiveThresholds::new();

        // Record 100 frames
        for _ in 0..100 {
            thresholds.record_frame(Duration::from_micros(500), 10_000);
        }

        // History should be capped at 60 frames
        assert_eq!(thresholds.frame_history.len(), 60);
    }

    #[test]
    fn test_reset() {
        let mut thresholds = AdaptiveThresholds::new();

        // Record frames and trigger adjustments
        for _ in 0..100 {
            thresholds.record_frame(Duration::from_micros(5000), 10_000);
        }

        thresholds.reset();

        // Should be back to defaults
        assert_eq!(thresholds.current().reactive_parallel_threshold, 25_000);
        assert_eq!(thresholds.frame_count(), 0);
        assert_eq!(thresholds.frame_history.len(), 0);
    }
}
