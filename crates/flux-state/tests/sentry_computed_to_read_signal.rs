use flux_state::{Computed, Effect, Runtime, Signal};
use std::sync::{Arc, Mutex};

#[test]
fn test_computed_to_read_signal_get() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 10);
    let (read, write) = signal.split();

    let computed = Computed::new(runtime.clone(), move || read.get() * 2);
    let read_signal_from_computed = computed.to_read_signal();

    // Initial state
    assert_eq!(read_signal_from_computed.get(), 20);
    assert_eq!(read_signal_from_computed.get_untracked(), 20);

    // Update root signal
    write.set(15);

    // Should reflect new value
    assert_eq!(read_signal_from_computed.get(), 30);
    assert_eq!(read_signal_from_computed.get_untracked(), 30);
}

#[test]
fn test_computed_to_read_signal_with() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), vec![1, 2, 3]);
    let (read, write) = signal.split();

    let computed = Computed::new(runtime.clone(), move || read.get());
    let read_signal_from_computed = computed.to_read_signal();

    // Initial state via with
    assert_eq!(read_signal_from_computed.with(|v| v.len()), 3);
    assert_eq!(read_signal_from_computed.with_untracked(|v| v.len()), 3);

    // Update root signal
    write.set(vec![1, 2, 3, 4, 5]);

    // Should reflect new value via with
    assert_eq!(read_signal_from_computed.with(|v| v.len()), 5);
    assert_eq!(read_signal_from_computed.with_untracked(|v| v.len()), 5);
}

#[test]
fn test_computed_to_read_signal_reactivity() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 5);
    let (read, write) = signal.split();

    let computed = Computed::new(runtime.clone(), move || read.get() + 5);
    let read_signal_from_computed = computed.to_read_signal();

    let effect_log = Arc::new(Mutex::new(Vec::new()));
    let log_clone = Arc::clone(&effect_log);

    let _effect = Effect::new(runtime.clone(), move || {
        log_clone
            .lock()
            .unwrap()
            .push(read_signal_from_computed.get());
    });

    // Effect should run initially
    assert_eq!(*effect_log.lock().unwrap(), vec![10]);

    // Update root signal
    write.set(15);

    // Effect should be triggered again because the read_signal's dependencies changed
    assert_eq!(*effect_log.lock().unwrap(), vec![10, 20]);
}
