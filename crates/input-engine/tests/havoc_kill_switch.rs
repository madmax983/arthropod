// Try fuzzing the text external update
use flux_state::{Runtime, Signal};
use input_engine::text::TextInputState;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_text_external_signal_truncation_fuzz(
        initial_string in ".*",
        truncation_len in 0..1000usize
    ) {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, initial_string.clone());
        let (read, write) = signal.split();

        let mut state = TextInputState {
            read_signal: read.clone(),
            write_signal: write.clone(),
            cursor_position: initial_string.chars().count(), // Cursor at end
            readonly: false,
            max_length: None,
        };

        // Externally modify signal
        let truncated: String = initial_string.chars().take(truncation_len).collect();
        write.set(truncated.clone());

        // Now delete one character. Ensure cursor clamp works.
        state.backspace();

        let val = read.get_untracked();
        assert!(val.chars().count() <= truncated.chars().count());
    }
}
