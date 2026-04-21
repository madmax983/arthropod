use flux_state::{Effect, Runtime, Signal};
use std::panic;
use std::sync::{Arc, Mutex};

#[test]
fn test_panic_drops_pending_effects() {
    let runtime = Runtime::new();
    let trigger = Signal::new(runtime.clone(), 0);
    let (read_trigger, write_trigger) = trigger.split();

    let run_counts = Arc::new(Mutex::new(vec![false; 1000]));
    let mut effects = Vec::new();

    for i in 0..1000 {
        let run_counts = run_counts.clone();
        let read_trigger = read_trigger.clone();
        effects.push(Effect::new(runtime.clone(), move || {
            let val = read_trigger.get();
            if val == 1 && i == 179 {
                panic!("Panic at index 179");
            }
            run_counts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)[i] = true;
        }));
    }

    // Reset counts before update
    {
        let mut counts = run_counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for ran in counts.iter_mut() {
            *ran = false;
        }
    }

    // Trigger update
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write_trigger.set(1);
    }));
    assert!(result.is_err(), "Should have panicked");

    // Recovery step: Trigger another update to flush the restored effects
    let recovery = Signal::new(runtime.clone(), 0);
    let (_, write_recovery) = recovery.split();
    write_recovery.set(1);

    // Verify results
    let counts = run_counts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    // Check that we have at least one execution and at least one failure (the panic itself)
    // The panic happens at 179.
    // All OTHER effects should have run.

    let mut missing_runs = 0;
    for (i, &ran) in counts.iter().enumerate() {
        if i != 179 && !ran {
            missing_runs += 1;
        }
    }

    println!("Missing runs: {}", missing_runs);
    assert_eq!(
        missing_runs, 0,
        "Some effects were skipped due to panic! Missing count: {}",
        missing_runs
    );
}
