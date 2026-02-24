# ADR 0029: Experimental Feature Strategy ('Nova')

**Status:** Accepted

**Date:** 2024-05-21

**Deciders:** Core Team, Codex

## Context

The Arthropod framework is rapidly expanding with new, advanced capabilities such as the Nova Story Engine, Particle Systems, Flux Radar, and Time Travel debugging tools. These features are significant in size and complexity but are not yet stable enough for general production use. Including them in the default build increases compile times and binary size for users who do not need them.

Furthermore, we need a mechanism to incubate these features ("Nova" initiatives) without destabilizing the core crate or confusing users about which APIs are production-ready.

## Decision

We will introduce a `nova` feature flag in the `arthropod` crate to manage these experimental subsystems.

### 1. Conditional Compilation

The `arthropod::experimental` module and its sub-modules (e.g., `story`, `particles`, `flux_radar`, `chronos`, `elastic`) will be guarded by `#[cfg(feature = "nova")]`.

### 2. Stubbed APIs for DX

When the `nova` feature is disabled, the public API surfaces of these modules will remain available as stubs but marked with `#[deprecated]` and `#[allow(deprecated)]`.

*   **Function Stubs**: Will panic or log errors with instructions to enable the feature in `Cargo.toml`.
*   **Struct Stubs**: Will be empty or hidden, preventing compilation of usage code but providing a clear deprecation message instead of a generic "unresolved import" error.

This approach provides a superior Developer Experience (DX) by guiding users directly to the solution when they attempt to use an experimental feature without the flag.

### 3. Namespace Isolation

All such features reside strictly within the `arthropod::experimental` namespace. This explicit path signals to users that the API is subject to breaking changes.

## Consequences

### Positive

-   **Reduced Bloat**: Default builds are leaner and faster, excluding heavy subsystems like the Story Engine or Flux Radar.
-   **Stability**: Core stability is preserved while experimental features iterate rapidly.
-   **Discoverability**: The `experimental` namespace makes it clear which APIs are unstable.
-   **Guidance**: The stub pattern transforms compiler errors into helpful instructions.

### Negative

-   **Maintenance Overhead**: Requires maintaining stub implementations for the disabled state, doubling the API surface maintenance for experimental modules.
-   **Configuration Complexity**: Users must explicitly opt-in to `features = ["nova"]` to access these tools.

## References

-   `crates/arthropod/src/experimental/mod.rs`
-   `crates/arthropod/Cargo.toml`
