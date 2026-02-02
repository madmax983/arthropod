# 16. Chronos Time Travel System

Date: 2024-05-22

## Status

Experimental

## Context

Enterprise applications frequently require robust undo/redo capabilities to improve user experience and safety. Implementing this logic manually for each widget or state variable leads to:
- **Code Duplication**: Command pattern boilerplate repeated across the codebase.
- **Inconsistency**: Different parts of the app behaving differently regarding history (e.g., some actions being undoable, others not).
- **State Desynchronization**: If the history stack gets out of sync with the actual application state.

Arthropod's `flux-state` library provides a reactive foundation, but it inherently models *current* state. There is no built-in mechanism to traverse previous states.

## Decision

We will implement a "Time Travel" system, codenamed **Chronos**, as an experimental module.

The core design involves:

1.  **`Timeline`**: A central registry that manages the history of operations. It maintains two stacks: `past` (undo stack) and `future` (redo stack).
2.  **`RetroSignal<T>`**: A wrapper around `flux-state::Signal<T>`. It intercepts `set` and `update` calls to record the change as a reversible `Operation` in the `Timeline` before applying it to the underlying signal.
3.  **Command Pattern with Closures**: Instead of reifying every action as a struct, we use `Box<dyn Fn()>` closures for `undo` and `redo` operations, capturing the necessary state (values and signal references) by value.

The API matches `flux-state` closely:
```rust
let timeline = Timeline::new();
let count = RetroSignal::new(runtime, timeline, 0);
count.set(1); // Records change
timeline.undo(); // count becomes 0
```

## Consequences

### Positive
- **transparency**: Developers can opt-in to history tracking by simply changing `Signal` to `RetroSignal`. Downstream consumers (Effects, Computeds) remain agnostic as they just read the underlying signal.
- **Atomicity**: The `Timeline` supports batching via `begin_transaction` and `commit_transaction`, allowing complex multi-signal updates to be undone as a single unit.
- **Safety**: Leveraging Rust's ownership and closure capture ensures that the undo/redo logic always has valid references to the signals it needs to modify.

### Negative
- **Memory Overhead**: Every state change involves cloning the old and new values to store them in the history closures. For large state objects, this could be expensive.
- **Performance**: `Timeline` is protected by a `Mutex`, introducing synchronization overhead for every write to a `RetroSignal`.
- **Complexity**: Branching history (making a change after an undo) clears the redo stack, which is standard behavior but implies data loss of the "future" timeline.
