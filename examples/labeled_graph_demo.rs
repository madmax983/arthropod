#[cfg(not(feature = "nova"))]
fn main() {
    println!("This example requires the 'nova' feature enabled. Run with --features nova");
}

#[cfg(feature = "nova")]
fn main() -> anyhow::Result<()> {
    use flux_devtools::run_inspector;
    use flux_state::{Computed, Runtime, Signal};
    use std::{thread, time::Duration};

    // 1. Initialize Runtime
    let runtime = Runtime::new();

    // 2. Create Reactive State with Labels
    let count = Signal::new(runtime.clone(), 0).with_label("Counter");

    let (read_count, write_count) = count.split();

    // Derived value
    let read_count_clone = read_count.clone();
    let _double_count = Computed::new(runtime.clone(), move || read_count_clone.get() * 2)
        .with_label("Double Counter");

    // Effect
    let read_count_clone2 = read_count.clone();
    let _effect = flux_state::Effect::new(runtime.clone(), move || {
        // Just reading to establish dependency
        let _ = read_count_clone2.get();
    })
    .with_label("Logger Effect");

    // 3. Spawn a background thread to simulate activity
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(500));
            write_count.update(|c| *c += 1);
        }
    });

    // 4. Run Inspector (Blocks main thread)
    println!("Starting Flux Inspector with Labeled Graph...");
    println!("Look for 'Counter', 'Double Counter', and 'Logger Effect' in the TUI.");
    run_inspector(runtime)?;

    Ok(())
}
