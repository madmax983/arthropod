use flux_state::{Runtime, Signal};
use std::thread;

#[test]
fn test_signal_sync_across_threads() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 42);
    let (read, _write) = signal.split();

    // Verify ReadSignal<T> is Sync when T: Send + Sync
    let handle = thread::spawn(move || {
        read.get() // Access from different thread
    });

    assert_eq!(handle.join().unwrap(), 42);
}

#[test]
fn test_signal_with_sync_type() {
    // i32 is Send + Sync.
    // Signal<i32> should be Sync.
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 42);
    let (read1, _) = signal.split();

    // Clone for the second thread
    let read2 = read1.clone();

    // Both references should work from different threads
    let handle = thread::spawn(move || read2.get());

    assert_eq!(read1.get(), 42);
    assert_eq!(handle.join().unwrap(), 42);
}

// NOTE: Signal<RefCell<T>> is no longer Sync because we switched from Mutex to RwLock
// to prevent deadlocks on concurrent reads of Sync types.
// If you need to share non-Sync types, wrap them in a Mutex yourself.

#[test]
fn test_write_signal_across_threads() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (read, write) = signal.split();

    let write_clone = write.clone();
    let handle = thread::spawn(move || {
        write_clone.set(100);
    });

    handle.join().unwrap();
    assert_eq!(read.get(), 100);
}

#[test]
fn test_concurrent_updates() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (read, write) = signal.split();

    let mut handles = vec![];

    for _ in 0..10 {
        let write_clone = write.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                write_clone.update(|v| *v += 1);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Signal::update locks the write lock, applies the closure, and releases.
    // So concurrent updates should be safe and atomic regarding the value modification.
    assert_eq!(read.get(), 1000);
}
