use flux_state::{Computed, Runtime, Signal};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_condvar_poison_recovery() {
    let runtime = Runtime::new();

    // Signal to trigger re-evaluation
    let signal = Signal::new(runtime.clone(), 0);
    let (read, write) = signal.split();

    // Synchronization primitive to ensure threads interleave correctly
    let sync = Arc::new(Mutex::new(false));

    // A computed value that blocks, waits for thread B to start, and then panics.
    let sync_clone = sync.clone();
    let read_clone = read.clone();
    let computed = Computed::new(runtime.clone(), move || {
        let val = read_clone.get();
        if val > 0 {
            // Signal that we've reached the point just before panic
            *sync_clone
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = true;

            // Give Thread B time to start waiting on this computation
            thread::sleep(Duration::from_millis(50));

            // Panic during computation
            panic!("Intentional panic during recompute");
        }
        val
    });

    // Make it stale
    write.set(1);

    // Thread A: Attempts to read `computed`. It will start computing it, and then panic.
    let computed_for_a = computed.clone();
    let thread_a = thread::spawn(move || {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            computed_for_a.get();
        }));
    });

    // Wait until Thread A reaches the point where it's about to panic
    // (Meaning Thread A has definitely claimed the `computing` lock for this node)
    while !*sync
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
    {
        thread::sleep(Duration::from_millis(5));
    }

    // Thread B: Attempts to read the same `computed` while Thread A is computing it.
    // Thread B will block on the `condvar.wait()` inside `wait_for_computation`.
    let computed_for_b = computed.clone();

    // We run thread B in a separate thread so we can time it out
    let (tx, rx) = std::sync::mpsc::channel();
    let _thread_b = thread::spawn(move || {
        // If the ComputingGuard Drop implementation is removed, Thread B will hang here indefinitely.
        // If it is present, Thread B will wake up (because Thread A's panic triggers `condvar.notify_all()`),
        // see that the node is still stale, and attempt to recompute it itself (and panic again).
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            computed_for_b.get();
        }));
        let _ = tx.send(res);
    });

    // Wait for Thread A to finish panicking
    let _ = thread_a.join();

    // Wait for Thread B to finish.
    // If we use a timeout, we can catch the hang explicitly and fail the test.
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(res) => {
            // Thread B woke up and finished (likely panicking itself since the computation still panics).
            assert!(
                res.is_err(),
                "Thread B should have also panicked when attempting to recompute the still-stale node"
            );
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            // If we hit this, Thread B never woke up from `condvar.wait()`.
            // This happens if `ComputingGuard` does not remove the node from `computing` and call `condvar.notify_all()` on drop.
            panic!(
                "Thread B hung indefinitely waiting for a computation that panicked! The `condvar.notify_all()` in `ComputingGuard::drop` was likely removed."
            );
        }
        Err(_) => panic!("Channel closed unexpectedly"),
    }
}
