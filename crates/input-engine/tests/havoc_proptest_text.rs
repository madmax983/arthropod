use flux_state::{Runtime, Signal};
use input_engine::TextInputState;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_random_text_state(
        initial_string in ".*",
        operations in prop::collection::vec(
            prop_oneof![
                Just(0), // insert
                Just(1), // backspace
                Just(2), // delete
                Just(3), // move left
                Just(4), // move right
            ],
            0..100
        ),
        chars in prop::collection::vec(any::<char>(), 0..100),
        cursor in 0..1000usize
    ) {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, initial_string.clone());
        let (read, write) = signal.split();

        let initial_cursor = std::cmp::min(cursor, initial_string.chars().count());

        let mut state = TextInputState {
            read_signal: read.clone(),
            write_signal: write,
            cursor_position: initial_cursor,
            readonly: false,
            max_length: None,
        };

        let mut char_idx = 0;

        for op in operations {
            match op {
                0 => {
                    if char_idx < chars.len() {
                        state.insert_char(chars[char_idx]);
                        char_idx += 1;
                    }
                }
                1 => state.backspace(),
                2 => state.delete(),
                3 => state.move_cursor_left(),
                4 => state.move_cursor_right(),
                _ => unreachable!(),
            }

            // Invariants: cursor must be within bounds of current text
            let current_len = read.get_untracked().chars().count();
            assert!(state.cursor_position <= current_len);
        }
    }
}
