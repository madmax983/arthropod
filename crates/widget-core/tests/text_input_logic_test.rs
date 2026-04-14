use flux_state::{Runtime, Signal};
use std::sync::Arc;
use widget_core::TextInputState;

// Helper to create a state wrapper for testing
fn create_state(initial_text: &str) -> (TextInputState, Arc<Runtime>) {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), initial_text.to_string());
    let (read_signal, write_signal) = signal.split();

    let state = TextInputState {
        read_signal,
        write_signal,
        cursor_position: initial_text.chars().count(), // Start at end by default for tests unless specified
        readonly: false,
        max_length: None,
    };
    (state, runtime)
}

#[test]
fn test_insert_at_end() {
    let (mut state, _rt) = create_state("Hello");
    state.insert_char('!');

    assert_eq!(state.read_signal.get(), "Hello!");
    assert_eq!(state.cursor_position, 6);
}

#[test]
fn test_insert_at_start() {
    let (mut state, _rt) = create_state("World");
    state.cursor_position = 0;
    state.insert_char('H');

    assert_eq!(state.read_signal.get(), "HWorld");
    assert_eq!(state.cursor_position, 1);
}

#[test]
fn test_insert_middle() {
    let (mut state, _rt) = create_state("AC");
    state.cursor_position = 1;
    state.insert_char('B');

    assert_eq!(state.read_signal.get(), "ABC");
    assert_eq!(state.cursor_position, 2);
}

#[test]
fn test_backspace_at_end() {
    let (mut state, _rt) = create_state("Hello");
    state.backspace();

    assert_eq!(state.read_signal.get(), "Hell");
    assert_eq!(state.cursor_position, 4);
}

#[test]
fn test_backspace_at_start_does_nothing() {
    let (mut state, _rt) = create_state("Hello");
    state.cursor_position = 0;
    state.backspace();

    assert_eq!(state.read_signal.get(), "Hello");
    assert_eq!(state.cursor_position, 0);
}

#[test]
fn test_delete_at_start() {
    let (mut state, _rt) = create_state("Hello");
    state.cursor_position = 0;
    state.delete();

    assert_eq!(state.read_signal.get(), "ello");
    assert_eq!(state.cursor_position, 0); // Cursor shouldn't move, text shifts left
}

#[test]
fn test_delete_at_end_does_nothing() {
    let (mut state, _rt) = create_state("Hello");
    // Cursor at 5 (end)
    state.delete();

    assert_eq!(state.read_signal.get(), "Hello");
    assert_eq!(state.cursor_position, 5);
}

#[test]
fn test_unicode_insertion_counts_chars_not_bytes() {
    let (mut state, _rt) = create_state("");

    // Insert Emoji (4 bytes)
    state.insert_char('😀');

    assert_eq!(state.read_signal.get(), "😀");
    assert_eq!(state.cursor_position, 1); // 1 char
    assert_eq!(state.read_signal.get().len(), 4); // 4 bytes
}

#[test]
fn test_unicode_backspace_removes_whole_char() {
    let (mut state, _rt) = create_state("Hi😀");
    // Cursor at 3 (end: 'H', 'i', '😀')

    state.backspace();

    assert_eq!(state.read_signal.get(), "Hi");
    assert_eq!(state.cursor_position, 2);
}

#[test]
fn test_unicode_cursor_movement() {
    let (mut state, _rt) = create_state("A😀B");
    state.cursor_position = 0;

    state.move_cursor_right();
    assert_eq!(state.cursor_position, 1); // After 'A'

    state.move_cursor_right();
    assert_eq!(state.cursor_position, 2); // After '😀'

    state.move_cursor_right();
    assert_eq!(state.cursor_position, 3); // After 'B' (End)

    state.move_cursor_right(); // Should clamp
    assert_eq!(state.cursor_position, 3);
}

#[test]
fn test_max_length_enforcement() {
    let (mut state, _rt) = create_state("123");
    state.max_length = Some(3);

    state.insert_char('4');

    assert_eq!(state.read_signal.get(), "123"); // Should not change
    assert_eq!(state.cursor_position, 3);
}

#[test]
fn test_readonly_enforcement() {
    let (mut state, _rt) = create_state("Readonly");
    state.readonly = true;

    state.insert_char('!');
    assert_eq!(state.read_signal.get(), "Readonly");

    state.backspace();
    assert_eq!(state.read_signal.get(), "Readonly");

    state.delete();
    assert_eq!(state.read_signal.get(), "Readonly");
}

#[test]
fn test_external_update_resiliency() {
    let (mut state, _rt) = create_state("LongText");
    // Cursor at 8

    // Simulate external update (e.g. valid signal write from elsewhere)
    // We can't access state.write_signal directly easily to set without consuming it if it wasn't public,
    // but here we can just use set().
    state.write_signal.set("Short".to_string());

    // State is now desynchronized: cursor=8, text="Short" (len 5)
    // Next operation should fix it before applying.

    state.insert_char('!');

    // Logic: ensure_cursor_valid() clamps 8 -> 5.
    // Then inserts '!' at 5 -> "Short!"
    // Cursor becomes 6.

    assert_eq!(state.read_signal.get(), "Short!");
    assert_eq!(state.cursor_position, 6);
}
