# Parallelization Baselines

**Date:** 2026-02-06
**ADR:** 0020 (Parallelization Strategy)
**Branch:** trunk (pre-parallelization)

## Pre-Parallelization Baselines

These baselines are recorded from the single-threaded pipeline before any parallelization work.
Run benchmarks with: `cargo bench -p arthropod-ecs`

### Existing Baselines (from benchmark-results.md)

| Benchmark | 10 | 100 | 1,000 | 10,000 |
|-----------|-----|------|--------|---------|
| ecs_update | 231 ns | 1.96 us | 20.2 us | 280 us |
| ecs_render | 151 ns | 890 ns | 9.5 us | 128 us |
| full_frame | 385 ns | 2.96 us | 30.4 us | ~408 us |

### New Per-System Breakdown Benchmarks

These benchmarks are added in Phase 0. Run after each phase to track improvements.

| Benchmark | 100 | 1,000 | 10,000 | 50,000 |
|-----------|------|--------|---------|---------|
| per_system/reactive_color | TBD | TBD | TBD | TBD |
| per_system/all_reactive | TBD | TBD | TBD | TBD |
| per_system/render_collection | - | TBD | TBD | TBD |
| per_system/full_frame_high | - | - | TBD | TBD |

### Phase Tracking

Record results after each phase completion:

| Phase | full_frame/10K | full_frame/50K | Delta vs Baseline |
|-------|---------------|----------------|-------------------|
| 0 (baseline) | TBD | TBD | - |
| 1 (merge reactive) | | | |
| 2 (multi_threaded) | | | |
| 3 (overlap a11y+render) | | | |
| 4 (gather-apply) | | | |
| 5 (parallel render) | | | |
| 6 (parallel a11y) | | | |
| 7 (text shaping) | | | |
| 8 (layout parallel) | | | |

## How to Record

```bash
# Run all benchmarks
cargo bench -p arthropod-ecs

# View HTML report
# target/criterion/report/index.html

# Compare against previous baseline
cargo bench -p arthropod-ecs -- --baseline phase0
```
