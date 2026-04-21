use flux_state::{Computed, Effect, Runtime, Signal};
use std::sync::{Arc, Mutex};

#[test]
fn test_dynamic_dependency_pruning() {
    let runtime = Runtime::new();

    let condition = Signal::new(runtime.clone(), true);
    let signal_a = Signal::new(runtime.clone(), 10);
    let signal_b = Signal::new(runtime.clone(), 20);

    let (read_cond, write_cond) = condition.split();
    let (read_a, write_a) = signal_a.split();
    let (read_b, write_b) = signal_b.split();

    // Computed: depends on Cond, and EITHER A or B
    let computed = Computed::new(runtime.clone(), move || {
        if read_cond.get() {
            read_a.get()
        } else {
            read_b.get()
        }
    });

    let run_count = Arc::new(Mutex::new(0));
    let run_count_clone = run_count.clone();

    // Effect to track updates to Computed
    let computed_clone = computed.clone();
    let _keep_alive = Effect::new(runtime.clone(), move || {
        let _ = computed_clone.get();
        *run_count_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
    });

    // Initial state: Cond=true, A=10. Computed=10.
    assert_eq!(computed.get(), 10);
    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1,
        "Effect should run initially"
    );

    // 1. Update B (inactive branch). Should NOT trigger update.
    write_b.set(21);
    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1,
        "Updating inactive dependency B should not trigger effect"
    );
    assert_eq!(computed.get(), 10, "Computed should still be 10");

    // 2. Update Condition -> false. Should trigger update.
    // Computed switches to B (21).
    write_cond.set(false);
    assert_eq!(computed.get(), 21);
    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        2,
        "Updating condition should trigger effect"
    );

    // 3. Update A (inactive branch). Should NOT trigger update.
    write_a.set(11);
    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        2,
        "Updating inactive dependency A should not trigger effect"
    );
    assert_eq!(computed.get(), 21, "Computed should still be 21");

    // 4. Update B (active branch). Should trigger update.
    write_b.set(22);
    assert_eq!(computed.get(), 22);
    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        3,
        "Updating active dependency B should trigger effect"
    );
}
