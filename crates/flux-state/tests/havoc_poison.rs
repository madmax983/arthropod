use flux_state::{Computed, Runtime, Signal};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

#[test]
fn test_deadlock_poisoning() {
    let runtime = Runtime::new();

    // Use RwLock<Option<Computed>> to create a cycle
    // We need Arc<RwLock> to share between threads and closures
    let comp_a_handle: Arc<RwLock<Option<Computed<i32>>>> = Arc::new(RwLock::new(None));
    let comp_b_handle: Arc<RwLock<Option<Computed<i32>>>> = Arc::new(RwLock::new(None));

    // Create Signal to trigger updates (making them stale)
    let trigger = Signal::new(runtime.clone(), 0);
    let (r_trigger, w_trigger) = trigger.split();

    // Computed A: Reads trigger, sleeps, then reads B
    let r_trigger_a = r_trigger.clone();
    let b_handle_a = comp_b_handle.clone();

    let comp_a = Computed::new(runtime.clone(), move || {
        let val = r_trigger_a.get(); // Subscribe to trigger

        if let Some(b) = b_handle_a.read().unwrap().as_ref() {
            // Sleep to allow T2 to start computing B
            thread::sleep(Duration::from_millis(50));
            // Read B
            b.get() + val
        } else {
            val
        }
    });

    // Computed B: Reads trigger, sleeps, then reads A
    let r_trigger_b = r_trigger.clone();
    let a_handle_b = comp_a_handle.clone();

    let comp_b = Computed::new(runtime.clone(), move || {
        let val = r_trigger_b.get();

        if let Some(a) = a_handle_b.read().unwrap().as_ref() {
            // Sleep to allow T1 to start computing A
            thread::sleep(Duration::from_millis(50));
            a.get() + val
        } else {
            val
        }
    });

    // Link them
    *comp_a_handle.write().unwrap() = Some(comp_a.clone());
    *comp_b_handle.write().unwrap() = Some(comp_b.clone());

    // Update trigger to mark them stale
    w_trigger.set(1);

    // Spawn threads to read them simultaneously
    let thread_a = thread::spawn(move || {
        let _ = std::panic::catch_unwind(|| {
            comp_a.get();
        });
    });

    let thread_b = thread::spawn(move || {
        let _ = std::panic::catch_unwind(|| {
            comp_b.get();
        });
    });

    // Wait for chaos to settle
    let _ = thread_a.join();
    let _ = thread_b.join();

    // Now check if Runtime is poisoned
    // If poisoned, this will panic (Err)
    // If our fix works, this should return Ok(())
    let result = std::panic::catch_unwind(|| {
        let _s = Signal::new(runtime.clone(), 100);
    });

    assert!(
        result.is_ok(),
        "Runtime mutex is poisoned! The system did not recover from the deadlock panic."
    );
}
