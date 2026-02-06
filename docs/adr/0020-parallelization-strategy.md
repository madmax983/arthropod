# ADR 0020: ECS Parallelization Strategy

**Status:** Accepted
**Date:** 2026-02-06
**Decision Makers:** Mark
**Context:** ADR 0001 (Hybrid ECS), ADR 0002 (bevy_ecs Selection)

## Context

Arthropod's ECS pipeline is currently 100% single-threaded at runtime. The frame pipeline — reactive updates, layout, a11y sync, render collection — executes sequentially on one thread. bevy_ecs 0.15 supports multi-threaded scheduling, but `multi_threaded` is not enabled, and all update systems contend on `ResMut<Scene>`, forcing sequential execution even if the scheduler were parallel.

### Current Performance (Pre-Parallelization Baselines)

| Entities | ECS Update | ECS Render | Full Frame |
|----------|------------|------------|------------|
| 1,000    | 20.2 us    | 9.5 us     | 30.4 us    |
| 10,000   | 280 us     | 128 us     | ~408 us    |

At 10K entities, the full ECS frame consumes ~2.4% of the 60fps budget. While acceptable today, scaling to 50K+ entities for enterprise dashboards will become a bottleneck.

### Problem Statement

1. All three reactive systems (`update_reactive_colors_system`, `update_reactive_transforms_system`, `update_reactive_opacity_system`) acquire `ResMut<Scene>` exclusively, preventing any parallel scheduling.
2. `collect_renderables_system` runs in a separate schedule, missing overlap opportunities with `sync_accessible_nodes_system`.
3. No rayon-based data parallelism is used for render instance collection or signal polling.

## Decision

Implement parallelization in 9 phases (0-8), ordered by risk and dependency:

### Phase 0: Baselines + ADR (This Document)
Establish reproducible benchmark baselines with per-system breakdowns.

### Phase 1: Merge Reactive Systems
Combine color/transform/opacity systems into `update_all_reactive_system`, reducing 3 `ResMut<Scene>` acquisitions to 1 and eliminating scheduling overhead.

### Phase 2: Enable `multi_threaded` + System Ordering
Enable bevy_ecs `multi_threaded` feature across all crates. Add explicit `.after()` ordering constraints that currently happen implicitly via `ResMut<Scene>` contention.

### Phase 3: Overlap A11y Sync with Render Collection
Merge render_schedule into update_schedule. Both `sync_accessible_nodes_system` (`Res<Scene>`, `ResMut<A11yTree>`) and `collect_renderables_system` (`Res<Scene>`, `ResMut<RenderCommands>`) use shared Scene access with disjoint mutable resources, enabling parallel execution.

### Phase 4: Gather-Apply for Reactive Systems
Split reactive updates into `gather_reactive_changes_system` (polls signals, no Scene access) and `apply_reactive_changes_system` (writes Scene, no signal access). Uses `ReactiveChangeBuffer` as an intermediate resource. Adds `rayon` dependency.

### Phase 5: Parallel Render Instance Collection
Use `rayon::par_iter()` in `collect_renderables_system` for scenes with >256 nodes. `create_rect_instance(&SceneNode)` is stateless and safe for parallel execution.

### Phase 6: Parallel A11y Sync (Gather-Apply)
Apply the same gather-apply pattern to a11y synchronization with `A11yBoundsBuffer`.

### Phase 7: Text Shaping Parallelization
Parallelize cosmic-text shaping using thread-local `FontSystem` instances via rayon.

### Phase 8: Layout Parallelization
Parallelize independent subtree layout computation. Highest risk due to Taffy's sequential tree traversal.

## Consequences

### Positive
- **30-50% reduction** in full frame ECS overhead at 10K entities (Phases 0-6)
- **Linear scaling** preserved with better constant factors
- **Foundation for 50K+ entities** in enterprise dashboards
- **No API breaking changes** through Phase 6 (internal refactoring only)
- **Phase 3** provides first real parallel win with minimal risk

### Negative
- **Increased complexity** in system scheduling (ordering constraints must be explicit)
- **rayon dependency** adds ~50KB to binary size (already transitively via criterion)
- **Thread pool initialization** adds ~1ms to startup
- **Phase 7-8** carry high risk and may not provide proportional benefit

### Risks
- Phase 7: Multiple `FontSystem` instances consume memory; cosmic-text thread safety unclear
- Phase 8: Taffy not designed for parallel execution; most real layouts have deep interconnections
- All phases: Must verify deterministic output after parallelization

## Verification

After each phase:
1. `cargo test --all` passes
2. `cargo clippy --all-targets --all-features -- -D warnings` passes (excluding pre-existing plat-core issues)
3. `cargo fmt --all -- --check` passes
4. `cargo bench -p arthropod-ecs` shows no single-threaded regression > 5%
5. `cargo run --example colored_rectangles` visually correct

## References
- [bevy_ecs multi_threaded documentation](https://docs.rs/bevy_ecs/0.15/bevy_ecs/)
- [rayon parallel iterators](https://docs.rs/rayon/1.11/)
- ADR 0001: Hybrid ECS Architecture
- ADR 0002: bevy_ecs Selection
