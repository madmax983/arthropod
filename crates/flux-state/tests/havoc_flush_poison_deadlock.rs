use flux_state::{Effect, Runtime, Signal};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct PanicOnDrop;

impl Drop for PanicOnDrop {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            panic!("Poisoning the runtime lock!");
        }
    }
}

#[test]
fn test_flush_guard_poison_deadlock() {
    let runtime = Runtime::new();

    // Signal used to control Thread A's effect
    let signal_a = Signal::new(runtime.clone(), 0);
    let (read_a, write_a) = signal_a.split();

    // Signal used to poison `inner` from Thread B
    let signal_b = Signal::new(
        runtime.clone(),
        Arc::new(PanicOnDrop) as Arc<dyn std::any::Any + Send + Sync>,
    );
    let (_read_b, write_b) = signal_b.split();

    let sync_effect_started = Arc::new(Mutex::new(false));
    let sync_inner_poisoned = Arc::new(Mutex::new(false));

    let sync_eff_clone = sync_effect_started.clone();
    let sync_pois_clone = sync_inner_poisoned.clone();
    let read_a_clone = read_a.clone();

    // Effect that runs when signal_a changes
    let _effect = Effect::new(runtime.clone(), move || {
        let val = read_a_clone.get();
        if val == 1 {
            // Signal that Thread A is now inside `flush_effects` -> `process_effect_batch`
            *sync_eff_clone.lock().unwrap() = true;

            // Wait for Thread B to poison `inner`
            while !*sync_pois_clone.lock().unwrap() {
                thread::sleep(Duration::from_millis(5));
            }

            // NOTE: We also panic here so that we don't try to lock the poisoned Mutex normally
            // inside flush_effects when inner = self.inner.lock() is called after process_effect_batch.
            panic!("Intentional panic to trigger drop");
        }
    });

    // Wait for initial effect to run
    thread::sleep(Duration::from_millis(50));

    // Thread A: triggers flush_effects, which starts running the effect
    let write_a_clone = write_a.clone();
    let thread_a = thread::spawn(move || {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            write_a_clone.set(1);
        }));
    });

    // Wait until Thread A is executing the effect (it holds `flushing_thread` but NOT `inner`)
    while !*sync_effect_started.lock().unwrap() {
        thread::sleep(Duration::from_millis(5));
    }

    // Thread B: Now we poison `inner`
    let write_b_clone = write_b.clone();
    let thread_b = thread::spawn(move || {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Overwriting `signal_b` causes the old `PanicOnDrop` value to be dropped
            // while `inner` is locked inside `set_signal` -> `inner.signals.insert`
            write_b_clone.set(Arc::new(()) as Arc<dyn std::any::Any + Send + Sync>);
        }));
    });

    let _ = thread_b.join(); // Wait for poison to happen

    // Signal Thread A to finish (by panicking)
    *sync_inner_poisoned.lock().unwrap() = true;

    let _ = thread_a.join();

    // Now Thread A's `FlushingGuard` dropped.
    // Since `inner` is poisoned, `if let Ok(mut inner) = self.runtime.inner.lock()` failed.
    // So `inner.flushing_thread` is STILL `Some(Thread A)`.

    // Thread C: Attempts to set a signal, which calls `flush_effects`.
    // It should loop forever (deadlock) in `flush_effects`!
    let write_a_clone2 = write_a.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let _thread_c = thread::spawn(move || {
        // We DON'T catch unwind here. If the lock is poisoned, `set_signal` might panic BEFORE `flush_effects`.
        // Let's use `write.set()` which unwraps.
        // Oh wait! `write_a_clone2.set(2)` calls `set_signal`, which starts with:
        // let mut inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        // It does NOT panic if the mutex is poisoned, it recovers!
        // So `set_signal` succeeds, and then it calls `flush_effects`.
        // `flush_effects` also recovers!
        // BUT it hits `while let Some(flushing) = inner.flushing_thread`
        // Since `flushing_thread` is STILL `Some(Thread A)`, it waits forever!
        write_a_clone2.set(2);
        let _ = tx.send(());
    });

    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(_) => {
            // Thread C finished! The deadlock was fixed.
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            panic!(
                "Thread C hung indefinitely! Deadlock in `flush_effects` due to poisoned mutex not being handled in `FlushingGuard::drop`."
            );
        }
        Err(_) => panic!("Channel closed unexpectedly"),
    }
}
