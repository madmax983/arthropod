#[cfg(test)]
mod tests {
    use flux_state::{Runtime, Signal};
    use hashbrown::HashMap;
    use indexmap::IndexMap;
    use input_engine::{InputNodeId, form::*, text::*};

    #[allow(dead_code)]
    fn create_text_state(initial: &str) -> TextInputState {
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
    fn should_trigger_submit_with_missing_form() {
        let mut form_states = HashMap::new();
        let text_input_states = IndexMap::new();
        let mut validators = HashMap::new();
        let form_id = InputNodeId(10);

        // This will return early without panicking or triggering a submit
        trigger_submit(
            form_id,
            &mut form_states,
            &text_input_states,
            &mut validators,
        );
    }
}
