# ADR 0024: Single-Pass Reactive Updates (YAGNI)

**Status:** Accepted
**Date:** 2026-02-21
**Deciders:** Architecture Team
**Context:** ADR 0020 (Parallelization Strategy), ADR 0003 (Flux State)

## Context

ADR 0020 outlined a 9-phase strategy for parallelizing the ECS pipeline. Phase 4 specifically proposed splitting reactive updates into `gather_reactive_changes_system` (parallel polling, no scene access) and `apply_reactive_changes_system` (sequential application to scene).

This architecture was implemented behind the `parallel-reactive` feature flag. The goal was to overlap signal polling with other systems that only need read-access to the Scene.

However, subsequent analysis and practical usage have revealed:
1.  **Complexity Overhead**: The gather-apply pattern requires buffering changes into an intermediate resource (`ReactiveChangeBuffer`), increasing memory traffic and allocation overhead.
2.  **Insufficient Gain**: At current and near-term projected entity counts (< 50k), the overhead of task scheduling and buffering outweighs the benefits of parallel signal polling. Reactive updates are typically sparse (only a small subset of signals change per frame), making the "dense" parallel iteration less effective.
3.  **YAGNI**: The complexity of managing two distinct pipelines (single-pass vs gather-apply) adds maintenance burden without providing tangible performance benefits for the majority of use cases.

## Decision

We decide to:
1.  **Standardize on Single-Pass Updates**: The `update_all_reactive_system` (introduced in ADR 0020 Phase 1) will be the standard, default implementation for reactive updates.
2.  **Deprecate Parallel-Reactive**: The `parallel-reactive` feature and the associated gather-apply systems (`gather_reactive_changes_system`, `apply_reactive_changes_system`) are considered deprecated and will be removed in a future cleanup pass.
3.  **Reject Phase 4**: We explicitly reject the move to Phase 4 (Gather-Apply) as the default architecture.

## Consequences

### Positive
-   **Simplicity**: The reactive update pipeline remains a simple, single-pass iteration over queries.
-   **Debuggability**: Single-threaded execution is easier to trace and debug.
-   **Reduced Overhead**: Eliminates the allocation and copying cost of the intermediate change buffer.

### Negative
-   **Parallelism Limit**: We accept that reactive updates will run serially on the main thread. This sets a hard ceiling on the number of *changing* reactive components we can handle per frame, but benchmarks suggest this ceiling is sufficiently high (> 100k simple updates at 60fps) for our target use cases.

## References
-   ADR 0020: ECS Parallelization Strategy (Phase 4 superseded by this ADR)
-   `crates/arthropod-ecs/src/systems/reactive.rs`: Implementation of `update_all_reactive_system`
