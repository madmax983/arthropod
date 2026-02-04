# Sentry's Journal

## 2024-05-22 - [Initial Audit]
**Learning:** Started fresh audit of `flux-state`.
**Action:** Adding comprehensive unit tests to `signal.rs`, `computed.rs`, and `effect.rs` to cover core reactive primitives.

## 2024-05-23 - [Reactive Graph Correctness]
**Learning:** Verified that `flux-state` correctly handles "Glitch Freedom" (diamond dependencies) and dynamic dependency pruning. The runtime correctly avoids unnecessary re-computations and executions.
**Action:** Used `AtomicUsize` counters to precisely measure execution counts in diamond patterns. This pattern is highly effective for verifying reactive graph behavior without relying on timing or race conditions.

## 2024-10-24 - [Panic Safety in Reactive Runtime]
**Learning:** Panics within an `Effect` closure were leaking into the `Runtime`'s thread-local `tracking_context`, causing subsequent unrelated signal reads to be incorrectly tracked as dependencies of the failed effect. This "zombie" dependency would cause a second panic when the unrelated signal was updated.
**Action:** Implemented a `ContextGuard` RAII wrapper in `Runtime` that ensures `pop_context` is always called, even during stack unwinding.

## 2024-10-24 - [Effect Drop Behavior]
**Learning:** `Effect::new` returns a handle that unsubscribes the effect when dropped. This is subtle and caused a test failure where an effect stopped working immediately because its return value was ignored.
**Action:** Always bind the result of `Effect::new` (e.g., `let _keep_alive = ...`) in tests or long-lived scopes to keep the subscription active.
