use flux_state::{Computed, Effect, Runtime, Signal};
use std::sync::{Arc, Mutex};

#[test]
fn test_write_signal_update_triggers_subscribers() {
    let runtime = Runtime::new();
    let count = Signal::new(runtime.clone(), vec![1, 2, 3]);
    let (read_count, write_count) = count.split();

    // Subscriber 1: Computed
    let read_clone1 = read_count.clone();
    let len_computed = Computed::new(runtime.clone(), move || read_clone1.with(|v| v.len()));

    // Subscriber 2: Effect
    let log = Arc::new(Mutex::new(Vec::new()));
    let log_clone = log.clone();
    let read_clone2 = read_count.clone();
    let _effect = Effect::new(runtime.clone(), move || {
        log_clone
            .lock()
            .unwrap()
            .push(read_clone2.with(|v| v.clone()));
    });

    // Initial state checks
    assert_eq!(len_computed.get(), 3);
    assert_eq!(
        *log.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        vec![vec![1, 2, 3]]
    );

    // Perform an update using WriteSignal::update
    write_count.update(|v| {
        v.push(4);
        v.push(5);
    });

    // Verify updates propagated
    assert_eq!(len_computed.get(), 5);
    assert_eq!(
        *log.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        vec![vec![1, 2, 3], vec![1, 2, 3, 4, 5]]
    );
}
