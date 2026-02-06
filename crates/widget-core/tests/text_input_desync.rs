use flux_state::{Runtime, Signal};
use widget_core::{TextInput, Widget, WidgetContext};

#[test]
fn test_text_input_cursor_desync_on_external_update() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    // 1. Initialize with long text
    let signal = Signal::new(runtime.clone(), "Hello World".to_string());
    let input = TextInput::new(signal.clone());
    let node_id = input.build(&mut ctx);

    // 2. Focus and move cursor to end (index 11)
    ctx.focus_node(node_id);
    // Cursor starts at end by default in current implementation (based on reading code, or we can force it)
    // Wait, let's check code:
    // ctx.add_text_input_state:
    // let cursor_position = read_signal.get_untracked().chars().count();
    // So yes, it starts at end.

    let initial_pos = ctx.get_cursor_position(node_id).unwrap();
    assert_eq!(initial_pos, 11);

    // 3. Update signal externally to shorter text
    // We need to use the write signal.
    // Since we don't have the write signal here (it's inside the widget),
    // we can use the signal we created initially.
    let (_, write) = signal.split();
    write.set("Hi".to_string());

    // 4. Verify text is updated in context (via read signal)
    assert_eq!(ctx.get_text_input_value(node_id).unwrap(), "Hi");

    // 5. Check cursor position - it is likely still 11 (The Bug)
    // Note: If the code was reactive, it might have updated, but TextInputState isn't.
    let _pos_after_update = ctx.get_cursor_position(node_id).unwrap();
    // We assert it is 11 to confirm the "buggy" state exists,
    // OR we can proceed to type and assert failure.
    // If we want to demonstrate the bug causes functional failure:

    // 6. Attempt to insert '!'
    ctx.send_char('!');

    // 7. Assert result
    // Expected behavior (Fix): Value becomes "Hi!", cursor becomes 3.
    // Bug behavior: Value remains "Hi", cursor remains 11 (or 12).

    let final_value = ctx.get_text_input_value(node_id).unwrap();

    // This assertion will fail if the bug exists
    assert_eq!(
        final_value, "Hi!",
        "Text input should work even after external signal update shortened the text"
    );

    let final_pos = ctx.get_cursor_position(node_id).unwrap();
    assert_eq!(final_pos, 3, "Cursor should be at the end of new text");
}
