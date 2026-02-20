use flux_state::{Effect, Runtime, Signal};
use std::panic;
use std::sync::{Arc, Mutex};

#[test]
fn test_orphaned_effect_panic_during_construction() {
    let runtime = Runtime::new();
    let trigger = Signal::new(runtime.clone(), 0);
    let (read_trigger, write_trigger) = trigger.split();

    let run_count = Arc::new(Mutex::new(0));
    let run_count_clone = run_count.clone();

    // 1. Attempt to create an effect that panics on the FIRST run.
    // This simulates a panic during `Effect::new` (construction).
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let _effect = Effect::new(runtime.clone(), move || {
            let val = read_trigger.get();
            *run_count_clone.lock().unwrap() += 1;

            if val == 0 {
                panic!("Panic during construction!");
            }
        });
    }));

    // Effect panicked on first run.
    // The `Effect` struct was never returned, so it was never dropped.
    // Therefore, `dispose_effect` was never called.

    // Check run count (should be 1 for the initial run)
    assert_eq!(*run_count.lock().unwrap(), 1);

    // 2. Trigger a dependency update.
    // If the "orphaned" effect is still registered and "clean" (thanks to my previous fix),
    // it will be re-scheduled and run again.
    // EXPECTED BEHAVIOR: It should NOT run, because it failed to construct.
    // ACTUAL BEHAVIOR (before fix): It runs again because it's still in the runtime.

    // We catch unwind here because if it runs, it might panic again (or run successfully if logic allows).
    // In this test, it will run, update count to 2, and NOT panic (val=1).
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write_trigger.set(1);
    }));

    // If the effect was truly orphaned and active, it ran again.
    // If we fixed it, it should not run.
    let count = *run_count.lock().unwrap();

    // Sentry: If count is 2, the orphan is alive. If 1, it's dead (correct).
    assert_eq!(
        count, 1,
        "Orphaned effect should not run after failed construction"
    );
}
