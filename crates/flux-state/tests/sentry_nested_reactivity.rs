use flux_state::{Computed, Effect, Runtime, Signal};
use std::panic;
use std::sync::{Arc, Mutex};

#[test]
fn test_signal_of_signals() {
    let runtime = Runtime::new();

    let inner1 = Signal::new(runtime.clone(), 10);
    let inner2 = Signal::new(runtime.clone(), 20);

    let outer = Signal::new(runtime.clone(), inner1.clone());
    let (read_outer, write_outer) = outer.split();

    let (_read_inner1, write_inner1) = inner1.clone().split();
    let (_read_inner2, write_inner2) = inner2.clone().split();

    // Computed depends on outer, then inner
    let read_outer_c = read_outer.clone();
    let computed = Computed::new(runtime.clone(), move || {
        let inner = read_outer_c.get(); // Depend on Outer
        let (read_inner, _) = inner.split();
        read_inner.get() // Depend on Current Inner
    });

    assert_eq!(computed.get(), 10);

    // 1. Update Inner 1. Should trigger computed.
    write_inner1.set(11);
    assert_eq!(computed.get(), 11);

    // 2. Switch Outer to Inner 2. Should trigger computed.
    write_outer.set(inner2.clone());
    assert_eq!(computed.get(), 20);

    // 3. Update Inner 1 (Old). Should NOT trigger computed (pruned).
    write_inner1.set(12);
    assert_eq!(computed.get(), 20);

    // 4. Update Inner 2 (New). Should trigger computed.
    write_inner2.set(21);
    assert_eq!(computed.get(), 21);
}

#[test]
fn test_effect_self_disposal() {
    let runtime = Runtime::new();
    let trigger = Signal::new(runtime.clone(), 0);
    let (read, write) = trigger.split();

    let effect_handle = Arc::new(Mutex::new(None::<Effect>));
    let handle_clone = effect_handle.clone();
    let run_count = Arc::new(Mutex::new(0));
    let count_clone = run_count.clone();

    let read_c = read.clone();
    let effect = Effect::new(runtime.clone(), move || {
        let val = read_c.get();
        *count_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;

        if val == 1 {
            // Dispose self safely
            // Note: `dispose_effect` acquires the runtime lock.
            // `run_effect` releases the lock before running this closure.
            // So this is safe from deadlocks.
            *handle_clone
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        }
    });

    *effect_handle
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(effect);

    // Run 0 (init)
    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1
    );

    // Run 1 (Trigger disposal)
    write.set(1);
    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        2
    );

    // Check if disposed
    assert!(
        effect_handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none()
    );

    // Run 2 (Should NOT run)
    write.set(2);
    assert_eq!(
        *run_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        2
    );
}

#[test]
fn test_recursion_limit_boundary() {
    let runtime = Runtime::new();
    let root = Signal::new(runtime.clone(), 0);
    let (read_root, _) = root.clone().split();

    // C1 -> Root
    // C2 -> C1
    // ...
    // C100 -> C99

    let read_root_c = read_root.clone();
    let mut current = Computed::new(runtime.clone(), move || read_root_c.get());
    for _ in 0..98 {
        // 1 + 98 = 99
        let prev = current.clone();
        current = Computed::new(runtime.clone(), move || prev.get());
    }
    // current is C99 (Depth 99).
    // Wait, let's recount.
    // i=0: prev=C1, new=C2.
    // i=97: prev=C98, new=C99.
    // So loop 0..98 produces C99.

    // One more for C100
    let prev = current.clone();
    let c100 = Computed::new(runtime.clone(), move || prev.get());

    // Force recompute of full chain
    let (_r, w) = root.clone().split();
    w.set(1);

    // Access C100.
    // Stack: [C100, C99, ..., C1] -> Depth 100.
    // Should succeed (Limit is 100).
    assert_eq!(c100.get(), 1);

    // Now add one more C101.
    let prev = c100.clone();
    let c101 = Computed::new(runtime.clone(), move || prev.get());

    // Update root again to force recompute chain
    w.set(2);

    // Access C101.
    // Stack: [C101, ..., C1] -> Depth 101.
    // Should panic.
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        c101.get();
    }));

    assert!(
        result.is_err(),
        "Should have panicked at depth 101 during recomputation"
    );
}
