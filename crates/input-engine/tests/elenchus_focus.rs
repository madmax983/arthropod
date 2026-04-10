#[cfg(test)]
mod tests {
    use flux_state::{Runtime, Signal};
    use indexmap::IndexMap;
    use input_engine::text::TextInputState;
    use input_engine::{InputNodeId, focus::*};

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
    fn should_focus_prev_with_missing_node() {
        let mut states = IndexMap::new();
        let node1 = InputNodeId(1);
        let node2 = InputNodeId(2);
        states.insert(node1, create_state("1"));
        states.insert(node2, create_state("2"));

        let mut focused = Some(InputNodeId(99)); // Invalid ID

        // This will fall back to `None` effectively in `update_focus` because `states.get_index_of` returns `None`.
        // If current is `None` and forward is false, it should jump to the last element.
        assert_eq!(focus_prev(&states, &mut focused), Some(node2));
        assert_eq!(focused, Some(node2));
    }
}
