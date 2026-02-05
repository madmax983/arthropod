//! Input state types
//!
//! Contains state definitions for text input, reactive text, and reactive color.

use flux_state::{ReadSignal, WriteSignal};
use render_engine::Color;

/// Text input state for a node
#[derive(Clone)]
pub struct TextInputState {
    pub read_signal: ReadSignal<String>,
    pub write_signal: WriteSignal<String>,
    pub cursor_position: usize,
    pub readonly: bool,
    pub max_length: Option<usize>,
}

impl TextInputState {
    /// Ensure cursor is within bounds relative to current signal value.
    /// Returns (current_value, char_count)
    fn ensure_cursor_valid(&mut self) -> (String, usize) {
        let current_value = self.read_signal.get_untracked();
        let char_count = current_value.chars().count();

        if self.cursor_position > char_count {
            self.cursor_position = char_count;
        }

        (current_value, char_count)
    }

    /// Insert a character at the current cursor position.
    pub fn insert_char(&mut self, c: char) {
        if self.readonly {
            return;
        }

        let (mut current_value, char_count) = self.ensure_cursor_valid();

        if let Some(max_len) = self.max_length {
            if char_count >= max_len {
                return;
            }
        }

        if let Some(byte_idx) = char_idx_to_byte_idx(&current_value, self.cursor_position) {
            current_value.insert(byte_idx, c);
            self.cursor_position += 1;
            self.write_signal.set(current_value);
        }
    }

    /// Delete the character before the cursor (Backspace).
    pub fn backspace(&mut self) {
        if self.readonly {
            return;
        }

        let (mut current_value, _) = self.ensure_cursor_valid();

        if self.cursor_position > 0 {
            if let Some(byte_idx) = char_idx_to_byte_idx(&current_value, self.cursor_position - 1) {
                current_value.remove(byte_idx);
                self.cursor_position -= 1;
                self.write_signal.set(current_value);
            }
        }
    }

    /// Delete the character after the cursor (Delete).
    pub fn delete(&mut self) {
        if self.readonly {
            return;
        }

        let (mut current_value, char_count) = self.ensure_cursor_valid();

        if self.cursor_position < char_count {
            if let Some(byte_idx) = char_idx_to_byte_idx(&current_value, self.cursor_position) {
                current_value.remove(byte_idx);
                self.write_signal.set(current_value);
            }
        }
    }

    /// Move the cursor left.
    pub fn move_cursor_left(&mut self) {
        let (_, _) = self.ensure_cursor_valid();

        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move the cursor right.
    pub fn move_cursor_right(&mut self) {
        let (_, char_count) = self.ensure_cursor_valid();

        if self.cursor_position < char_count {
            self.cursor_position += 1;
        }
    }
}

/// Reactive text state for a node
#[derive(Clone)]
pub struct ReactiveTextState {
    pub read_signal: ReadSignal<String>,
}

/// Reactive color state for a node
#[derive(Clone)]
pub struct ReactiveColorState {
    pub read_signal: ReadSignal<Color>,
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
