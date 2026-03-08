use flux_state::{Computed, Runtime, Signal};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

#[test]
#[should_panic]
fn test_cross_thread_cycle_deadlock() {
    // This test is designed to expose a deadlock in the runtime.
    // T1 reads A -> waits for B.
    // T2 reads B -> waits for A.
    // Without deadlock detection, this hangs.
    // With deadlock detection, it should panic.

    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        let runtime = Runtime::new();
        let s = Signal::new(runtime.clone(), 0);
        let (r_s, w_s) = s.split();

        // Containers to allow circular reference
        let b_holder: Arc<RwLock<Option<Computed<i32>>>> = Arc::new(RwLock::new(None));
        let b_reader = b_holder.clone();

        let r_s_a = r_s.clone();
        let a = Computed::new(runtime.clone(), move || {
            let s_val = r_s_a.get();
            if s_val == 0 {
                0
            } else {
                // Sleep to ensure T2 grabs B lock
                thread::sleep(Duration::from_millis(50));
                if let Some(b) = b_reader.read().unwrap().as_ref() {
                    b.get() + 1
                } else {
                    0
                }
            }
        });

        let a_holder = Arc::new(RwLock::new(Some(a.clone())));
        let a_reader = a_holder.clone();

        let r_s_b = r_s.clone();
        let b = Computed::new(runtime.clone(), move || {
            let s_val = r_s_b.get();
            if s_val == 0 {
                0
            } else {
                // Sleep to ensure T1 grabs A lock
                thread::sleep(Duration::from_millis(50));
                if let Some(a) = a_reader.read().unwrap().as_ref() {
                    a.get() + 1
                } else {
                    0
                }
            }
        });

        {
            let mut guard = b_holder.write().unwrap();
            *guard = Some(b.clone());
        }

        // Initialize (non-stale)
        assert_eq!(a.get(), 0);
        assert_eq!(b.get(), 0);

        // Mark stale
        w_s.set(1);

        // Spawn threads
        let a_thread = a.clone();
        let t1 = thread::spawn(move || {
            a_thread.get();
        });

        let b_thread = b.clone();
        let t2 = thread::spawn(move || {
            b_thread.get();
        });

        // We expect these to panic or complete.
        // If they deadlock, join will block forever (caught by outer timeout).
        // Since deadlock detection is enabled, one or both of these threads will panic.
        let r1 = t1.join();
        let r2 = t2.join();

        // If either thread panicked, the main test thread should propagate the panic or just complete.
        // For the sake of this test, we expect a deadlock panic if we get here.
        if r1.is_err() || r2.is_err() {
            panic!("Deadlock detected: Cyclic dependency in computed values across threads.");
        }

        let _ = tx.send(());
    });

    // Wait for result with timeout
    if rx.recv_timeout(Duration::from_secs(2)).is_err() {
        panic!("Test timed out - Deadlock detected! (System stuck in wait loop)");
    }
}
