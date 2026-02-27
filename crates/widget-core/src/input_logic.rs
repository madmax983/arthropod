use crate::input_state::TextInputState;
use indexmap::IndexMap;
use render_engine::NodeId;

fn with_focused_mut<F>(states: &mut IndexMap<NodeId, TextInputState>, focused: Option<NodeId>, f: F)
where
    F: FnOnce(&mut TextInputState),
{
    if let Some(id) = focused {
        if let Some(state) = states.get_mut(&id) {
            f(state);
        }
    }
}

/// Send a character to focused input
pub fn send_char(
    text_input_states: &mut IndexMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
    c: char,
) {
    with_focused_mut(text_input_states, focused_node, |state| {
        state.insert_char(c)
    });
}

/// Send backspace to focused input
pub fn send_backspace(
    text_input_states: &mut IndexMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
) {
    with_focused_mut(text_input_states, focused_node, |state| state.backspace());
}

/// Send delete to focused input
pub fn send_delete(
    text_input_states: &mut IndexMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
) {
    with_focused_mut(text_input_states, focused_node, |state| state.delete());
}

/// Send left arrow key to focused input
pub fn send_key_left(
    text_input_states: &mut IndexMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
) {
    with_focused_mut(text_input_states, focused_node, |state| {
        state.move_cursor_left()
    });
}

/// Send right arrow key to focused input
pub fn send_key_right(
    text_input_states: &mut IndexMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
) {
    with_focused_mut(text_input_states, focused_node, |state| {
        state.move_cursor_right()
    });
}

/// Focus the next focusable node (Tab navigation).
pub fn focus_next(
    text_input_states: &IndexMap<NodeId, TextInputState>,
    focused_node: &mut Option<NodeId>,
) -> Option<NodeId> {
    input_engine::focus::focus_next(text_input_states, focused_node)
}

/// Focus the previous focusable node (Shift+Tab navigation).
pub fn focus_prev(
    text_input_states: &IndexMap<NodeId, TextInputState>,
    focused_node: &mut Option<NodeId>,
) -> Option<NodeId> {
    input_engine::focus::focus_prev(text_input_states, focused_node)
}
