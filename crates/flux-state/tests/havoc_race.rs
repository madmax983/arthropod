use flux_state::{Computed, Runtime, Signal};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[test]
fn test_stale_read_race() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (read, write) = signal.split();

    // Use a barrier to synchronize threads
    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = barrier.clone();

    // Create a computed that simulates a slow computation
    let computed = Computed::new(runtime.clone(), move || {
        // This is called initially.
        // We only want to block on subsequent calls.
        if read.get() == 0 {
            return 0;
        }

        // Wait for main thread to be ready to read
        // But only if we are the "slow" thread?
        // Let's use the barrier unconditionally here, but we need to make sure
        // the initial computation doesn't block.
        // The initial computation happens inside Computed::new, before we spawn the other thread.
        // So the barrier must be met by the main thread too? No.

        // Let's just use a simple flag or atomic, or rely on the signal value.
        // If signal is 1, we block.
        barrier_clone.wait();

        // Simulate work
        thread::sleep(Duration::from_millis(50));

        read.get()
    });

    // Verify initial state
    assert_eq!(computed.get(), 0);

    // Make computed stale
    write.set(1);

    // Spawn a thread to trigger recomputation
    let computed_thread = computed.clone();
    let handle = thread::spawn(move || {
        // This will trigger recompute because it's stale (signal is 1).
        // Inside compute closure:
        // 1. Checks signal == 1.
        // 2. Waits on barrier.
        // 3. Sleeps.
        // 4. Returns 1.
        computed_thread.get()
    });

    // Main thread waits for the background thread to reach the barrier (start computing)
    barrier.wait();

    // At this point:
    // The background thread is inside the compute closure.
    // The `stale` flag has likely been cleared by `prepare_execution`.
    // The new value has NOT been stored yet.

    // Main thread tries to read.
    // If the bug exists, `is_stale` returns false, and we get the OLD value (0).
    // If fixed, `recompute` (or `get`) blocks until computation finishes, then returns 1.
    let val = computed.get();

    handle.join().unwrap();

    // Assert we got the fresh value
    assert_eq!(
        val, 1,
        "Read stale value 0 while recomputing! Race condition detected."
    );
}
