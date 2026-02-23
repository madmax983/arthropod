#[cfg(feature = "nova")]
use flux_state::{Computed, Runtime, Signal};

// We need the `nova` feature to access `inspect_graph`.
// Without it, we can't inspect the internal state, so we skip the test.
#[cfg(feature = "nova")]
#[test]
#[should_panic(expected = "Memory Leak detected")]
fn test_signal_leak() {
    let runtime = Runtime::new();

    // Initial count should be 0
    let snapshot_initial = runtime.inspect_graph();
    assert_eq!(
        snapshot_initial.nodes.len(),
        0,
        "Runtime should start clean"
    );

    // Create and drop 1000 signals
    // Since `Signal` is dropped at the end of each iteration,
    // we expect the runtime to clean them up (or use Weak refs).
    for _ in 0..1000 {
        let _ = Signal::new(runtime.clone(), 0);
    }

    // Inspect graph
    let snapshot_after = runtime.inspect_graph();

    // If signals were cleaned up, we should have 0 nodes.
    // If they leaked, we have 1000 nodes.
    //
    // The current implementation uses strong `Arc` references in the `signals` map,
    // so they are kept alive indefinitely by the `Runtime`.
    // This assertion will fail, proving the memory leak.
    assert_eq!(
        snapshot_after.nodes.len(),
        0,
        "Memory Leak detected! Expected 0 signals, found {}. \
        Signals are not being dropped from the Runtime.",
        snapshot_after.nodes.len()
    );
}

#[cfg(feature = "nova")]
#[test]
#[should_panic(expected = "Memory Leak detected")]
fn test_computed_leak() {
    let runtime = Runtime::new();

    // Initial count should be 0
    let snapshot_initial = runtime.inspect_graph();
    assert_eq!(
        snapshot_initial.nodes.len(),
        0,
        "Runtime should start clean"
    );

    // Create a base signal to depend on
    let signal = Signal::new(runtime.clone(), 0);
    let (read, _) = signal.split();

    // Create and drop 1000 computeds
    for _ in 0..1000 {
        let read_clone = read.clone();
        let _ = Computed::new(runtime.clone(), move || read_clone.get());
    }

    // Inspect graph
    let snapshot_after = runtime.inspect_graph();

    // We expect 1 signal node + 0 computed nodes.
    // If leaked, we have 1 + 1000 = 1001 nodes.
    //
    // This assertion will fail, proving the memory leak.
    assert_eq!(
        snapshot_after.nodes.len(),
        1,
        "Memory Leak detected! Expected 1 node (Signal), found {}. \
        Computeds are not being dropped from the Runtime.",
        snapshot_after.nodes.len()
    );
}
