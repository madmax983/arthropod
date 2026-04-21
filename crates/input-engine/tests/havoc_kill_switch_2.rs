use flux_state::{Runtime, Signal};
use input_engine::TextInputState;

#[test]
fn test_repro_backspace_panic() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime, " ".to_string());
    let (read, write) = signal.split();

    let mut state = TextInputState {
        read_signal: read.clone(),
        write_signal: write.clone(),
        cursor_position: 1,
        readonly: false,
        max_length: None,
    };

    write.set("".to_string());

    state.backspace();
}
