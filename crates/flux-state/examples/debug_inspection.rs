//! Debug Inspection Demo
//!
//! This example demonstrates how to use the `Debug` implementation for Signals and Computed values
//! to inspect their current state. This is useful for debugging reactive applications.
//!
//! Run with: `cargo run -p flux-state --example debug_inspection`

use flux_state::{Computed, Runtime, Signal};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MyData {
    x: i32,
    y: String,
}

fn main() {
    let runtime = Runtime::new();

    println!("--- Testing Signal Debug ---");
    let signal = Signal::new(runtime.clone(), 42);
    println!("Signal<i32>: {:?}", signal);

    let data = MyData {
        x: 10,
        y: "test".to_string(),
    };
    let signal_struct = Signal::new(runtime.clone(), data);
    println!("Signal<MyData>: {:?}", signal_struct);

    let (read, write) = signal.split();
    println!("ReadSignal: {:?}", read);
    println!("WriteSignal: {:?}", write);

    println!("\n--- Testing Computed Debug ---");
    let read_clone = read.clone();
    let computed = Computed::new(runtime.clone(), move || read_clone.get() * 2);
    println!("Computed (initial): {:?}", computed);

    println!("Updating Signal...");
    write.set(10);
    // Computed is now stale internally until accessed, but Debug impl forces update
    println!("Computed (after update, before read): {:?}", computed);

    println!("\n--- Testing Locked Signal ---");
    // To demonstrate deadlock prevention, we intentionally hold a lock on a signal
    // in a separate thread while trying to debug print it from the main thread.

    let runtime2 = Runtime::new();
    let signal_lock = Signal::new(runtime2.clone(), 0);
    let (_r_lock, w_lock) = signal_lock.split();

    let w_clone = w_lock.clone();
    let r_clone = _r_lock.clone();

    let barrier = Arc::new(std::sync::Barrier::new(2));
    let b1 = barrier.clone();

    thread::spawn(move || {
        w_clone.update(|val| {
            // Hold lock
            *val = 1;
            b1.wait(); // Signal main thread we have lock
            thread::sleep(Duration::from_millis(100)); // Hold it a bit longer
        });
    });

    barrier.wait(); // Wait for thread to acquire lock

    // Now try to debug print r_lock
    // Without `try_lock`, this would block or deadlock.
    // With `try_lock`, it should print `<locked>` immediately.
    println!("Locked ReadSignal: {:?}", r_clone);

    println!("\n--- Demo Complete ---");
}
