use flux_state::{Runtime, Signal};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
#[should_panic(expected = "Deadlock detected")]
fn test_havoc_write_write_deadlock() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), 10);
        let (_, write) = signal.split();

        write.update(|_| {
            write.set(20); // Deadlock: holding write lock and trying to get write lock on the same thread
        });

        tx.send(()).unwrap();
    });

    if rx.recv_timeout(Duration::from_millis(500)).is_err() {
        panic!("Deadlock detected: Double write lock blocked the thread.");
    }
}
