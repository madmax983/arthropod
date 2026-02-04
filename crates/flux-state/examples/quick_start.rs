//! Quick Start example for flux-state.
//!
//! This example demonstrates the core concepts: Runtime, Signal, Computed, and Effect.

use flux_state::{Computed, Effect, Runtime, Signal};

fn main() {
    // 1. Create a runtime (shared via Arc)
    // The Runtime manages the dependency graph.
    let runtime = Runtime::new();

    // 2. Create a signal
    // Signals are the atomic units of state.
    // We pass runtime.clone() because the Signal needs to register with the runtime.
    let count = Signal::new(runtime.clone(), 0);

    // We split the signal into read and write handles.
    // This allows us to pass them to different closures or threads safely.
    let (read_count, write_count) = count.split();

    // 3. Create a computed value (derived state)
    // Computed values automatically update when their dependencies change.

    // We clone the read handle to move it into the closure.
    let read_count_computed = read_count.clone();

    let double_count = Computed::new(runtime.clone(), move || {
        // Calling .get() registers a dependency.
        // Whenever read_count changes, this closure will re-run.
        read_count_computed.get() * 2
    });

    // 4. Create an effect (side effect)
    // Effects run automatically when their dependencies change.
    // We must keep the effect handle alive, otherwise it will be dropped and stop listening.
    let _effect = Effect::new(runtime.clone(), move || {
        println!(
            "Count: {}, Double: {}",
            read_count.get(),
            double_count.get()
        );
    });

    // 5. Update state
    // Output: Count: 0, Double: 0 (Initial run)

    write_count.set(1); // Output: Count: 1, Double: 2
    write_count.set(5); // Output: Count: 5, Double: 10
}
