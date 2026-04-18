# 0040. Module Encapsulation (Facade Pattern)

Date: 2026-06-29

## Status

Proposed

## Context

Across multiple crates in the Arthropod workspace (such as `widget-core`, `arthropod-ecs`, `plat-core`, etc.), internal implementation modules (e.g., `form`, `layout`, `input_state`) were unnecessarily exposed as `pub mod`. This leaked internal directory structures to external consumer crates, created a sprawling API surface area, and violated the architectural directive to default to `pub(crate)` for internal boundaries. This lack of encapsulation led to high coupling, making internal refactoring difficult without causing breaking changes downstream.

## Decision

We restructured the internal modules of core crates by converting `pub mod` declarations to `pub(crate) mod` (or wrapping them in internal submodules). To maintain a clean, flat public API, we explicitly re-export only the necessary cross-crate APIs at the crate root via `pub use` statements. This enforces the **Facade pattern** at the crate boundary.

## Consequences

### Positive
- **Reduced Coupling**: External crates can no longer depend on deeply nested internal structures, decoupling them from internal implementation details.
- **Maintainability**: Internal architecture (like the division between `layout`, `primitive`, or `form` modules) can be refactored without triggering breaking changes downstream.
- **Cleaner API Surface**: The public API of each crate is explicitly curated in its `lib.rs`, reducing cognitive load for consumers and improving `cargo doc` output.

### Negative
- **Maintenance Overhead**: Maintainers must remember to explicitly add `pub use` statements at the crate root for any new functionality that needs to be accessible outside the crate.
