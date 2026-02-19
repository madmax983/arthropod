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
    if text_input_states.is_empty() {
        return None;
    }

    let current_index = (*focused_node).and_then(|f| text_input_states.get_index_of(&f));

    let next_index = match current_index {
        Some(idx) => (idx + 1) % text_input_states.len(),
        None => 0,
    };

    let (next_node, _) = text_input_states.get_index(next_index)?;
    let next_node = *next_node;
    *focused_node = Some(next_node);
    Some(next_node)
}

/// Focus the previous focusable node (Shift+Tab navigation).
pub fn focus_prev(
    text_input_states: &IndexMap<NodeId, TextInputState>,
    focused_node: &mut Option<NodeId>,
) -> Option<NodeId> {
    if text_input_states.is_empty() {
        return None;
    }

    let current_index = (*focused_node).and_then(|f| text_input_states.get_index_of(&f));

    let prev_index = match current_index {
        Some(0) => text_input_states.len() - 1,
        Some(idx) => idx - 1,
        None => text_input_states.len() - 1,
    };

    let (prev_node, _) = text_input_states.get_index(prev_index)?;
    let prev_node = *prev_node;
    *focused_node = Some(prev_node);
    Some(prev_node)
}
