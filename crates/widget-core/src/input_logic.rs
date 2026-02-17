use crate::input_state::TextInputState;
use indexmap::IndexMap;
use render_engine::NodeId;

/// Send a character to focused input
pub fn send_char(
    text_input_states: &mut IndexMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
    c: char,
) {
    if let Some(focused_id) = focused_node {
        if let Some(state) = text_input_states.get_mut(&focused_id) {
            state.insert_char(c);
        }
    }
}

/// Send backspace to focused input
pub fn send_backspace(
    text_input_states: &mut IndexMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
) {
    if let Some(focused_id) = focused_node {
        if let Some(state) = text_input_states.get_mut(&focused_id) {
            state.backspace();
        }
    }
}

/// Send delete to focused input
pub fn send_delete(
    text_input_states: &mut IndexMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
) {
    if let Some(focused_id) = focused_node {
        if let Some(state) = text_input_states.get_mut(&focused_id) {
            state.delete();
        }
    }
}

/// Send left arrow key to focused input
pub fn send_key_left(
    text_input_states: &mut IndexMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
) {
    if let Some(focused_id) = focused_node {
        if let Some(state) = text_input_states.get_mut(&focused_id) {
            state.move_cursor_left();
        }
    }
}

/// Send right arrow key to focused input
pub fn send_key_right(
    text_input_states: &mut IndexMap<NodeId, TextInputState>,
    focused_node: Option<NodeId>,
) {
    if let Some(focused_id) = focused_node {
        if let Some(state) = text_input_states.get_mut(&focused_id) {
            state.move_cursor_right();
        }
    }
}

/// Focus the next focusable node (Tab navigation).
pub fn focus_next(
    text_input_states: &IndexMap<NodeId, TextInputState>,
    focused_node: &mut Option<NodeId>,
) -> Option<NodeId> {
    let focusable: Vec<NodeId> = text_input_states.keys().copied().collect();
    if focusable.is_empty() {
        return None;
    }

    let current_index = (*focused_node).and_then(|f| focusable.iter().position(|&id| id == f));

    let next_index = match current_index {
        Some(idx) => (idx + 1) % focusable.len(),
        None => 0,
    };

    let next_node = focusable[next_index];
    *focused_node = Some(next_node);
    Some(next_node)
}

/// Focus the previous focusable node (Shift+Tab navigation).
pub fn focus_prev(
    text_input_states: &IndexMap<NodeId, TextInputState>,
    focused_node: &mut Option<NodeId>,
) -> Option<NodeId> {
    let focusable: Vec<NodeId> = text_input_states.keys().copied().collect();
    if focusable.is_empty() {
        return None;
    }

    let current_index = (*focused_node).and_then(|f| focusable.iter().position(|&id| id == f));

    let prev_index = match current_index {
        Some(0) => focusable.len() - 1,
        Some(idx) => idx - 1,
        None => focusable.len() - 1,
    };

    let prev_node = focusable[prev_index];
    *focused_node = Some(prev_node);
    Some(prev_node)
}
