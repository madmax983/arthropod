#[cfg(test)]
mod tests {
    use flux_state::{Runtime, Signal};
    use indexmap::IndexMap;
    use input_engine::{InputNodeId, text::*};

    fn create_state(initial: &str) -> TextInputState {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, initial.to_string());
        let (read, write) = signal.split();
        TextInputState {
            read_signal: read,
            write_signal: write,
            cursor_position: initial.chars().count(), // Default to end
            readonly: false,
            max_length: None,
        }
    }

    #[test]
    fn test_send_keys() {
        let mut states = IndexMap::new();
        let state1 = create_state("test");
        let id1 = InputNodeId(1);
        states.insert(id1, state1);

        let state2 = create_state("other");
        let id2 = InputNodeId(2);
        states.insert(id2, state2);

        send_key_left(&mut states, Some(id1));
        assert_eq!(states.get(&id1).unwrap().cursor_position, 3);

        send_key_right(&mut states, Some(id1));
        assert_eq!(states.get(&id1).unwrap().cursor_position, 4);

        send_char(&mut states, Some(id1), '!');
        assert_eq!(states.get(&id1).unwrap().read_signal.get(), "test!");

        send_backspace(&mut states, Some(id1));
        assert_eq!(states.get(&id1).unwrap().read_signal.get(), "test");

        send_key_left(&mut states, Some(id1));
        send_delete(&mut states, Some(id1));
        assert_eq!(states.get(&id1).unwrap().read_signal.get(), "tes");
    }
}
