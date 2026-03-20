use flux_state::{Runtime, Signal};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
struct DropBomb {
    read_handle: Option<flux_state::ReadSignal<DropBomb>>,
}

impl Drop for DropBomb {
    fn drop(&mut self) {
        if let Some(read) = &self.read_handle {
            // Tries to read the signal while the old value is being dropped
            // inside `WriteSignal::set()`. Since `set()` holds the write lock,
            // this read lock will cause a deadlock.
            let _ = read.get_untracked();
        }
    }
}

#[test]
#[should_panic(expected = "Deadlock detected")]
fn test_havoc_drop_deadlock() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), DropBomb { read_handle: None });
        let (read, write) = signal.split();

        write.update(|v| {
            v.read_handle = Some(read.clone());
        });

        // Now the current value inside `signal` has a `read_handle`.
        // When we call `set()`, the old value will be dropped inside the write lock.
        write.set(DropBomb { read_handle: None });

        tx.send(()).unwrap();
    });

    if rx.recv_timeout(Duration::from_millis(500)).is_err() {
        panic!(
            "Deadlock detected: Dropping the old value inside the write lock blocked the thread."
        );
    }
}
