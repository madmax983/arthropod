use flux_state::{Effect, Runtime, Signal};
use std::panic;

#[test]
fn test_panic_pollution() {
    // 🛡️ Sentry: This test verifies that a panic inside an effect does not
    // pollute the thread-local tracking context.

    let runtime = Runtime::new();
    let signal_a = Signal::new(runtime.clone(), 0);
    let (_read_a, _write_a) = signal_a.split();

    // 1. Trigger a panic inside an effect and catch it.
    let _panic_effect = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Effect::new(runtime.clone(), || {
            panic!("Boom!");
        })
    }));

    // 2. Create a new signal B and read it.
    // If context is polluted, B will register the panicked effect as a subscriber.
    let signal_b = Signal::new(runtime.clone(), 100);
    let (read_b, write_b) = signal_b.split();

    // This read should be untracked.
    let _ = read_b.get();

    // 3. Update B.
    // If context was polluted, this will try to re-run the panicked effect.
    // We expect this NOT to panic.
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        write_b.set(200);
    }));

    assert!(
        result.is_ok(),
        "Writing to an unrelated signal caused a panic! The runtime context is polluted."
    );
}
