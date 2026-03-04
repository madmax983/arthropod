use flux_state::{Runtime, Signal};

#[test]
fn test_untested_update_logic() {
    let runtime = Runtime::new();
    let count: Signal<i32> = Signal::new(runtime, 0);
    let (read, write) = count.split();

    write.update(|val| *val += 5);
    assert_eq!(read.get(), 5);
}

#[test]
fn test_update_triggers_effects() {
    use flux_state::Effect;
    use std::sync::{Arc, Mutex};

    let runtime = Runtime::new();
    let count: Signal<i32> = Signal::new(runtime.clone(), 0);
    let (read, write) = count.split();

    let run_count = Arc::new(Mutex::new(0));
    let run_count_clone = run_count.clone();

    let read_clone = read.clone();
    let _effect = Effect::new(runtime.clone(), move || {
        read_clone.get();
        *run_count_clone.lock().unwrap() += 1;
    });

    assert_eq!(*run_count.lock().unwrap(), 1);

    write.update(|val| *val += 5);
    assert_eq!(*run_count.lock().unwrap(), 2);
    assert_eq!(read.get(), 5);
}

#[test]
fn test_get_computed_if_fresh_returns_none() {
    let runtime = flux_state::Runtime::new();
    let signal = flux_state::Signal::new(runtime.clone(), 0);
    let (read, write) = signal.split();

    let computed = flux_state::Computed::new(runtime.clone(), move || read.get() * 2);

    assert_eq!(computed.get(), 0);

    // After updating the signal, the computed node is marked as stale.
    // It should not be recalculated eagerly because we are lazily updating.
    write.set(1);

    // If we call with_untracked, it will recalculate and return 2.
    // Let's verify that get_computed_if_fresh returns None internally when stale.
    // The public API doesn't expose `get_computed_if_fresh`, but we can verify it
    // indirectly by checking that the stale status correctly forces a recompute
    // when using `with_untracked`
    let val = computed.with_untracked(|v| *v);
    assert_eq!(val, 2);
}
