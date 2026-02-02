use flux_state::{Computed, Effect, Runtime, Signal};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
fn test_signal_reentrancy_deadlock() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime, 0);
    let (read, _) = signal.split();

    let (tx, rx) = mpsc::channel();
    let read_clone = read.clone();

    thread::spawn(move || {
        // Attempt reentrancy: with -> get (which uses with)
        // This should deadlock because standard Mutex is not reentrant.
        read_clone.with(|_val| {
            let _ = read_clone.get();
        });
        tx.send(()).unwrap();
    });

    // If we receive a message, it means it didn't deadlock (Fail).
    // If we timeout, it means it deadlocked (Pass).
    match rx.recv_timeout(Duration::from_millis(500)) {
        Ok(_) => panic!("Signal reentrancy successfully completed (Expected Deadlock!)"),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Success! The thread is hung.
            println!("Deadlock confirmed: Signal reentrancy hangs.");
        }
        Err(e) => panic!("Channel error: {:?}", e),
    }
}

#[test]
fn test_computed_reentrancy_deadlock() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (read, _) = signal.split();

    let read_c = read.clone();
    let computed = Computed::new(runtime, move || read_c.get());

    let (tx, rx) = mpsc::channel();
    let comp_clone = computed.clone();

    thread::spawn(move || {
        // Attempt reentrancy: with -> get (which uses with)
        comp_clone.with(|_val| {
            let _ = comp_clone.get();
        });
        tx.send(()).unwrap();
    });

    match rx.recv_timeout(Duration::from_millis(500)) {
        Ok(_) => panic!("Computed reentrancy successfully completed (Expected Deadlock!)"),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            println!("Deadlock confirmed: Computed reentrancy hangs.");
        }
        Err(e) => panic!("Channel error: {:?}", e),
    }
}

#[test]
fn test_effect_write_read_cycle_deadlock() {
    let runtime = Runtime::new();
    let signal_a = Signal::new(runtime.clone(), "A");
    let (read_a, write_a) = signal_a.split();

    let signal_b = Signal::new(runtime.clone(), "B");
    let (read_b, _write_b) = signal_b.split();

    // Setup an effect that reads both signals.
    // It will run immediately.
    let r_a = read_a.clone();
    let r_b = read_b.clone();
    let _effect = Effect::new(runtime.clone(), move || {
        let _ = r_a.get();
        let _ = r_b.get();
    });

    let (tx, rx) = mpsc::channel();

    // We want to trigger a deadlock.
    // Thread holds Lock B (via with).
    // Thread updates Signal A.
    // Signal A update triggers Effect.
    // Effect tries to read Signal B.
    // Effect tries to Lock B -> Deadlock.

    let w_a_clone = write_a.clone();
    let r_b_clone = read_b.clone();

    thread::spawn(move || {
        r_b_clone.with(|_val_b| {
            // We hold B lock.
            // Now set A.
            w_a_clone.set("A2");
            // set() -> notify() -> flush_effects() -> run_effect()
            // Effect reads A (ok) then reads B (DEADLOCK).
        });
        tx.send(()).unwrap();
    });

    match rx.recv_timeout(Duration::from_millis(500)) {
        Ok(_) => panic!("Effect cycle successfully completed (Expected Deadlock!)"),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            println!("Deadlock confirmed: Effect write/read cycle hangs.");
        }
        Err(e) => panic!("Channel error: {:?}", e),
    }
}
