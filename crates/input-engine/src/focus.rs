//! Focus navigation logic.

use crate::text::TextInputState;
use indexmap::IndexMap;
use render_engine::NodeId;

/// Calculate the next focus index based on direction
fn cycle_focus_index(current_idx: Option<usize>, total: usize, forward: bool) -> usize {
    match current_idx {
        Some(idx) => {
            if forward {
                (idx + 1) % total
            } else if idx == 0 {
                total - 1
            } else {
                idx - 1
            }
        }
        None => {
            if forward {
                0
            } else {
                total - 1
            }
        }
    }
}

/// Update focus state helper
fn update_focus(
    text_input_states: &IndexMap<NodeId, TextInputState>,
    focused_node: &mut Option<NodeId>,
    forward: bool,
) -> Option<NodeId> {
    if text_input_states.is_empty() {
        return None;
    }

    let current_index = (*focused_node).and_then(|f| text_input_states.get_index_of(&f));
    let next_index = cycle_focus_index(current_index, text_input_states.len(), forward);

    let (next_node, _) = text_input_states.get_index(next_index)?;
    let next_node = *next_node;
    *focused_node = Some(next_node);
    Some(next_node)
}

/// Focus the next focusable node (Tab navigation).
pub fn focus_next(
    text_input_states: &IndexMap<NodeId, TextInputState>,
    focused_node: &mut Option<NodeId>,
) -> Option<NodeId> {
    update_focus(text_input_states, focused_node, true)
}

/// Focus the previous focusable node (Shift+Tab navigation).
pub fn focus_prev(
    text_input_states: &IndexMap<NodeId, TextInputState>,
    focused_node: &mut Option<NodeId>,
) -> Option<NodeId> {
    update_focus(text_input_states, focused_node, false)
}
