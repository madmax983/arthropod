use flux_state::{Computed, Runtime, Signal};
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn test_stale_while_computing_propagates_to_subscribers() {
    let runtime = Runtime::new();

    // We create a root signal
    let root = Signal::new(runtime.clone(), 0);
    let (read_root, write_root) = root.split();

    // Barriers for synchronizing Thread A and Main thread
    let barrier_start = Arc::new(Barrier::new(2));
    let barrier_end = Arc::new(Barrier::new(2));

    let b_start_c = barrier_start.clone();
    let b_end_c = barrier_end.clone();

    // The inner computed that we can pause
    let read_root_c = read_root.clone();
    let inner = Computed::new(runtime.clone(), move || {
        let val = read_root_c.get();
        // Wait at the barrier so the main thread can update the root while this is computing
        if val == 1 {
            b_start_c.wait();
            b_end_c.wait();
        }
        val
    });

    // The outer computed that depends on the inner computed
    let outer = Computed::new(runtime.clone(), move || inner.get() + 1);

    // 1. Initial state
    assert_eq!(outer.get(), 1); // 0 + 1 = 1

    // 2. We update root to 1. Both `inner` and `outer` are marked stale.
    write_root.set(1);

    // 3. Thread A tries to read `outer`.
    // It will start computing `outer`, which will read `inner`.
    // Reading `inner` will start computing `inner`, which blocks at `barrier_start` because val == 1.
    let outer_clone = outer.clone();
    let thread_a = thread::spawn(move || outer_clone.get());

    // 4. Wait for Thread A to reach the inner computation and block there
    barrier_start.wait();

    // 5. Main thread updates root to 2 *while* inner is computing!
    // Since inner is already computing, it will be added to `stale_while_computing`.
    // Crucially, `mark_stale` historically DID NOT propagate this staleness
    // to `outer` because `inner` was ALREADY stale in the `stale` set,
    // so it early-returned false!
    write_root.set(2);

    // 6. Unpause inner computation
    barrier_end.wait();

    // 7. Thread A finishes reading `outer`.
    let _ = thread_a.join().unwrap();

    // 8. Main thread tries to read `outer`.
    // IF the bug exists, `outer` does NOT know it is stale, because
    // step 5 early-returned and never marked `outer` as stale.
    // So Thread C will get a cached, STALE value instead of re-evaluating!
    let final_val = outer.get();

    // final_val MUST be 3 (2 + 1) because root is 2.
    assert_eq!(final_val, 3, "Lost update: outer did not recompute!");
}
