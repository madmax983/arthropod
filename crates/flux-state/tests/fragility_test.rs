use flux_state::{Computed, Effect, Runtime, Signal};
use std::sync::{Arc, Barrier};
use std::thread;
use proptest::prelude::*;

#[test]
#[should_panic(expected = "Reactive recursion limit exceeded")]
fn deep_computed_chain_panic() {
    let runtime = Runtime::new();
    let root = Signal::new(runtime.clone(), 0);
    let (read, _) = root.clone().split();

    // First computed depends on signal
    let r = read.clone();
    let mut last_computed = Computed::new(runtime.clone(), move || r.get());

    // Chain 200 computed values
    for _ in 0..200 {
        let prev = last_computed.clone();
        last_computed = Computed::new(runtime.clone(), move || prev.get() + 1);
    }

    // Trigger the chain reaction by updating the root!
    // This marks everyone stale.
    let (_, write_root) = root.split();
    write_root.set(1);

    // Now read the last one. It should recurse to recompute.
    let _ = last_computed.get();
}

#[test]
#[should_panic(expected = "Reactive recursion limit exceeded")]
fn deep_effect_chain_panic() {
    let runtime = Runtime::new();

    // Create 202 signals
    let mut signals = Vec::new();
    for _ in 0..202 {
        signals.push(Signal::new(runtime.clone(), 0));
    }

    // Keep effects alive!
    let mut effects = Vec::new();

    // Create 201 effects
    for i in 0..201 {
        let (read_current, _) = signals[i].clone().split();
        let (_, write_next) = signals[i+1].clone().split();

        effects.push(Effect::new(runtime.clone(), move || {
            let val = read_current.get();
            write_next.set(val + 1);
        }));
    }

    // Trigger the chain
    let (_, write_0) = signals[0].clone().split();
    write_0.set(1);
}

#[test]
fn concurrent_stress_test() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (_, write) = signal.clone().split();

    let barrier = Arc::new(Barrier::new(10));
    let mut handles = Vec::new();

    for _ in 0..10 {
        let w = write.clone();
        let b = barrier.clone();
        handles.push(thread::spawn(move || {
            b.wait();
            for _ in 0..1000 {
                w.update(|val| *val += 1);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let (read, _) = signal.split();
    assert_eq!(read.get_untracked(), 10000);
}

proptest! {
    // Test that the recursion limit is enforced around 100
    #[test]
    fn test_recursion_limit_exact(depth in 90..110usize) {
        let runtime = Runtime::new();
        let root = Signal::new(runtime.clone(), 0);
        let (read, _) = root.clone().split();

        let r = read.clone();
        let mut last_computed = Computed::new(runtime.clone(), move || r.get());

        // Chain 'depth' computed values
        for _ in 0..depth {
            let prev = last_computed.clone();
            last_computed = Computed::new(runtime.clone(), move || prev.get() + 1);
        }

        let (_, write_root) = root.clone().split();
        write_root.set(1);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            last_computed.get()
        }));

        if depth > 105 {
             assert!(result.is_err(), "Should panic at depth {}", depth);
        } else if depth < 95 {
             assert!(result.is_ok(), "Should NOT panic at depth {}", depth);
        }
    }
}
