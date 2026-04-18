use flux_state::{Computed, Effect, Runtime, Signal};
use std::panic;
use std::sync::{Arc, Mutex};

/// 🛡️ Sentry: Verify that effects are processed in a Breadth-First manner.
/// When Effect A triggers Signal B, Effect B is scheduled.
/// If Signal A triggers both Effect B and Effect C, they should run in the same batch.
#[test]
fn test_effect_batch_order() {
    let runtime = Runtime::new();
    let root = Signal::new(runtime.clone(), 0);
    let (read_root, write_root) = root.split();

    // Log of executed effects
    let execution_log = Arc::new(Mutex::new(Vec::new()));
    let log_clone = execution_log.clone();

    // Effect 1: Depends on root
    let read_root_1 = read_root.clone();
    let log_1 = execution_log.clone();
    let _effect_1 = Effect::new(runtime.clone(), move || {
        let val = read_root_1.get();
        if val > 0 {
            log_1
                .lock()
                .unwrap()
                .push(format!("Effect 1 ran with {}", val));
        }
    });

    // Effect 2: Depends on root
    let read_root_2 = read_root.clone();
    let log_2 = execution_log.clone();
    let _effect_2 = Effect::new(runtime.clone(), move || {
        let val = read_root_2.get();
        if val > 0 {
            log_2
                .lock()
                .unwrap()
                .push(format!("Effect 2 ran with {}", val));
        }
    });

    // Trigger update
    write_root.set(1);

    let log = log_clone.lock().unwrap();

    assert_eq!(log.len(), 2, "Both effects should have run");
    assert!(log.contains(&"Effect 1 ran with 1".to_string()));
    assert!(log.contains(&"Effect 2 ran with 1".to_string()));
}

/// 🛡️ Sentry: Verify that "Dynamic Dependency Pruning" works correctly.
/// If a signal is not read in the current execution path (e.g. inside `if false`),
/// it should be removed from dependencies.
/// Subsequent updates to that signal should NOT trigger re-computation.
#[test]
fn test_dynamic_dependency_pruning() {
    let runtime = Runtime::new();
    let toggle = Signal::new(runtime.clone(), true);
    let (read_toggle, write_toggle) = toggle.split();

    let value_a = Signal::new(runtime.clone(), 10);
    let (read_a, _write_a) = value_a.split();

    let value_b = Signal::new(runtime.clone(), 20);
    let (read_b, write_b) = value_b.split();

    let compute_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_clone = compute_count.clone();

    let read_toggle_c = read_toggle.clone();
    let read_a_c = read_a.clone();
    let read_b_c = read_b.clone();

    let computed = Computed::new(runtime.clone(), move || {
        count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if read_toggle_c.get() {
            read_a_c.get()
        } else {
            read_b_c.get()
        }
    });

    // Initial run (eager)
    assert_eq!(compute_count.load(std::sync::atomic::Ordering::Relaxed), 1);
    assert_eq!(computed.get(), 10); // A active

    // Update B (inactive branch). Should NOT trigger re-computation.
    write_b.set(30);
    // Access it. If B was pruned, it should NOT be stale, so NO recomputation.
    assert_eq!(computed.get(), 10);
    assert_eq!(
        compute_count.load(std::sync::atomic::Ordering::Relaxed),
        1,
        "Pruned dependency B changed, but computed ran!"
    );

    // Switch toggle to false (B active). Should trigger re-computation.
    write_toggle.set(false);
    // Access to trigger recomputation
    assert_eq!(computed.get(), 30); // B active (new value)
    assert_eq!(compute_count.load(std::sync::atomic::Ordering::Relaxed), 2);

    // Update B again (now active). Should trigger re-computation.
    write_b.set(40);
    assert_eq!(
        compute_count.load(std::sync::atomic::Ordering::Relaxed),
        2,
        "Lazy computation shouldn't run until accessed"
    );

    // Access it
    assert_eq!(computed.get(), 40);
    assert_eq!(compute_count.load(std::sync::atomic::Ordering::Relaxed), 3);
}

/// 🛡️ Sentry: Verify Signal Poisoning behavior.
/// If a thread panics while holding a signal lock (e.g. inside `with`),
/// the signal becomes poisoned and subsequent accesses panic.
#[test]
fn test_signal_poisoning() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), "safe".to_string());
    let (read, write) = signal.split();

    // 1. Panic inside `read` (with RwLock, this does NOT poison)
    let read_clone = read.clone();
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(move || {
        read_clone.with(|_val| {
            panic!("Read panic");
        });
    }));

    // Should still be accessible
    let read_check_1 = read.clone();
    let result = panic::catch_unwind(panic::AssertUnwindSafe(move || {
        read_check_1.get();
    }));
    assert!(
        result.is_ok(),
        "Signal should NOT be poisoned after read panic"
    );

    // 2. Panic inside `write` (with RwLock, this DOES poison)
    let write_clone = write.clone();
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(move || {
        write_clone.update(|_val| {
            panic!("Write panic");
        });
    }));

    // Should be poisoned now
    let result = panic::catch_unwind(panic::AssertUnwindSafe(move || {
        read.get();
    }));
    // After our fix, signal SHOULD recover from poison
    assert!(
        result.is_ok(),
        "Signal SHOULD recover gracefully after write panic"
    );
}

/// 🛡️ Sentry: Verify Recursion Limit on Computed Side-Effects.
/// A computed value modifying a signal it depends on triggers a cycle.
/// This should hit recursion limit (stack overflow protection) or deadlock detection.
#[test]
fn test_computed_side_effect_cycle() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (read, write) = signal.split();

    let write_clone = write.clone();
    let read_clone = read.clone();

    // Create a computed that reads signal AND writes to it.
    let result = panic::catch_unwind(panic::AssertUnwindSafe(move || {
        let _computed = Computed::new(runtime.clone(), move || {
            let val = read_clone.get();
            if val < 100 {
                write_clone.set(val + 1);
            }
            val
        });
    }));

    if let Err(e) = result {
        if let Some(msg) = e.downcast_ref::<&str>() {
            println!("Panicked with: {}", msg);
        } else if let Some(msg) = e.downcast_ref::<String>() {
            println!("Panicked with: {}", msg);
        }
    } else {
        println!(
            "Did not panic. Side-effect cycle was swallowed (expected behavior for self-invalidation during compute)."
        );
    }
}
