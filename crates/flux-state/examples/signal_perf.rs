use flux_state::{Computed, Runtime, Signal};
use std::time::Instant;

fn main() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), 0);
    let (read, _write) = signal.split();

    let iterations = 10_000_000;

    // Warm up
    for _ in 0..1000 {
        let _ = read.get();
    }

    // Benchmark Signal Read
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = read.get();
    }
    let duration = start.elapsed();
    println!("Signal::get ({} iterations): {:?}", iterations, duration);
    println!("Time per read (Signal): {:?}", duration / iterations as u32);

    // Benchmark Computed Read (Cold - should be fast as not stale)
    let computed = Computed::new(runtime, move || read.get());

    // Warm up
    for _ in 0..1000 {
        let _ = computed.get();
    }

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = computed.get();
    }
    let duration = start.elapsed();
    println!("Computed::get ({} iterations): {:?}", iterations, duration);
    println!(
        "Time per read (Computed): {:?}",
        duration / iterations as u32
    );
}
