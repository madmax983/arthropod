use flux_state::{Computed, Runtime, Signal};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[test]
#[should_panic(expected = "Race condition detected")]
fn test_computed_race_condition() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (read_sig, write_sig) = signal.split();

    // Barrier to synchronize the race:
    // 2 participants: Thread A (compute) and Main Thread (read)
    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = barrier.clone();

    // The Computed value that depends on `signal`.
    // We inject a delay/barrier inside the computation to catch the runtime
    // in the state where `stale` is false (cleared) but the new value isn't stored.
    let computed = Computed::new(runtime.clone(), move || {
        let val = read_sig.get();
        if val == 1 {
            // We are in the update phase triggered by `write_sig.set(1)`.
            // Block here. At this point, `Runtime::recompute` has already cleared the `stale` flag.
            barrier_clone.wait();
            // Sleep a bit to ensure the other thread has time to read the STALE value.
            thread::sleep(Duration::from_millis(100));
        }
        val + 100
    });

    // Initial value check (runs immediately upon creation)
    assert_eq!(computed.get(), 100);

    // Trigger update
    write_sig.set(1);
    // `computed` is now marked stale.

    // Spawn Thread A to trigger recompute
    let computed_a = computed.clone();
    let handle_a = thread::spawn(move || {
        // Accessing `computed_a` will trigger `recompute`.
        // Inside `recompute`, it will hit the barrier.
        computed_a.get()
    });

    // Wait for Thread A to hit the barrier.
    // This guarantees that Thread A is inside `compute_fn` and has cleared the `stale` flag.
    barrier.wait();

    // Main Thread (Thread B) reads immediately.
    // Since `stale` is cleared by Thread A, `is_stale()` returns false.
    // So `recompute()` is skipped.
    // It proceeds to read the stored value, which is still the OLD value (100)
    // because Thread A hasn't finished computing/storing the new value yet.
    let val = computed.get();

    // Allow Thread A to finish
    handle_a.join().unwrap();

    // The correct behavior requires the value to be consistent with the dependency update.
    // Since `signal` is 1, `computed` should be 101.
    // If we read 100, we have a race condition (stale read).
    assert_eq!(
        val, 101,
        "Race condition detected! Read stale value 100 instead of 101. \
        The `stale` flag was cleared before the new value was written."
    );
}
