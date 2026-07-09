use flux_state::{Runtime, Signal};
use std::thread;
use std::time::Duration;

#[test]
fn havoc_rwlock_downgrade_deadlock() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 10);
    let (read, write) = signal.split();

    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        write.update(|val| {
            let _ = read.get();
            *val = 11;
        });
        tx.send(()).unwrap();
    });

    let res = rx.recv_timeout(Duration::from_millis(100));
    assert!(
        matches!(res, Err(std::sync::mpsc::RecvTimeoutError::Disconnected)),
        "Test failed: ReadSignal::get inside WriteSignal::update did NOT deadlock!"
    );
}

#[test]
fn havoc_rwlock_downgrade_deadlock_untracked() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 10);
    let (read, write) = signal.split();

    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        write.update(|val| {
            let _ = read.get_untracked();
            *val = 11;
        });
        tx.send(()).unwrap();
    });

    let res = rx.recv_timeout(Duration::from_millis(100));
    assert!(
        matches!(res, Err(std::sync::mpsc::RecvTimeoutError::Disconnected)),
        "Test failed: ReadSignal::get_untracked inside WriteSignal::update did NOT deadlock!"
    );
}

#[test]
fn havoc_rwlock_double_write_deadlock() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 10);
    let (_, write) = signal.split();

    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        write.update(|val| {
            write.set(11);
            *val = 11;
        });
        tx.send(()).unwrap();
    });

    let res = rx.recv_timeout(Duration::from_millis(100));
    assert!(
        matches!(res, Err(std::sync::mpsc::RecvTimeoutError::Disconnected)),
        "Test failed: WriteSignal::set inside WriteSignal::update did NOT deadlock!"
    );
}
