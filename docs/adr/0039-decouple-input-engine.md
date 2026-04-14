# 0039. Decouple Input Engine from Render Engine

Date: 2026-04-14

## Status

Accepted

## Context

The `input-engine` was previously coupled to the `render-engine` solely to use `render_engine::NodeId` as keys in its `IndexMap` and `HashMap` data structures. This created an unnecessary dependency loop where a foundational logical component was tied to the heavy rendering subsystem.

## Decision

We removed the dependency on `render-engine` from `input-engine` by introducing an independent identifier `input_engine::InputNodeId`. Internal state containers inside `input-engine` now use `InputNodeId` as keys instead of `NodeId`. The `WidgetContext` inside `widget-core` handles mapping and coordination between the two disparate ID spaces without exposing the `InputNodeId` type publicly.

## Consequences

### Positive
- **Reduced Coupling**: The `input-engine` is now a pure logic crate entirely independent of the `render-engine`.
- **Improved Build Times**: Less complex dependency graphs reduce potential build bottlenecks.

### Negative
- **Indirection**: A minor layer of mapping is now required in `widget-core` to resolve `NodeId` references to `InputNodeId` references when interacting with the `input-engine`.
