use flux_state::{Effect, Runtime, Signal};
use std::panic;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::thread;

#[test]
fn test_zombie_effect_pollution() {
    let runtime = Runtime::new();

    // Side channel to detect zombie execution
    let run_count = Arc::new(AtomicUsize::new(0));
    let r_c = run_count.clone();

    // Signal to act as bait
    let bait = Signal::new(runtime.clone(), 0);
    let (read_bait, write_bait) = bait.split();

    let r_runtime = runtime.clone();
    let r_read_bait = read_bait.clone();

    // Run everything in a thread to isolate panic but keep runtime shared
    let handle = thread::spawn(move || {
        // 1. Attempt to create an effect that panics
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            Effect::new(r_runtime.clone(), move || {
                // If this runs again, increment counter
                let count = r_c.fetch_add(1, Ordering::SeqCst);

                // Panic on first run (we can use run_count to gate)
                // run_count was 0, now 1.
                if count == 0 {
                    panic!("Boom");
                }
            });
        }));

        // Effect panicked. run_count is 1.
        // Thread stack "should" be clean, but it's polluted with the Effect ID
        // because `pop_context` was skipped during panic.

        // 2. Access the bait signal.
        // Because the thread thinks it's still running the Effect,
        // it adds the Effect as a subscriber to `bait`.
        // This is the CRITICAL failure point: Unintended dependency tracking.
        let _ = r_read_bait.get();
    });

    handle.join().unwrap();

    assert_eq!(
        run_count.load(Ordering::SeqCst),
        1,
        "Initial run + panic occurred"
    );

    // 3. Update the bait.
    // If pollution happened, the zombie effect is listening to `bait`.
    write_bait.set(10);

    // 4. Check if zombie ran.
    // If it ran, run_count should be 2.
    // If it didn't run (correct behavior), run_count should be 1.
    assert_eq!(
        run_count.load(Ordering::SeqCst),
        1,
        "Zombie effect should NOT run. The system is robust."
    );
}
