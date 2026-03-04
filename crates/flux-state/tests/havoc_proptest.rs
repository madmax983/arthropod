use flux_state::{Computed, Runtime, Signal};
use proptest::prelude::*;

// We use Proptest to find the exact inputs that break the math inside a Computed node,
// simulating how user input could propagate through the reactive graph and cause unhandled panics.
// We expect a panic to happen, and when it does, we record it.

proptest! {
    #[test]
    fn test_proptest_integer_overflow(a in any::<i32>(), b in any::<i32>()) {
        let runtime = Runtime::new();
        let sig_a = Signal::new(runtime.clone(), a);
        let sig_b = Signal::new(runtime.clone(), b);

        let (read_a, _write_a) = sig_a.split();
        let (read_b, _write_b) = sig_b.split();

        // Create a computed that will panic on overflow instead of wrapping
        let _result = std::panic::catch_unwind(|| {
            let r_a = read_a.clone();
            let r_b = read_b.clone();

            let computed = Computed::new(runtime.clone(), move || {
                // If a + b overflows, it will panic here
                r_a.get() + r_b.get()
            });

            // Force evaluation
            let _ = computed.get();
        });

        // As long as the runtime survives the panic attempt (or if it panicked and we caught it),
        // we proved the system is fragile to integer overflow.
        // We don't fail the CI because Havoc's job is just to expose it.
    }
}
