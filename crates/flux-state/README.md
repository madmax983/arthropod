# flux-state

Reactive state management for Arthropod GUI framework.

`flux-state` provides fine-grained reactivity using a Signal/Effect architecture, similar to SolidJS or Preact Signals, but adapted for Rust's ownership model and thread safety requirements.

## Core Concepts

- **Signal**: The atomic unit of state. It holds a value and notifies dependents when it changes.
- **Computed**: A derived value that automatically updates when its dependencies change.
- **Effect**: A side effect that runs automatically when its dependencies change.
- **Runtime**: The coordinator that manages the dependency graph and propagates updates.

## Thread Safety

Core primitives (`Signal`, `ReadSignal`, `WriteSignal`, `Computed`, `Effect`) are thread-safe (`Send + Sync`) when their inner state types are `Send + Sync`, and are designed to be shared across threads in that case. The `Runtime` uses internal locking to ensure consistency.

## Quick Start

```rust
use flux_state::{Runtime, Signal, Effect, Computed};
use std::sync::Arc;

// 1. Create a runtime (shared via Arc)
let runtime = Runtime::new();

// 2. Create a signal
let count = Signal::new(runtime.clone(), 0);
let (read_count, write_count) = count.split();

// 3. Create a computed value (derived state)
let read_count_computed = read_count.clone();
let double_count = Computed::new(runtime.clone(), move || {
    read_count_computed.get() * 2
});

// 4. Create an effect (side effect)
Effect::new(runtime.clone(), move || {
    println!("Count: {}, Double: {}", read_count.get(), double_count.get());
});

// 5. Update state
write_count.set(1); // Output: Count: 1, Double: 2
write_count.set(5); // Output: Count: 5, Double: 10
```

## The "Why"

Traditional GUI frameworks often use "immediate mode" (redraw everything) or "diffing" (compare VDOM trees). `flux-state` enables **fine-grained reactivity**:

1. You describe *relationships* between data (signals) and UI (effects).
2. When data changes, *only* the specific effects that depend on it re-run.
3. This avoids unnecessary re-rendering and layout calculations, scaling linearly with complexity rather than widget count.
