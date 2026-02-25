use flux_state::{Runtime, Signal};
use std::sync::{Arc, Barrier, mpsc};
use std::thread;
use std::time::Duration;

#[test]
fn test_read_read_deadlock() {
    let runtime = Runtime::new();
    let signal_a = Signal::new(runtime.clone(), 10);
    let signal_b = Signal::new(runtime.clone(), 20);

    let barrier = Arc::new(Barrier::new(2));
    let (tx, rx) = mpsc::channel();

    let a = signal_a.clone();
    let b = signal_b.clone();
    let b_barrier = barrier.clone();
    let tx1 = tx.clone();

    // Thread 1: Read A -> Sleep -> Read B
    thread::spawn(move || {
        b_barrier.wait(); // Synchronize start
        a.with(|_| {
            // Hold lock on A
            thread::sleep(Duration::from_millis(50));
            // Try to acquire lock on B
            b.with(|_| {
                // Success
            });
        });
        tx1.send("Thread 1 success").unwrap();
    });

    let a = signal_a.clone();
    let b = signal_b.clone();
    let b_barrier = barrier.clone();
    let tx2 = tx.clone();

    // Thread 2: Read B -> Sleep -> Read A
    thread::spawn(move || {
        b_barrier.wait(); // Synchronize start
        b.with(|_| {
            // Hold lock on B
            thread::sleep(Duration::from_millis(50));
            // Try to acquire lock on A
            a.with(|_| {
                // Success
            });
        });
        tx2.send("Thread 2 success").unwrap();
    });

    // We expect 2 success messages.
    // If deadlocked, we won't get them.

    // Message 1
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(_) => {}
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("Deadlock detected: Thread 1 or 2 timed out waiting for lock.")
        }
        Err(e) => panic!("Channel error: {:?}", e),
    }

    // Message 2
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(_) => {}
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("Deadlock detected: Second thread timed out waiting for lock.")
        }
        Err(e) => panic!("Channel error: {:?}", e),
    }
}
