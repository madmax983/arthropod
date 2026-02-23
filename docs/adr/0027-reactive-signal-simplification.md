# ADR 0027: Reactive Signal Simplification

**Status:** Accepted

**Date:** 2026-05-20

**Deciders:** Architecture Team

## Context

The initial implementation of `flux-state` (ADR 0003) relied on `Rc<RefCell<T>>` for signal storage to minimize overhead in a single-threaded environment. However, the ECS architecture (ADR 0001) required components to be `Send + Sync` to allow for potential parallel system execution.

To bridge this gap, a wrapper type called `MainThreadSignal<T>` was introduced. This wrapper:
1.  Held a `ReadSignal<T>` (which was `!Send` and `!Sync`).
2.  Implemented `unsafe impl Send for MainThreadSignal<T>` and `unsafe impl Sync for MainThreadSignal<T>`.
3.  Relied on a strict contract that the signal would *only* be accessed on the main thread, despite the type system claiming otherwise.

This approach was fragile. It introduced "fake" thread safety that could lead to Undefined Behavior if a system accidentally accessed the signal from a worker thread. It also added unnecessary boilerplate (`.get()` vs `.val()`) and confusion about when to use which type.

## Decision

We have removed the `MainThreadSignal` wrapper entirely in favor of using `flux_state::ReadSignal<T>` directly.

This was made possible by re-architecting `flux-state` to use a thread-safe internal runtime (see ADR 0015 context):
1.  **Internal Mutex**: The `Runtime` now wraps its state in `Arc<Mutex<RuntimeInner>>`.
2.  **Thread-Safe Signals**: `ReadSignal<T>` now holds an `Arc` to the runtime and the signal ID. Since the runtime is thread-safe, `ReadSignal<T>` is automatically `Send + Sync` (provided `T: Send + Sync`).
3.  **Cross-Thread Access**: Signals can now be safely read from any thread (e.g., background networking, physics integration) without fear of data races.

## Consequences

### Positive

-   **Simplicity**: One signal type (`ReadSignal<T>`) for both UI logic and ECS components.
-   **Safety**: Removed `unsafe impl` blocks. The Rust compiler now correctly enforces thread safety based on the inner type `T`.
-   **Concurrency**: Signals can be legitimately shared and accessed across threads, enabling true multi-threaded reactive systems if needed.
-   **Reduced Boilerplate**: No need to wrap/unwrap signals or use specialized accessors.

### Negative

-   **Locking Overhead**: Accessing a signal now involves acquiring a mutex (coarse-grained per runtime operation). This is slower than `RefCell` borrow checking but necessary for thread safety.
-   **Deadlock Risks**: While the runtime mitigates simple recursion, complex cross-thread dependency cycles could potentially deadlock if not managed carefully (though `flux-state` includes cycle detection).

### Compliance

-   **Deprecation**: This ADR officially deprecates and confirms the removal of `MainThreadSignal`.
-   **Migration**: All existing code using `MainThreadSignal` must be refactored to use `ReadSignal` directly.

## References

-   ADR 0003: Flux-State Reactive Model
-   ADR 0015: Flux-State Internal Architecture
-   Commit: `refactor(arthropod-ecs): remove MainThreadSignal wrapper`
