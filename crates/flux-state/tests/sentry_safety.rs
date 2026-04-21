use flux_state::{Computed, Effect, Runtime, Signal};
use std::sync::{Arc, Mutex};

#[test]
fn test_signal_with_untracked() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 10);
    let (read, write) = signal.split();

    // Setup an effect to detect tracking
    let run_count = Arc::new(Mutex::new(0));
    let run_count_clone = run_count.clone();

    // First run tracks dependencies
    let _keep_alive = Effect::new(runtime.clone(), move || {
        // This should NOT track dependency on signal
        let _ = read.with_untracked(|v| *v);
        *run_count_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
    });

    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1,
        "Effect should run once initially"
    );

    // Update signal
    write.set(20);

    // Should NOT have run again because the read was untracked
    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1,
        "Effect should not run on update"
    );
}

#[test]
fn test_computed_with_untracked() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 10);
    let (read, write) = signal.split();

    let computed = Computed::new(runtime.clone(), move || read.get() * 2);

    let run_count = Arc::new(Mutex::new(0));
    let run_count_clone = run_count.clone();

    let computed_clone = computed.clone();
    let _keep_alive = Effect::new(runtime.clone(), move || {
        // This should NOT track dependency on computed
        let _ = computed_clone.with_untracked(|v| *v);
        *run_count_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
    });

    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1,
        "Effect should run once initially"
    );

    // Update signal -> marks computed stale
    write.set(20);

    // Computed value is stale, but Effect should NOT have been notified because it untracked-read Computed.
    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1,
        "Effect should not run on update"
    );

    // Verify computed updated correctly when accessed
    assert_eq!(computed.get(), 40, "Computed should return updated value");
}

#[test]
fn test_computed_lazy_update() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 10);
    let (read, write) = signal.split();

    let compute_count = Arc::new(Mutex::new(0));
    let compute_count_clone = compute_count.clone();

    let computed = Computed::new(runtime.clone(), move || {
        *compute_count_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
        read.get() * 2
    });

    // Initial compute happens on creation
    assert_eq!(
        *compute_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1,
        "Should compute on creation"
    );
    assert_eq!(computed.get(), 20);
    assert_eq!(
        *compute_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1,
        "Should use memoized value"
    );

    // Update signal
    write.set(20);

    // Should NOT recompute yet (lazy)
    assert_eq!(
        *compute_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1,
        "Should be lazy (not recomputed yet)"
    );

    // Read value
    assert_eq!(computed.get(), 40);
    // Now it should have recomputed
    assert_eq!(
        *compute_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        2,
        "Should recompute on access"
    );

    // Read again
    assert_eq!(computed.get(), 40);
    // Should use memoized
    assert_eq!(
        *compute_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        2,
        "Should use memoized value again"
    );
}

#[test]
fn test_signal_with_parity() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), "test".to_string());

    // Verify Signal::with works same as ReadSignal::with
    let val1 = signal.with(|s| s.len());

    let (read, _) = signal.split();
    let val2 = read.with(|s| s.len());

    assert_eq!(val1, val2);
    assert_eq!(val1, 4);
}
