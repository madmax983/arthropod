//! Focus navigation logic.

use crate::text::TextInputState;
use indexmap::IndexMap;
use render_engine::NodeId;

/// Calculate the next focus index based on direction
fn cycle_focus_index(current_idx: Option<usize>, total: usize, forward: bool) -> usize {
    if total == 0 {
        return 0;
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};

    fn create_state(initial: &str) -> TextInputState {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, initial.to_string());
        let (read, write) = signal.split();
        TextInputState {
            read_signal: read,
            write_signal: write,
            cursor_position: initial.chars().count(),
            readonly: false,
            max_length: None,
        }
    }

    #[test]
    fn should_cycle_forward_through_indices() {
        assert_eq!(cycle_focus_index(Some(0), 3, true), 1);
        assert_eq!(cycle_focus_index(Some(1), 3, true), 2);
        assert_eq!(cycle_focus_index(Some(2), 3, true), 0);
        assert_eq!(cycle_focus_index(None, 3, true), 0);
    }

    #[test]
    fn should_cycle_backward_through_indices() {
        assert_eq!(cycle_focus_index(Some(0), 3, false), 2);
        assert_eq!(cycle_focus_index(Some(1), 3, false), 0);
        assert_eq!(cycle_focus_index(Some(2), 3, false), 1);
        assert_eq!(cycle_focus_index(None, 3, false), 2);
    }

    #[test]
    fn should_not_panic_on_zero_total_forward() {
        assert_eq!(cycle_focus_index(Some(0), 0, true), 0);
        assert_eq!(cycle_focus_index(None, 0, false), 0);
    }

    #[test]
    fn should_update_focus_correctly_on_empty() {
        let states = IndexMap::new();
        let mut focused = None;
        assert_eq!(update_focus(&states, &mut focused, true), None);
        assert_eq!(focused, None);
    }

    #[test]
    fn should_focus_next_and_prev() {
        let mut states = IndexMap::new();
        let node1 = NodeId(1);
        let node2 = NodeId(2);
        let node3 = NodeId(3);
        states.insert(node1, create_state("1"));
        states.insert(node2, create_state("2"));
        states.insert(node3, create_state("3"));

        let mut focused = None;

        // None -> 1st node (forward)
        assert_eq!(focus_next(&states, &mut focused), Some(node1));
        assert_eq!(focused, Some(node1));

        // 1st -> 2nd
        assert_eq!(focus_next(&states, &mut focused), Some(node2));
        assert_eq!(focused, Some(node2));

        // 2nd -> 3rd
        assert_eq!(focus_next(&states, &mut focused), Some(node3));
        assert_eq!(focused, Some(node3));

        // 3rd -> 1st
        assert_eq!(focus_next(&states, &mut focused), Some(node1));
        assert_eq!(focused, Some(node1));

        // 1st -> 3rd (backward)
        assert_eq!(focus_prev(&states, &mut focused), Some(node3));
        assert_eq!(focused, Some(node3));

        // 3rd -> 2nd
        assert_eq!(focus_prev(&states, &mut focused), Some(node2));
        assert_eq!(focused, Some(node2));
    }

    #[test]
    fn should_focus_prev_from_none() {
        let mut states = IndexMap::new();
        let node1 = NodeId(1);
        let node2 = NodeId(2);
        states.insert(node1, create_state("1"));
        states.insert(node2, create_state("2"));

        let mut focused = None;
        // None -> last node
        assert_eq!(focus_prev(&states, &mut focused), Some(node2));
        assert_eq!(focused, Some(node2));
    }
}
