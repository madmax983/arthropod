use flux_state::{Computed, Effect, Runtime, Signal};
use std::panic;
use std::sync::{Arc, Mutex};

#[test]
fn test_computed_panic_recovery() {
    let runtime = Runtime::new();
    let trigger = Signal::new(runtime.clone(), true); // true = safe initially
    let (read_trigger, write_trigger) = trigger.split();

    let computed = Computed::new(runtime.clone(), move || {
        if !read_trigger.get() {
            panic!("Computed Panic!");
        }
        "Safe"
    });

    assert_eq!(computed.get(), "Safe");

    // 1. Trigger panic
    // We update the signal, which might trigger recomputation immediately if eager?
    // Computed is lazy, but `Computed::new` is eager.
    // Updating dependency marks computed as stale. It won't recompute until accessed (lazy).
    write_trigger.set(false);

    // Now access it
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        computed.get();
    }));
    assert!(result.is_err(), "Computed should have panicked");

    // 2. Fix the condition
    write_trigger.set(true);

    // 3. Access again - should work if recovered
    let val = computed.get();
    assert_eq!(val, "Safe", "Computed should recover from panic");
}

#[test]
fn test_effect_panic_recovery() {
    let runtime = Runtime::new();
    let count = Signal::new(runtime.clone(), 0);
    let (read_count, write_count) = count.split();

    let effect_run_count = Arc::new(Mutex::new(0));
    let run_count_clone = effect_run_count.clone();

    // Effect panics if count is 1.
    // Must keep effect handle alive!
    let _effect_handle = Effect::new(runtime.clone(), move || {
        let val = read_count.get();
        *run_count_clone.lock().unwrap() += 1;
        if val == 1 {
            panic!("Effect Panic!");
        }
    });

    // 0. Initial run (count=0) -> OK
    assert_eq!(*effect_run_count.lock().unwrap(), 1);

    // 1. Set to 1 -> Panic
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write_count.set(1);
    }));
    assert!(result.is_err(), "Effect should have panicked");

    // verify run count increased
    assert_eq!(*effect_run_count.lock().unwrap(), 2);

    // 2. Set to 2 -> Should run again
    // If the effect is "stale" but not "pending", it won't run.
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write_count.set(2);
    }));
    assert!(result.is_ok(), "Setting unrelated value should not panic");

    // Verify effect ran again
    // Expected: 3 runs (0, 1[panic], 2)
    assert_eq!(
        *effect_run_count.lock().unwrap(),
        3,
        "Effect should have recovered and run again"
    );
}

#[test]
fn test_effect_panic_before_tracking_leaves_zombie() {
    let runtime = Runtime::new();
    let trigger = Signal::new(runtime.clone(), 0);
    let (read_trigger, write_trigger) = trigger.split();

    let run_count = Arc::new(Mutex::new(0));
    let run_count_clone = run_count.clone();

    // Effect panics BEFORE reading any signal on first run.
    let _effect_handle = Effect::new(runtime.clone(), move || {
        *run_count_clone.lock().unwrap() += 1;

        // We use get_untracked to check value without registering dependency.
        // If we panic here, we haven't registered any dependencies yet.
        let current_val = read_trigger.get_untracked();

        if current_val == 1 {
            panic!("Transient Panic!");
        }

        // Actual dependency tracking happens here.
        // If we panicked above, this line is never reached.
        let _ = read_trigger.get();
    });

    // 1. Initial run (val=0). Runs once.
    assert_eq!(*run_count.lock().unwrap(), 1);

    // 2. Set to 1 -> Panic
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write_trigger.set(1);
    }));
    assert!(result.is_err(), "Effect should have panicked");
    assert_eq!(*run_count.lock().unwrap(), 2);

    // 3. Set to 2 -> Should it run?
    // If it panicked before tracking, it lost its dependency on `trigger`.
    // It should NOT run when `trigger` changes to 2.
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write_trigger.set(2);
    }));
    assert!(result.is_ok());

    // Verify count is still 2 (zombie)
    let count = *run_count.lock().unwrap();
    assert_eq!(
        count, 2,
        "Effect should be a zombie because it panicked before tracking dependency"
    );
}
