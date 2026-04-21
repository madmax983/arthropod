use flux_state::{Effect, Runtime, Signal};
use std::panic;
use std::sync::{Arc, Mutex};

#[test]
fn test_batch_panic_recovery() {
    let runtime = Runtime::new();
    let trigger = Signal::new(runtime.clone(), 0);
    let (read, write) = trigger.split();

    let execution_log = Arc::new(Mutex::new(Vec::new()));

    // We create 3 effects.
    // Note: The runtime processes pending effects from a batch. However, because dependency
    // tracking uses HashSets, the order in which effects are added to the pending queue
    // is non-deterministic.
    // The test verifies that regardless of order, if one effect panics, the others are
    // preserved and eventually executed.

    // Effect 1: Safe effect
    let log1 = execution_log.clone();
    let read1 = read.clone();
    let _effect1 = Effect::new(runtime.clone(), move || {
        let val = read1.get();
        if val == 1 {
            log1.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push("E1");
        }
    });

    // Effect 2: Safe effect
    let log2 = execution_log.clone();
    let read2 = read.clone();
    let _effect2 = Effect::new(runtime.clone(), move || {
        let val = read2.get();
        if val == 1 {
            log2.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push("E2");
        }
    });

    // Effect 3: Panicking effect
    let log3 = execution_log.clone();
    let read3 = read.clone();
    let _effect3 = Effect::new(runtime.clone(), move || {
        let val = read3.get();
        if val == 1 {
            log3.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push("E3_START");
            panic!("Effect 3 Panic");
        }
    });

    // 1. Trigger Update to value 1
    // This schedules all 3 effects.
    // One of them (E3) will panic.
    // The others should remain in the pending queue via PanicRestorer.
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write.set(1);
    }));

    assert!(result.is_err(), "Should have panicked");

    // Check log: E3 started.
    // Note: Due to HashSet iteration order in dependency tracking, the execution order
    // of effects 1, 2, and 3 is non-deterministic.
    // E3 might have run first, last, or in the middle.

    // 2. Trigger another update to flush pending effects.
    // We can use a dummy signal to trigger a flush cycle without changing 'trigger' again.
    // If E3 panicked early, this ensures remaining effects (restored by PanicRestorer) are executed.
    let dummy = Signal::new(runtime.clone(), 0);
    let (_, dummy_write) = dummy.split();
    dummy_write.set(1);

    // Now check that all effects eventually ran.
    {
        let log = execution_log
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // We verify set membership rather than order because of the non-deterministic HashSet
        assert!(log.contains(&"E1"), "Effect 1 should have run");
        assert!(log.contains(&"E2"), "Effect 2 should have run");
        assert!(log.contains(&"E3_START"), "Effect 3 should have started");

        // Length check to ensure no duplicates (optional but good sanity check)
        assert_eq!(
            log.len(),
            3,
            "All 3 effects should have run exactly once (or started)"
        );
    }
}

#[test]
fn test_panic_restorer_empty() {
    // Tests the path where PanicRestorer drops when `remaining_effects` is empty.
    let runtime = Runtime::new();
    let trigger = Signal::new(runtime.clone(), 0);
    let (read, write) = trigger.split();

    let _effect = Effect::new(runtime.clone(), move || {
        let val = read.get();
        if val == 1 {
            panic!("Single effect panic");
        }
    });

    // 1. Trigger Update to value 1
    // This schedules 1 effect. `take_pending_effects` moves it to `local_effects`.
    // `process_effect_batch` iterates, pops the single effect.
    // `PanicRestorer` is created with an EMPTY `remaining_effects`.
    // The effect runs and panics.
    // `PanicRestorer` drops. Since `remaining_effects` is empty, it does nothing.
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write.set(1);
    }));

    assert!(result.is_err(), "Should have panicked");

    // We can do another safe update to ensure the runtime isn't poisoned or looping
    let (read2, write2) = Signal::new(runtime.clone(), 0).split();
    let log = Arc::new(Mutex::new(Vec::new()));
    let log_clone = log.clone();
    let _effect2 = Effect::new(runtime.clone(), move || {
        let val = read2.get();
        if val == 1 {
            log_clone
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push("Safe");
        }
    });

    write2.set(1);
    assert_eq!(
        *log.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        vec!["Safe"]
    );
}

#[test]
fn test_drop_computing_guard_panic() {
    let runtime = Runtime::new();

    let sig1 = Signal::new(runtime.clone(), 0);
    let (read1, write1) = sig1.split();

    let computed = flux_state::Computed::new(runtime.clone(), move || {
        let val = read1.get();
        if val == 1 {
            panic!("Intentional panic inside computed");
        }
        val
    });

    // Access once to init
    assert_eq!(computed.get(), 0);

    write1.set(1);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        computed.get();
    }));
    assert!(result.is_err());

    // Since it panicked, the ComputingGuard's drop was executed and it should have removed itself from computing list
    // If it was removed, then next thread wouldn't deadlock
    write1.set(2);
    assert_eq!(computed.get(), 2);
}
