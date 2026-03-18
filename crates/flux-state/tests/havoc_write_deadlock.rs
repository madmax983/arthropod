use flux_state::{Runtime, Signal};
use std::sync::{Arc, Barrier, mpsc};
use std::thread;
use std::time::Duration;

#[test]
#[should_panic(expected = "Deadlock detected")]
fn test_update_update_deadlock() {
    let runtime = Runtime::new();
    let signal_a = Signal::new(runtime.clone(), 10);
    let signal_b = Signal::new(runtime.clone(), 20);

    let (_, write_a1) = signal_a.clone().split();
    let (_, write_b1) = signal_b.clone().split();

    let (_, write_a2) = signal_a.clone().split();
    let (_, write_b2) = signal_b.clone().split();

    let barrier = Arc::new(Barrier::new(2));
    let (tx, rx) = mpsc::channel();

    let b1 = barrier.clone();
    let tx1 = tx.clone();

    // Thread 1: Update A -> Sleep -> Update B
    thread::spawn(move || {
        b1.wait();
        write_a1.update(|_| {
            thread::sleep(Duration::from_millis(50));
            write_b1.update(|_| {});
        });
        tx1.send(()).unwrap();
    });

    let b2 = barrier.clone();
    let tx2 = tx.clone();

    // Thread 2: Update B -> Sleep -> Update A
    thread::spawn(move || {
        b2.wait();
        write_b2.update(|_| {
            thread::sleep(Duration::from_millis(50));
            write_a2.update(|_| {});
        });
        tx2.send(()).unwrap();
    });

    // Wait for completion or timeout
    for _ in 0..2 {
        if rx.recv_timeout(Duration::from_millis(500)).is_err() {
            panic!("Deadlock detected: Threads timed out waiting for locks.");
        }
    }
}
