# ADR 0015: Flux-State Internal Architecture and Thread Safety

**Status:** Accepted

**Date:** 2026-01-22

**Deciders:** Architecture Team

## Context

The `flux-state` library provides reactive state management (Signals, Effects, Computeds) for Arthropod. The initial design focused on the public API (ADR 0003).

However, the internal implementation must handle:
1.  **Thread Safety**: Signals might be updated from different threads (e.g., networking, physics).
2.  **Concurrency**: Race conditions in dependency tracking must be prevented.
3.  **Deadlocks**: Reentrant access to signals within the same thread must be carefully managed.
4.  **Ownership**: Signals and Effects need shared ownership across the reactive graph.

The previous naive implementation using `Rc<RefCell<T>>` was not thread-safe and could panic if accessed across threads.

## Decision

We adopt a **Mutex-protected Internal Runtime** architecture for `flux-state`.

### Architecture Overview

1.  **Runtime Split**: Separation of the public `Runtime` handle from the private `RuntimeInner` state.
    ```rust
    // Public API (Thread-safe handle)
    #[derive(Clone)]
    pub struct Runtime {
        inner: Arc<Mutex<RuntimeInner>>,
    }

    // Internal State (Protected by Mutex)
    struct RuntimeInner {
        signals: HashMap<NodeId, Arc<dyn Any + Send + Sync>>,
        computeds: HashMap<NodeId, ComputedNode>,
        effects: HashMap<NodeId, Arc<dyn Fn() + Send + Sync>>,
        dependencies: HashMap<NodeId, HashSet<NodeId>>,
        // ...
    }
    ```

2.  **Type Erasure**: Signals store values as `Arc<dyn Any + Send + Sync>`.
    -   Allows the `Runtime` to store heterogeneous signals in a single `HashMap`.
    -   `Send + Sync` bound ensures values can be safely accessed across threads.
    -   Retrieval involves downcasting (`downcast_ref`) with type checks.

3.  **Coarse-Grained Locking**: Operations on `Runtime` (create signal, update signal, run effect) acquire the `Mutex` once, perform the graph operation, and release it.
    -   Minimizes lock contention.
    -   Prevents inconsistent graph states during updates.

4.  **Non-Reentrant Mutex**: The standard `std::sync::Mutex` is used.
    -   **Constraint**: A thread holding the lock cannot acquire it again.
    -   **Consequence**: Reactive closures (Effects, Computeds) **must not** call `Runtime` methods that acquire the lock *while* the runtime is already processing an update (e.g., inside another locked operation).
    -   **Mitigation**: The runtime releases the lock *before* invoking user callbacks (effects/computeds), then re-acquires it to update internal state.

### Dependency Tracking

To track dependencies safely:
1.  `RuntimeInner` maintains a `tracking_context: HashMap<ThreadId, Vec<NodeId>>`.
2.  When an effect runs:
    -   The runtime pushes the effect's `NodeId` onto the current thread's tracking stack.
    -   The lock is released.
    -   The effect closure executes.
    -   When the effect reads a signal, `signal.get()` acquires the lock briefly to register the dependency between the signal and the current context (top of stack).
    -   The lock is re-acquired to pop the context.

## Consequences

### Positive

-   **Thread Safety**: `Runtime` is `Send + Sync`, allowing signals to be passed between threads safely.
-   **Correctness**: Graph operations are atomic with respect to the dependency graph.
-   **Flexibility**: `Any` type erasure allows the runtime to manage any signal type.

### Negative

-   **Deadlock Risk**: User code that triggers a recursive update or complex dependency cycle *within* a lock scope (if we failed to release it) would deadlock. Our "release-then-run" strategy mitigates this for effects, but care is needed.
-   **Performance**: Mutex locking adds overhead compared to `RefCell`, but is necessary for multi-threading. Coarse-grained locking helps amortize this cost.

### Compliance

-   **Rust Safety**: Adheres to Rust's ownership and concurrency rules without `unsafe` blocks for synchronization (relying on `std::sync` primitives).

## References

-   `crates/flux-state/src/runtime.rs`: Implementation of `Runtime` and `RuntimeInner`.
-   ADR 0003: Flux-State Reactive Model (Public API).
