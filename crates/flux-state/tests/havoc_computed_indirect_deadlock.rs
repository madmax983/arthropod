use flux_state::{Computed, Runtime, Signal};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
// This test previously panicked due to deadlock.
// Now that we use RwLock instead of Mutex, recursive read locks are allowed,
// so this should PASS without deadlock.
fn test_computed_dependency_reentrancy_no_deadlock() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let runtime = Runtime::new();

        // Signal Y
        let y = Signal::new(runtime.clone(), 0);
        let (read_y, write_y) = y.split();

        // Computed A (Constant)
        // This acts as the "already locked" resource
        let a = Computed::new(runtime.clone(), || 1);

        // Computed B (Depends on Y and A)
        // This will be triggered to recompute inside A.with()
        let a_clone = a.clone();
        let read_y_clone = read_y.clone();
        let b = Computed::new(runtime.clone(), move || {
            // When recomputing B, we need to read A.
            // Reading A requires locking A.
            // We also read Y so that B becomes stale when Y changes.
            read_y_clone.get() + a_clone.get()
        });

        // Initialize values (lazy init)
        let _ = a.get();
        let _ = b.get();

        // Update Y to make B stale.
        // A is NOT stale.
        write_y.set(1);

        // Call A.with()
        // This locks A (read lock).
        a.with(|_val| {
            // Inside A.with, we access B.
            // B is stale (because Y changed), so it recomputes.
            // B recomputation reads A.
            // A.get() tries to acquire read lock on A.
            // Recursive read lock succeeds!
            let _val = b.get();
        });

        // Success: deadlock did not occur
        tx.send(()).unwrap();
    });

    // We expect the operation to complete within the timeout.
    match rx.recv_timeout(Duration::from_millis(500)) {
        Ok(_) => {
            // Test Passed: No deadlock
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("Deadlock confirmed: Indirect computed reentrancy hangs.");
        }
        Err(e) => panic!("Channel error: {:?}", e),
    }
}
