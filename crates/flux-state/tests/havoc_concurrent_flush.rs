use flux_state::{Effect, Runtime, Signal};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_havoc_concurrent_flush_race() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (read, write) = signal.split();

    let done = Arc::new(Mutex::new(false));
    let done_clone = done.clone();

    // Effect that just sleeps to simulate work
    let _effect = Effect::new(runtime.clone(), move || {
        let _val = read.get();
        thread::sleep(Duration::from_millis(100));
        *done_clone.lock().unwrap() = true;
    });

    // Reset done flag after initial run
    *done.lock().unwrap() = false;

    // Thread 1 writes, which queues the effect, then manually calls run_effects
    // Actually, write.set() automatically calls flush_effects() right now.
    // Let's spawn a thread that sets the value.
    let w_clone = write.clone();
    let t1 = thread::spawn(move || {
        w_clone.set(1); // This will block for 100ms inside the effect
    });

    // Give t1 a moment to grab the effects and start sleeping
    thread::sleep(Duration::from_millis(20));

    // Thread 2 writes a new value.
    // It will push the effect to pending_effects, then call flush_effects.
    // But t1 is currently processing!
    // Wait, let's see if t2 blocks until the effect is done.
    write.set(2);

    // If t2 returns immediately while the effect is still running, it assumes the effect finished.
    // But the effect might not have run for `set(2)` yet.
    // Let's assert that the effect ran for `set(2)`!
    // Wait, if it didn't block, `done` might still be false.
    let is_done = *done.lock().unwrap();
    // Ensure that it's done, proving the race condition is fixed.
    assert!(
        is_done,
        "Race condition: set() returned before its effects finished running!"
    );

    t1.join().unwrap();
}
