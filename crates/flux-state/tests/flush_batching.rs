use flux_state::{Effect, Runtime, Signal};
use std::sync::{Arc, Mutex};

// Verify effect execution order and recursion handling.
// This test mimics the behavior of `flush_effects` to ensure optimization doesn't break it.
#[test]
fn test_flush_effects_order_and_recursion() {
    let runtime = Runtime::new();
    let log = Arc::new(Mutex::new(Vec::new()));

    // Create a chain of signals to trigger effects in a specific order.
    let s1 = Signal::new(runtime.clone(), 0);
    let (read1, write1) = s1.split();

    let s2 = Signal::new(runtime.clone(), 0);
    let (read2, write2) = s2.split();

    // Effect 2 depends on s2
    let log_clone = log.clone();
    let _e2 = Effect::new(runtime.clone(), move || {
        if read2.get() > 0 {
            log_clone.lock().unwrap().push("E2".to_string());
        }
    });

    // Effect 1 depends on s1 and updates s2
    let log_clone = log.clone();
    let write2_clone = write2.clone();
    let _e1 = Effect::new(runtime.clone(), move || {
        if read1.get() > 0 {
            log_clone.lock().unwrap().push("E1".to_string());
            // This update should trigger E2.
            // In the current implementation (immediate recursive flush?),
            // or queued?
            // "Flush pending effects (synchronous for now)" - from runtime.rs
            write2_clone.set(1);
            log_clone.lock().unwrap().push("E1_done".to_string());
        }
    });

    // Initial state: effects run once.
    log.lock().unwrap().clear();

    // Update s1. This triggers E1.
    // E1 runs, updates s2. This triggers E2.
    // If flush is recursive/immediate:
    // s1 set -> notify(s1) -> mark E1 stale -> flush -> run E1
    // E1 runs -> s2 set -> notify(s2) -> mark E2 stale -> flush (inside E1?) -> run E2
    // So order should be E1 -> E2 -> E1_done?
    // OR if flush queues E2 but doesn't run it until E1 finishes?
    // Let's find out.
    write1.set(1);

    let recorded = log.lock().unwrap().clone();
    println!("Recorded: {:?}", recorded);

    // Based on `runtime.rs`:
    // write.set calls `notify`.
    // `notify` calls `flush_effects`.
    // `flush_effects` loops popping pending effects.
    // `run_effect` runs E1.
    // E1 calls `write2.set`.
    // `write2.set` calls `notify`.
    // `notify` calls `flush_effects`.
    // Nested `flush_effects` sees E2 pending. Runs E2.
    // E2 finishes.
    // Nested `flush_effects` returns.
    // `notify` returns.
    // `write2.set` returns.
    // E1 finishes.
    // `run_effect` returns.
    // Outer `flush_effects` continues (if more effects).

    // So expected order: E1, E2, E1_done.
    // If optimization changes this (e.g. by processing batch),
    // it must support re-entrancy.

    assert_eq!(recorded, vec!["E1", "E2", "E1_done"]);
}

#[test]
fn test_flush_effects_batching() {
    // This test ensures that if multiple effects are pending, they are all executed.
    let runtime = Runtime::new();
    let s = Signal::new(runtime.clone(), 0);
    let (read, write) = s.split();

    let counter = Arc::new(Mutex::new(0));

    let c1 = counter.clone();
    let r1 = read.clone();
    let _e1 = Effect::new(runtime.clone(), move || {
        r1.get();
        *c1.lock().unwrap() += 1;
    });

    let c2 = counter.clone();
    let r2 = read.clone();
    let _e2 = Effect::new(runtime.clone(), move || {
        r2.get();
        *c2.lock().unwrap() += 1;
    });

    let c3 = counter.clone();
    let r3 = read.clone();
    let _e3 = Effect::new(runtime.clone(), move || {
        r3.get();
        *c3.lock().unwrap() += 1;
    });

    // Initial run
    assert_eq!(*counter.lock().unwrap(), 3);

    // Update
    *counter.lock().unwrap() = 0;
    write.set(1);

    assert_eq!(*counter.lock().unwrap(), 3);
}
