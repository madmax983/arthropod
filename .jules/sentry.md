# Sentry's Journal

## 2024-05-22 - [Initial Audit]
**Learning:** Started fresh audit of `flux-state`.
**Action:** Adding comprehensive unit tests to `signal.rs`, `computed.rs`, and `effect.rs` to cover core reactive primitives.

## 2024-05-23 - [Reactive Graph Correctness]
**Learning:** Verified that `flux-state` correctly handles "Glitch Freedom" (diamond dependencies) and dynamic dependency pruning. The runtime correctly avoids unnecessary re-computations and executions.
**Action:** Used `AtomicUsize` counters to precisely measure execution counts in diamond patterns. This pattern is highly effective for verifying reactive graph behavior without relying on timing or race conditions.
