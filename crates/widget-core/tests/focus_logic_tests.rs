use flux_state::{Runtime, Signal};
use widget_core::{TextInput, Widget, WidgetContext};

#[test]
fn test_focus_next_cycling() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    // Create 3 text inputs
    let val1 = Signal::new(runtime.clone(), String::new());
    let val2 = Signal::new(runtime.clone(), String::new());
    let val3 = Signal::new(runtime.clone(), String::new());

    let input1 = TextInput::new(val1);
    let input2 = TextInput::new(val2);
    let input3 = TextInput::new(val3);

    let id1 = input1.build(&mut ctx);
    let id2 = input2.build(&mut ctx);
    let id3 = input3.build(&mut ctx);

    // Initial state: no focus
    assert_eq!(ctx.focused_node(), None);

    // First tab: focus first input
    let next = ctx.focus_next();
    assert_eq!(next, Some(id1));
    assert_eq!(ctx.focused_node(), Some(id1));

    // Second tab: focus second input
    let next = ctx.focus_next();
    assert_eq!(next, Some(id2));
    assert_eq!(ctx.focused_node(), Some(id2));

    // Third tab: focus third input
    let next = ctx.focus_next();
    assert_eq!(next, Some(id3));
    assert_eq!(ctx.focused_node(), Some(id3));

    // Fourth tab: wrap around to first input
    let next = ctx.focus_next();
    assert_eq!(next, Some(id1));
    assert_eq!(ctx.focused_node(), Some(id1));
}

#[test]
fn test_focus_prev_cycling() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    // Create 3 text inputs
    let val1 = Signal::new(runtime.clone(), String::new());
    let val2 = Signal::new(runtime.clone(), String::new());
    let val3 = Signal::new(runtime.clone(), String::new());

    let input1 = TextInput::new(val1);
    let input2 = TextInput::new(val2);
    let input3 = TextInput::new(val3);

    let id1 = input1.build(&mut ctx);
    let id2 = input2.build(&mut ctx);
    let id3 = input3.build(&mut ctx);

    // Initial state: no focus
    assert_eq!(ctx.focused_node(), None);

    // Shift+Tab: focus last input (wrap around backwards)
    let prev = ctx.focus_prev();
    assert_eq!(prev, Some(id3));
    assert_eq!(ctx.focused_node(), Some(id3));

    // Shift+Tab again: focus second input
    let prev = ctx.focus_prev();
    assert_eq!(prev, Some(id2));
    assert_eq!(ctx.focused_node(), Some(id2));

    // Shift+Tab again: focus first input
    let prev = ctx.focus_prev();
    assert_eq!(prev, Some(id1));
    assert_eq!(ctx.focused_node(), Some(id1));

    // Shift+Tab again: wrap around to last input
    let prev = ctx.focus_prev();
    assert_eq!(prev, Some(id3));
    assert_eq!(ctx.focused_node(), Some(id3));
}

#[test]
fn test_focus_cycling_empty() {
    let mut ctx = WidgetContext::new_test();

    // No inputs
    assert_eq!(ctx.focus_next(), None);
    assert_eq!(ctx.focus_prev(), None);
}
