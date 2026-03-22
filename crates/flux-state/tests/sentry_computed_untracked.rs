use flux_state::{Computed, Effect, Runtime, Signal};
use std::sync::Arc;

#[test]
fn test_computed_untracked_and_read_signal_methods() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), vec![1, 2, 3]);
    let (read, write) = signal.split();

    let computed = Computed::new(runtime.clone(), move || read.get());

    // First it is fresh
    assert_eq!(computed.with_untracked(|v| v.len()), 3);

    // Make it stale
    write.set(vec![1]);

    // Now it should compute but NOT track
    let read_val = computed.with_untracked(|v| v.len());
    assert_eq!(read_val, 1);

    // Read again, now it's fresh again
    assert_eq!(computed.with_untracked(|v| v.len()), 1);

    // Convert to read signal
    let read_computed = computed.to_read_signal();

    // Now test that reading from the read_computed does what we expect
    assert_eq!(read_computed.get().len(), 1);
    write.set(vec![1, 2]);
    assert_eq!(read_computed.get().len(), 2);

    // Test with
    assert_eq!(read_computed.with(|v| v.len()), 2);

    // Test with_untracked
    assert_eq!(read_computed.with_untracked(|v| v.len()), 2);
    assert_eq!(read_computed.get_untracked().len(), 2);

    write.set(vec![1, 2, 3]);
    assert_eq!(read_computed.get_untracked().len(), 3);

    // Test runtime
    assert!(Arc::ptr_eq(read_computed.runtime(), &runtime));
}

#[test]
fn test_effect_cleanup_and_drop() {
    let runtime = Runtime::new();
    let count = Signal::new(runtime.clone(), 0);
    let (read, write) = count.split();

    let _effect = Effect::new(runtime.clone(), move || {
        let _ = read.get();
    });

    write.set(1);
}
