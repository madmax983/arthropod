use flux_state::{Effect, Runtime, Signal};
use std::panic;

#[test]
fn test_infinite_recursion_crash() {
    let runtime = Runtime::new();

    let sig_a = Signal::new(runtime.clone(), 0);
    let (read_a, write_a) = sig_a.split();

    let sig_b = Signal::new(runtime.clone(), 0);
    let (read_b, write_b) = sig_b.split();

    println!("👺 Detonating infinite recursion...");

    let result = panic::catch_unwind(move || {
        // Effect 1
        let write_b_clone = write_b.clone();
        let _effect1 = Effect::new(runtime.clone(), move || {
            let val = read_a.get();
            if val < 10_000_000 {
                write_b_clone.set(val + 1);
            }
        });

        // Effect 2 - This will trigger the loop immediately upon creation!
        let write_a_clone = write_a.clone();
        let _effect2 = Effect::new(runtime.clone(), move || {
            let val = read_b.get();
            if val < 10_000_000 {
                write_a_clone.set(val + 1);
            }
        });

        // Note: We don't even need to call write_a.set(1) explicitly,
        // as the effects trigger each other during initialization.
        // But if they didn't, we would do it here.
    });

    match result {
        Ok(_) => panic!("Should have panicked with recursion limit exceeded"),
        Err(e) => {
            if let Some(msg) = e.downcast_ref::<&str>() {
                assert!(msg.contains("Reactive recursion limit exceeded"), "Unexpected panic message: {}", msg);
                println!("✅ Recursion limit caught successfully: {}", msg);
            } else if let Some(msg) = e.downcast_ref::<String>() {
                assert!(msg.contains("Reactive recursion limit exceeded"), "Unexpected panic message: {}", msg);
                println!("✅ Recursion limit caught successfully: {}", msg);
            } else {
                panic!("Unknown panic type");
            }
        }
    }
}
