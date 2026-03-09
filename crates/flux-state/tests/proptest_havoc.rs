use flux_state::{Computed, Runtime, Signal};
use proptest::prelude::*;

proptest! {
    // This will naturally panic because i32::MAX + 1 overflows in debug builds.
    // The previous implementation used saturating_add to avoid it and then manually panicked,
    // but Havoc says we want to break the math with proptest!
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn test_arbitrary_math_overflow(n in i32::MAX-10..=i32::MAX) {
        let runtime = Runtime::new();
        let s = Signal::new(runtime.clone(), n);
        let (r, _) = s.split();

        let c = Computed::new(runtime.clone(), move || {
            r.get() + 1 // Genuine integer overflow
        });

        let _ = c.get();
    }
}
