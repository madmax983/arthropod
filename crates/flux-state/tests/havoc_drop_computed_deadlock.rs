use flux_state::{Computed, Runtime, Signal};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
struct DropBombComputed {
    computed_handle: Option<flux_state::Computed<i32>>,
}

impl Drop for DropBombComputed {
    fn drop(&mut self) {
        if let Some(comp) = &self.computed_handle {
            // Tries to read the computed while the old value is being dropped
            // inside `WriteSignal::set()`. Since `set()` holds the write lock,
            // we will see what happens when reading the computed value inside drop.
            let _ = comp.get();
        }
    }
}

#[test]
#[should_panic(expected = "Deadlock detected")]
fn test_havoc_drop_computed_deadlock() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let runtime = Runtime::new();
        let signal = Signal::new(
            runtime.clone(),
            DropBombComputed {
                computed_handle: None,
            },
        );
        let (read, write) = signal.split();

        // Let's create a computed
        let computed = Computed::new(runtime.clone(), move || {
            // The computed value reads the signal!
            // Wait, but `read` returns a DropBombComputed.
            // That would drop it again? No, it returns a clone.
            // Let's just have it return an i32 to be simpler.
            read.with(|_| 42)
        });

        // Ensure it's evaluated
        assert_eq!(computed.get(), 42);

        write.update(|v| {
            v.computed_handle = Some(computed.clone());
        });

        write.set(DropBombComputed {
            computed_handle: None,
        });

        tx.send(()).unwrap();
    });

    if rx.recv_timeout(Duration::from_millis(500)).is_err() {
        panic!(
            "Deadlock detected: Dropping the old value inside the write lock blocked the thread."
        );
    }
}
