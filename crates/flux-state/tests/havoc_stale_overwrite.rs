use flux_state::{Computed, Runtime, Signal};
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn test_havoc_stale_overwrite() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (read_sig, write_sig) = signal.split();

    let barrier_start = Arc::new(Barrier::new(2));
    let barrier_end = Arc::new(Barrier::new(2));

    let b_start_c = barrier_start.clone();
    let b_end_c = barrier_end.clone();

    let read_sig_c = read_sig.clone();

    // We spawn a thread to create the computed value so it can block
    // while the main thread updates the signal.
    let runtime_c = runtime.clone();
    let t1 = thread::spawn(move || {
        Computed::new(runtime_c, move || {
            let val = read_sig_c.get();
            if val == 0 {
                b_start_c.wait();
                b_end_c.wait();
            }
            val
        })
    });

    // Main thread acts as thread 2
    barrier_start.wait(); // wait for t1 to read the old value
    write_sig.set(1); // modify signal to 1
    barrier_end.wait(); // allow t1 to finish computation

    let computed = t1.join().unwrap();

    // Now, computed should be stale because signal was set to 1.
    // When we get it, it should evaluate to 1.
    let final_val = computed.get();
    assert_eq!(
        final_val, 1,
        "Havoc: Computed value is stale but flag was overwritten!"
    );
}
