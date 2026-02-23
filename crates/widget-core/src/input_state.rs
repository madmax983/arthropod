//! Input state types
//!
//! This module defines the state structures used by widgets to manage user input and reactive updates.
//! These states are typically stored in the ECS (Entity Component System) as components attached to widget entities.
//!
//! # Main Types
//!
//! - [`TextInputState`]: Manages the state of a text input field (cursor position, text value, etc.).
//! - [`ReactiveTextState`]: Manages text content that updates reactively from a signal.
//! - [`ComputedTextState`]: Manages text content derived from a computed value.
//! - [`ReactiveColorState`]: Manages color updates from a signal.

use flux_state::{Computed, ReadSignal, WriteSignal};
use render_engine::Color;

/// State for a text input widget.
///
/// This struct holds the reactive signals for the text value, as well as the local state
/// for the cursor position and constraints like `max_length`.
///
/// It is typically created by the `TextInput` widget and attached to the entity.
#[derive(Clone)]
pub struct TextInputState {
    /// Read handle for the text value.
    pub read_signal: ReadSignal<String>,
    /// Write handle for updating the text value.
    pub write_signal: WriteSignal<String>,
    /// Current cursor position (in characters, not bytes).
    pub cursor_position: usize,
    /// Whether the input is read-only.
    pub readonly: bool,
    /// Optional maximum length (in characters).
    pub max_length: Option<usize>,
}

impl TextInputState {
    /// Ensure cursor is within bounds relative to current signal value.
    ///
    /// This handles cases where the text signal might be updated externally
    /// (e.g., reset to empty string), ensuring the local cursor position
    /// remains valid.
    ///
    /// Returns a tuple of `(current_value, char_count)`.
    fn ensure_cursor_valid(&mut self) -> (String, usize) {
        let current_value = self.read_signal.get_untracked();
        let char_count = current_value.chars().count();

        if self.cursor_position > char_count {
            self.cursor_position = char_count;
        }

        (current_value, char_count)
    }

    /// Insert a character at the current cursor position.
    ///
    /// If `readonly` is true or `max_length` would be exceeded, the insertion is ignored.
    /// Updates the `write_signal` with the new value and advances the cursor.
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

    /// Delete the character before the cursor (Backspace behavior).
    ///
    /// If `readonly` is true or cursor is at the start, does nothing.
    /// Updates the `write_signal` and moves the cursor back.
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

    /// Delete the character after the cursor (Delete key behavior).
    ///
    /// If `readonly` is true or cursor is at the end, does nothing.
    /// Updates the `write_signal`.
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

    /// Move the cursor left by one character.
    ///
    /// Clamps to 0.
    pub fn move_cursor_left(&mut self) {
        let (_, _) = self.ensure_cursor_valid();

        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move the cursor right by one character.
    ///
    /// Clamps to the length of the text.
    pub fn move_cursor_right(&mut self) {
        let (_, char_count) = self.ensure_cursor_valid();

        if self.cursor_position < char_count {
            self.cursor_position += 1;
        }
    }
}

/// Reactive text state for a node.
///
/// Used by widgets that display text which can change over time based on a `Signal`.
#[derive(Clone)]
pub struct ReactiveTextState {
    /// The source signal for the text.
    pub read_signal: ReadSignal<String>,
}

/// Computed text state for a node.
///
/// Used by widgets that display text derived from other state via a `Computed` value.
#[derive(Clone)]
pub struct ComputedTextState {
    /// The computed value source.
    pub computed: Computed<String>,
}

/// Reactive color state for a node.
///
/// Used by widgets that need to update their visual color based on a signal
/// (e.g., hover states, theming, or data-driven coloring).
#[derive(Clone)]
pub struct ReactiveColorState {
    /// The source signal for the color.
    pub read_signal: ReadSignal<Color>,
}

/// Convert a character index to a byte index in a string.
///
/// Returns `None` if the character index is out of bounds.
/// Returns `Some(len)` if `char_idx` equals the character count (insertion point at end).
///
/// This handles multi-byte UTF-8 characters correctly.
fn char_idx_to_byte_idx(s: &str, char_idx: usize) -> Option<usize> {
    let mut count = 0;
    for (idx, _) in s.char_indices() {
        if count == char_idx {
            return Some(idx);
        }
        count += 1;
    }
    // If we reached here, char_idx >= count
    if count == char_idx {
        Some(s.len())
    } else {
        None
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
