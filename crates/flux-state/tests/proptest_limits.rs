use flux_state::{Computed, Runtime, Signal};
use proptest::prelude::*;
use std::panic;

proptest! {
    // We will test if creating deep graphs in flux-state with `proptest` can break things
    #[test]
    fn test_deep_reactivity_graph(depth in 1..200) {
        let result = panic::catch_unwind(|| {
            let runtime = Runtime::new();
            let signal = Signal::new(runtime.clone(), 0);
            let (read, write) = signal.split();

            // Create a deep chain of computed values: C1 -> C2 -> C3 -> ...
            let mut last_computed = Computed::new(runtime.clone(), move || {
                read.get() + 1
            });

            for _ in 1..depth {
                let rc = last_computed.clone();
                last_computed = Computed::new(runtime.clone(), move || {
                    rc.get() + 1
                });
            }

            // Try to read the deepest node
            let initial_val = last_computed.get();
            assert_eq!(initial_val, depth);

            // Update the signal and see if it propagates all the way up without overflowing limits
            write.set(10);
            let updated_val = last_computed.get();
            assert_eq!(updated_val, depth + 10);
        });

        if depth > 100 {
            // Havoc expects the system to crash and burn when depth > 100!
            prop_assert!(result.is_err(), "Havoc predicted a panic, but it didn't happen!");
        } else {
            prop_assert!(result.is_ok(), "Failed for small depth!");
        }
    }
}
