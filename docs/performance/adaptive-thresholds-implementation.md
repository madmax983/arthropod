# Adaptive Thresholds Implementation

## Overview

Implemented an adaptive threshold system that automatically tunes parallel execution thresholds based on observed frame performance. This system addresses the 5.4x performance regression discovered during parallelization (10K entities: 408 μs → 2.22 ms).

## Implementation Summary

### 1. Core Adaptive System (`crates/arthropod-ecs/src/adaptive.rs`)

**Key Components:**

- `ThresholdConfig`: Holds current threshold values
  - `reactive_parallel_threshold`: Min entities for gather-apply pattern (default: 25,000)
  - `render_parallel_threshold`: Min entities for parallel render collection (default: 1,000)
  - `text_parallel_threshold`: Min text nodes for parallel shaping (default: 8)

- `AdaptiveThresholds`: Monitors frame metrics and adjusts thresholds
  - Tracks 60-frame sliding window for efficiency metrics
  - Compares to pre-parallelization baseline (0.028 μs/entity)
  - Adjusts thresholds with hysteresis to prevent thrashing
  - 30-frame cooldown between adjustments

**Adaptation Strategy:**

```rust
// Efficiency ratio = observed / baseline
if efficiency_ratio > 2.0 {
    // 2x slower than baseline: increase thresholds 50% (use parallel less)
    reactive_threshold *= 1.5;
    render_threshold *= 1.3;
} else if efficiency_ratio > 1.5 {
    // 50% slower: increase thresholds 20%
    reactive_threshold *= 1.2;
    render_threshold *= 1.1;
} else if efficiency_ratio < 0.8 {
    // Better than baseline: decrease thresholds 10% (use parallel more)
    reactive_threshold *= 0.9;
    render_threshold *= 0.9;
}
```

**Safety Features:**

- Warmup period: No adjustments for first 60 frames
- Minimum data: Requires 10 frames before making decisions
- Threshold caps: reactive ≤ 100K, render ≤ 10K
- Minimum floors: reactive ≥ 1K, render ≥ 100

### 2. Feature Flag (`parallel-reactive`)

Added feature flag to make gather-apply pattern optional:

```toml
[features]
parallel-reactive = []  # High overhead, only beneficial at 25K+ entities
```

**Without feature (default):**
- Uses `update_all_reactive_system` (single-pass signal polling + scene update)
- Lower overhead for typical UI scales (1-10K entities)

**With feature:**
- Uses `gather_reactive_changes_system` + `apply_reactive_changes_system`
- Decouples signal polling from scene mutation
- Beneficial only at 25K+ entities based on crossover analysis

### 3. System Integration

**Updated Systems:**

1. **`collect_renderables_system`** (render.rs):
   - Reads `Res<AdaptiveThresholds>`
   - Checks `render_parallel_threshold` before using rayon
   - Threshold adjusts dynamically based on observed performance

2. **`build_frame_schedule`** (context.rs):
   - Conditionally uses gather-apply pattern based on feature flag
   - Imports systems conditionally to avoid unused warnings

3. **`FrameworkContext::update`** (context.rs):
   - Records frame timing and entity count
   - Feeds metrics to `AdaptiveThresholds` for adjustment
   - Happens automatically on every frame

## Test Coverage

**Unit Tests (8 tests in `adaptive.rs`):**

- `test_default_thresholds`: Verifies initial values
- `test_warmup_period_no_adjustment`: No changes during first 60 frames
- `test_increase_threshold_on_poor_efficiency`: Raises thresholds when slow
- `test_decrease_threshold_on_good_efficiency`: Lowers thresholds when fast
- `test_hysteresis_prevents_thrashing`: Cooldown prevents rapid changes
- `test_threshold_caps`: Maximum/minimum bounds enforced
- `test_sliding_window_maintains_size`: History capped at 60 frames
- `test_reset`: Reset to defaults (for testing)

**Integration:** All existing tests pass (97 total across arthropod-ecs + arthropod-mcp)

## Usage

### Default Behavior (Recommended)

```rust
// Create context - adaptive thresholds initialized automatically
let mut context = FrameworkContext::new();

// Update loop - thresholds adjust automatically
loop {
    context.update();  // Records metrics, adjusts thresholds
    let instances = context.render();
    backend.render_instances(&instances)?;
}
```

No code changes required. The system automatically:
- Monitors frame timing and entity counts
- Adjusts thresholds to minimize overhead
- Converges to optimal execution strategy within ~60 frames

### Enabling Gather-Apply Pattern (Advanced)

```toml
# In Cargo.toml
[dependencies]
arthropod-ecs = { version = "0.1", features = ["parallel-reactive"] }
```

Only enable if you have sustained workloads with 25K+ entities. For typical UIs (1-10K entities), the default (no feature) is faster.

### Inspecting Thresholds (Debugging)

```rust
use arthropod_ecs::AdaptiveThresholds;

// Read current thresholds from World resource
let thresholds = context.world().resource::<AdaptiveThresholds>();
let config = thresholds.current();

println!("Reactive threshold: {}", config.reactive_parallel_threshold);
println!("Render threshold: {}", config.render_parallel_threshold);
```

## Performance Impact

### Without Feature (Default - Recommended)

Based on crossover analysis:

- **1K entities**: ~544 μs (vs 2.27 ms with gather-apply) - **4.2x faster**
- **10K entities**: ~2.27 ms (vs 2.27 ms with gather-apply) - **Same**
- **25K entities**: ~6.87 ms (vs potentially faster with gather-apply)

**Recommendation:** Use default (no feature) for typical UIs up to 25K entities.

### With Feature + Adaptive Thresholds

Initial thresholds (conservative):
- Reactive: 25,000 entities (gather-apply disabled for most workloads)
- Render: 1,000 entities (parallel collection for medium+ scenes)

After 60-frame warmup:
- Thresholds adjust based on actual observed performance
- Converges to optimal execution strategy
- Overhead minimized while maintaining throughput

## Key Findings

1. **Gather-apply overhead too high at typical scales**
   - 2594% overhead at 1K entities
   - 709% overhead at 10K entities
   - Benefit only appears at 25K+ entities

2. **Feature flag approach optimal**
   - Simple path (no feature) for typical use cases
   - Advanced path (with feature) for extreme scale
   - No runtime cost for choosing wrong path

3. **Adaptive thresholds converge quickly**
   - 60-frame warmup period
   - 10-frame minimum for decisions
   - 30-frame cooldown prevents thrashing

4. **Render parallelization beneficial earlier**
   - Best efficiency at 10K entities (0.24 μs/entity)
   - Adaptive threshold starts at 1K (conservative)
   - Adjusts down if performance is good

## Future Work

1. **Profile-guided optimization**
   - Add instrumentation to measure overhead sources
   - Identify specific bottlenecks in gather-apply pattern
   - Optimize buffer allocation patterns

2. **Workload-specific thresholds**
   - Different thresholds for different entity archetypes
   - Reactive updates vs layout vs rendering
   - Per-system threshold tuning

3. **Explicit threshold control**
   - API for manually setting thresholds
   - Performance profiles (e.g., "low-latency", "high-throughput")
   - Runtime threshold inspection/control

4. **Extended metrics**
   - Track per-system timing
   - Correlate threshold adjustments with performance changes
   - Export metrics for external analysis

## References

- [Parallelization Analysis](./parallelization-analysis.md) - Root cause analysis
- [Crossover Analysis](./parallelization-baselines.md) - Benchmark results
- [ADR 0020: Parallelization Strategy](../adr/0020-parallelization-strategy.md)
