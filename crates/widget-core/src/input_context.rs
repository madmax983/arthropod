use flux_state::{ReadSignal, WriteSignal};
use indexmap::IndexMap;
use render_engine::NodeId;
use std::collections::{HashMap, HashSet};

/// Text input state for a node
#[derive(Clone)]
pub struct TextInputState {
    pub read_signal: ReadSignal<String>,
    pub write_signal: WriteSignal<String>,
    pub cursor_position: usize,
    pub readonly: bool,
    pub max_length: Option<usize>,
}

/// Reactive text state for a node
#[derive(Clone)]
pub struct ReactiveTextState {
    pub read_signal: ReadSignal<String>,
}

/// Convert a character index to a byte index in a string.
fn char_idx_to_byte_idx(s: &str, char_idx: usize) -> Option<usize> {
    s.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .or_else(|| {
            // If char_idx equals char count, return the string length
            // (valid insertion point at the end)
            if char_idx == s.chars().count() {
                Some(s.len())
            } else {
                None
            }
        })
}

pub struct InputContext {
    pub text_input_states: IndexMap<NodeId, TextInputState>,
    pub reactive_text_states: HashMap<NodeId, ReactiveTextState>,
    pub focused_node: Option<NodeId>,
    pub placeholders: HashSet<NodeId>,
}

impl InputContext {
    pub fn new() -> Self {
        Self {
            text_input_states: IndexMap::new(),
            reactive_text_states: HashMap::new(),
            focused_node: None,
            placeholders: HashSet::new(),
        }
    }

    pub fn add_text_input_state(
        &mut self,
        node_id: NodeId,
        read_signal: ReadSignal<String>,
        write_signal: WriteSignal<String>,
        readonly: bool,
        max_length: Option<usize>,
    ) {
        let cursor_position = read_signal.get_untracked().chars().count();
        self.text_input_states.insert(
            node_id,
            TextInputState {
                read_signal,
                write_signal,
                cursor_position,
                readonly,
                max_length,
            },
        );
    }

    pub fn add_reactive_text_state(&mut self, node_id: NodeId, read_signal: ReadSignal<String>) {
        self.reactive_text_states
            .insert(node_id, ReactiveTextState { read_signal });
    }

    pub fn add_placeholder(&mut self, node_id: NodeId) {
        self.placeholders.insert(node_id);
    }

    pub fn has_placeholder(&self, node_id: NodeId) -> bool {
        self.placeholders.contains(&node_id)
    }

    pub fn is_text_input(&self, node_id: NodeId) -> bool {
        self.text_input_states.contains_key(&node_id)
    }

    pub fn focus_node(&mut self, node_id: NodeId) {
        self.focused_node = Some(node_id);
    }

    pub fn is_focused(&self, node_id: NodeId) -> bool {
        self.focused_node == Some(node_id)
    }

    pub fn blur_node(&mut self, node_id: NodeId) {
        if self.focused_node == Some(node_id) {
            self.focused_node = None;
        }
    }

    pub fn get_cursor_position(&self, node_id: NodeId) -> Option<usize> {
        self.text_input_states
            .get(&node_id)
            .map(|state| state.cursor_position)
    }

    pub fn get_text_input_value(&self, node_id: NodeId) -> Option<String> {
        self.text_input_states
            .get(&node_id)
            .map(|state| state.read_signal.get_untracked())
    }

    pub fn send_char(&mut self, c: char) {
        if let Some(focused_id) = self.focused_node {
            if let Some(state) = self.text_input_states.get_mut(&focused_id) {
                if state.readonly {
                    return;
                }

                let mut current_value = state.read_signal.get_untracked();

                if let Some(max_len) = state.max_length {
                    if current_value.chars().count() >= max_len {
                        return;
                    }
                }

                if let Some(byte_idx) = char_idx_to_byte_idx(&current_value, state.cursor_position)
                {
                    current_value.insert(byte_idx, c);
                    state.cursor_position += 1;
                    state.write_signal.set(current_value);
                }
            }
        }
    }

    pub fn send_backspace(&mut self) {
        if let Some(focused_id) = self.focused_node {
            if let Some(state) = self.text_input_states.get_mut(&focused_id) {
                if state.readonly {
                    return;
                }

                if state.cursor_position > 0 {
                    let mut current_value = state.read_signal.get_untracked();

                    if let Some(byte_idx) =
                        char_idx_to_byte_idx(&current_value, state.cursor_position - 1)
                    {
                        current_value.remove(byte_idx);
                        state.cursor_position -= 1;
                        state.write_signal.set(current_value);
                    }
                }
            }
        }
    }

    pub fn send_delete(&mut self) {
        if let Some(focused_id) = self.focused_node {
            if let Some(state) = self.text_input_states.get_mut(&focused_id) {
                if state.readonly {
                    return;
                }

                let current_value = state.read_signal.get_untracked();
                let char_count = current_value.chars().count();

                if state.cursor_position < char_count {
                    let mut new_value = current_value;

                    if let Some(byte_idx) = char_idx_to_byte_idx(&new_value, state.cursor_position)
                    {
                        new_value.remove(byte_idx);
                        state.write_signal.set(new_value);
                    }
                }
            }
        }
    }

    pub fn send_key_left(&mut self) {
        if let Some(focused_id) = self.focused_node {
            if let Some(state) = self.text_input_states.get_mut(&focused_id) {
                if state.cursor_position > 0 {
                    state.cursor_position -= 1;
                }
            }
        }
    }

    pub fn send_key_right(&mut self) {
        if let Some(focused_id) = self.focused_node {
            if let Some(state) = self.text_input_states.get_mut(&focused_id) {
                let current_value = state.read_signal.get_untracked();
                let char_count = current_value.chars().count();

                if state.cursor_position < char_count {
                    state.cursor_position += 1;
                }
            }
        }
    }

    /// Focus the next focusable node (Tab navigation).
    ///
    /// Cycles through all text inputs in the order they were added.
    /// If no node is focused, focuses the first one.
    /// Wraps around from last to first.
    ///
    /// # Returns
    ///
    /// The newly focused `NodeId`, or `None` if there are no focusable nodes.
    pub fn focus_next(&mut self) -> Option<NodeId> {
        let focusable: Vec<NodeId> = self.text_input_states.keys().copied().collect();
        if focusable.is_empty() {
            return None;
        }

        let current_index = self
            .focused_node
            .and_then(|f| focusable.iter().position(|&id| id == f));

        let next_index = match current_index {
            Some(idx) => (idx + 1) % focusable.len(),
            None => 0,
        };

        let next_node = focusable[next_index];
        self.focused_node = Some(next_node);
        Some(next_node)
    }

    /// Focus the previous focusable node (Shift+Tab navigation).
    ///
    /// Cycles through all text inputs in reverse order.
    /// If no node is focused, focuses the last one.
    /// Wraps around from first to last.
    ///
    /// # Returns
    ///
    /// The newly focused `NodeId`, or `None` if there are no focusable nodes.
    pub fn focus_prev(&mut self) -> Option<NodeId> {
        let focusable: Vec<NodeId> = self.text_input_states.keys().copied().collect();
        if focusable.is_empty() {
            return None;
        }

        let current_index = self
            .focused_node
            .and_then(|f| focusable.iter().position(|&id| id == f));

        let prev_index = match current_index {
            Some(0) => focusable.len() - 1,
            Some(idx) => idx - 1,
            None => focusable.len() - 1,
        };

        let prev_node = focusable[prev_index];
        self.focused_node = Some(prev_node);
        Some(prev_node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_idx_to_byte_idx_helper() {
        // ASCII
        assert_eq!(char_idx_to_byte_idx("abc", 0), Some(0));
        assert_eq!(char_idx_to_byte_idx("abc", 1), Some(1));
        assert_eq!(char_idx_to_byte_idx("abc", 2), Some(2));
        assert_eq!(char_idx_to_byte_idx("abc", 3), Some(3)); // End position

        // Emoji (4 bytes each)
        assert_eq!(char_idx_to_byte_idx("😀", 0), Some(0));
        assert_eq!(char_idx_to_byte_idx("😀", 1), Some(4)); // End of 4-byte emoji

        // Mixed
        assert_eq!(char_idx_to_byte_idx("a😀b", 0), Some(0)); // 'a'
        assert_eq!(char_idx_to_byte_idx("a😀b", 1), Some(1)); // emoji start
        assert_eq!(char_idx_to_byte_idx("a😀b", 2), Some(5)); // 'b'
        assert_eq!(char_idx_to_byte_idx("a😀b", 3), Some(6)); // end

        // Out of bounds
        assert_eq!(char_idx_to_byte_idx("abc", 10), None);
        assert_eq!(char_idx_to_byte_idx("😀", 5), None);
    }
}
