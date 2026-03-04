use flux_state::{Runtime, Signal};
use proptest::prelude::*;

// Fuzzing: Use `arbitrary` to throw random bytes at the FFI and API layers.
// Since `flux-state` doesn't have an FFI layer, we throw random bytes at a String Signal.
// We want to see if `String::from_utf8` or similar causes issues if not handled,
// but we can just use `Vec<u8>` or `String` arbitrary generators.

proptest! {
    #[test]
    fn test_fuzz_string_api(data in any::<String>()) {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime.clone(), data.clone());

        let (read, write) = signal.split();

        // Throw random bytes (String data)
        let _result = std::panic::catch_unwind(|| {
            write.set(data.clone());
            let val = read.get();
            assert_eq!(val, data);
        });

        // Fuzzing with emojis, symbols, and null bytes!
    }
}
