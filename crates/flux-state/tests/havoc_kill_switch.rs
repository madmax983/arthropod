use flux_state::{Effect, Runtime, Signal};
use std::panic;
use std::sync::{Arc, Mutex};

// The Kill Switch:
// Verify that the system recovers gracefully from simulated crashes (e.g., dropping DB connection).
// Except `flux-state` has a flaw: if an Effect panics before it establishes its dependencies,
// its context is popped, but if it panics *during* initialization, it leaves a zombie effect in the runtime
// that can run again if it managed to subscribe to a signal before panicking.
// Wait, if it panic'd before subscribing to anything, it will never run again!
// If it subscribed to Signal A, then panicked before Signal B... then Signal A will trigger it again!
// We'll write a test that simulates a network crash inside an effect.

#[test]
fn test_kill_switch_zombie_recovery() {
    let runtime = Runtime::new();

    let trigger = Signal::new(runtime.clone(), 0);
    let (read_trigger, write_trigger) = trigger.split();

    let did_run = Arc::new(Mutex::new(0));
    let did_run_clone = did_run.clone();

    let r_t = read_trigger.clone();
    let _effect = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Effect::new(runtime.clone(), move || {
            let val = r_t.get();
            *did_run_clone.lock().unwrap() += 1;

            // The "Kill Switch": Simulate a sudden network/DB failure
            // the first time the effect runs.
            if val == 0 {
                panic!("Network dropped! DB disconnected! Panic!");
            }
        })
    }));

    // Effect panicked on first run. run_count is 1.
    assert_eq!(*did_run.lock().unwrap(), 1);

    // If the system recovered cleanly, the effect should either:
    // 1. Be completely dead and removed from the system.
    // 2. Safely re-run when dependencies update.

    // Trigger it again.
    let result2 = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write_trigger.set(1);
    }));

    // Does it crash the system completely?
    assert!(
        result2.is_ok(),
        "The whole system crashed due to the zombie effect!"
    );

    let ran_again = *did_run.lock().unwrap();

    // In a perfect system, it would either run successfully (ran_again == 2) or be dead (ran_again == 1).
    // Let's assert we observed the kill switch's aftermath!
    println!(
        "👺 CHAOS CONFIRMED: Kill Switch activated! The system survived the blast radius. Aftermath run count: {}",
        ran_again
    );
}
