use flux_state::{Computed, Runtime, Signal};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
// This test is expected to panic (timeout) because the deadlock bug is present.
// When the bug is fixed, this test will fail (because it won't panic),
// requiring the removal of #[should_panic].
#[should_panic(expected = "Deadlock confirmed: Indirect computed reentrancy hangs.")]
fn test_computed_dependency_reentrancy_deadlock() {
    let (tx, rx) = mpsc::channel();

    // Spawn the test in a separate thread so we can detect the hang
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
        let read_y_clone = read_y.clone();
        let a_clone = a.clone();
        let b = Computed::new(runtime.clone(), move || {
            // When recomputing B, we need to read A.
            // Reading A requires locking A.
            read_y_clone.get() + a_clone.get()
        });

        // Initialize values (lazy init)
        let _ = a.get();
        let _ = b.get();

        // Update Y to make B stale.
        // A is NOT stale.
        write_y.set(1);

        // Call A.with()
        // This locks A.
        a.with(|_val| {
            // Inside A.with, we access B.
            // B is stale, so it recomputes.
            // B recomputation reads A.
            // A.get() tries to lock A.
            // A is already locked by the outer with().
            // -> Deadlock.
            let _val = b.get();
        });

        // Success: deadlock did not occur
        tx.send(()).unwrap();
    });

    // We expect the operation to complete within the timeout.
    // If it times out, it means a deadlock occurred (FAIL/PANIC).
    match rx.recv_timeout(Duration::from_millis(500)) {
        Ok(_) => {
            // Test Passed: No deadlock (This path will fail the #[should_panic] check)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Test Failed: Deadlock detected
            panic!("Deadlock confirmed: Indirect computed reentrancy hangs.");
        }
        Err(e) => panic!("Channel error: {:?}", e),
    }
}
