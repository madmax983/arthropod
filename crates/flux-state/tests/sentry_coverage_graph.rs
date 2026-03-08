use flux_state::{Runtime, Signal};
use std::sync::Arc;

#[test]
fn test_signal_debug() {
    let runtime = Runtime::new();
    let count = Signal::new(runtime.clone(), 42);
    let debug_str = format!("{:?}", count);
    assert!(debug_str.contains("Signal(id:"));
    assert!(debug_str.contains("value: 42"));

    let (read, write) = count.clone().split();
    let debug_str_read = format!("{:?}", read);
    assert!(debug_str_read.contains("ReadSignal(id:"));
    assert!(debug_str_read.contains("value: 42"));

    let debug_str_write = format!("{:?}", write);
    assert!(debug_str_write.contains("WriteSignal(id:"));
    assert!(debug_str_write.contains("value: 42"));
}

#[test]
fn test_signal_runtime_getter() {
    let runtime = Runtime::new();
    let count = Signal::new(runtime.clone(), 42);
    assert!(Arc::ptr_eq(count.runtime(), &runtime));

    let (read, _) = count.split();
    assert!(Arc::ptr_eq(read.runtime(), &runtime));
}

#[test]
fn test_signal_debug_locked() {
    let runtime = Runtime::new();
    let count = Signal::new(runtime.clone(), 42);
    let (read, write) = count.clone().split();

    write.update(|_| {
        let debug_str = format!("{:?}", count);
        assert!(debug_str.contains("value: <locked>"));

        let debug_str_read = format!("{:?}", read);
        assert!(debug_str_read.contains("value: <locked>"));

        let debug_str_write = format!("{:?}", write);
        assert!(debug_str_write.contains("value: <locked>"));
    });
}

#[test]
fn test_computed_with_untracked_stale() {
    let runtime = flux_state::Runtime::new();
    let count = flux_state::Signal::new(runtime.clone(), 10);
    let (read, write) = count.split();

    let computed = flux_state::Computed::new(runtime.clone(), move || read.get() * 2);

    assert_eq!(computed.get(), 20);

    write.set(15);

    let val = computed.with_untracked(|v| *v);
    assert_eq!(val, 30);
}

#[test]
fn test_computed_with_stale() {
    let runtime = flux_state::Runtime::new();
    let count = flux_state::Signal::new(runtime.clone(), 10);
    let (read, write) = count.split();

    let computed = flux_state::Computed::new(runtime.clone(), move || read.get() * 2);

    assert_eq!(computed.get(), 20);

    write.set(20);

    let val = computed.with(|v| *v);
    assert_eq!(val, 40);
}
