# ADR 0019: Theme Engine Simplification

**Status:** Accepted

**Date:** 2026-05-25

**Deciders:** Architecture Team, Atlas

## Context

The `theme-engine` crate was originally designed (in [ADR 0009](0009-three-layer-theming-architecture.md)) to follow a strict three-layer architecture. Layer 1 (Platform Native Defaults) in `plat-core` was intended to expose a `BackgroundMaterial` enum, and Layer 2 (`theme-engine`) was designed to consume this via a "bridge" or mapping layer, often duplicating the enum to strictly decouple the semantic tokens from the platform implementation.

During implementation, this redundancy proved to be unnecessary boilerplate. The `theme-engine` crate refactoring eliminated the `material_bridge` module and the redundant `BackgroundMaterial` enum. Instead, it now directly uses `plat_core::BackdropMaterial` (renamed from `BackgroundMaterial`) for window background effects.

This structural change occurred without a corresponding ADR, leading to a discrepancy between the architectural documentation and the actual codebase.

## Decision

We formally accept the simplification of the `theme-engine` architecture:

1.  **Direct Type Usage**: `theme-engine` shall directly import and use `plat_core::BackdropMaterial` within its `TokenValue` enum.
2.  **Elimination of Bridge**: The `material_bridge` module and any redundant `BackgroundMaterial` definitions in `theme-engine` are removed.
3.  **Naming Alignment**: We acknowledge and accept the rename of the core type from `BackgroundMaterial` to `BackdropMaterial` in `plat-core`.
4.  **Re-export**: `theme-engine` is permitted to re-export `BackdropMaterial` from `plat-core` to provide a unified import surface for consumers.

## Consequences

### Positive

-   **Reduced Boilerplate**: Eliminates the need to maintain two identical enums and a conversion layer.
-   **Maintenance Efficiency**: Adding a new material (e.g., for a new OS version) only requires changes in `plat-core`, automatically becoming available to `theme-engine`.
-   **Performance**: Removes a trivial conversion step at runtime/startup.

### Negative

-   **Tighter Coupling**: `theme-engine` now exposes a type directly from `plat-core`. Breaking changes in `plat-core::BackdropMaterial` will strictly break `theme-engine`'s public API. This is considered an acceptable trade-off as both crates reside in the "Foundation Layer" and are versioned together.

## References

-   [ADR 0009: Three-Layer Theming Architecture](0009-three-layer-theming-architecture.md)
-   `crates/plat-core/src/materials.rs`
-   `crates/theme-engine/src/design_tokens.rs`
