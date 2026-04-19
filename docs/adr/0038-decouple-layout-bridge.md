# 0038. Decouple Layout Bridge from ECS

Date: 2026-04-07

## Status

Accepted

## Context

The `layout_bridge` module interfaces `render-engine` with `layout-engine` and has no dependencies on the ECS layer. Keeping it in `arthropod-ecs` created unnecessary coupling, forcing non-ECS layout logic to depend on the ECS crate.

## Decision

We moved the `layout_bridge` from `arthropod-ecs` to `widget-core`.

## Consequences

### Positive
- Improved cohesion.
- Both `arthropod` and `arthropod-ecs` can use the bridge without `arthropod` needing to depend on `arthropod-ecs` for non-ECS layout logic.
- This reduces coupling and maintains a clean dependency graph.

### Negative
- Code that historically imported the layout bridge from `arthropod-ecs` must be updated to import it from `widget-core`.