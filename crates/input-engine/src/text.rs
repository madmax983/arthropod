//! Text input state types and logic.
//!
//! This module provides a "headless" text editing state machine. It handles the logic
//! for a single-line text input field, including:
//! - UTF-8 aware cursor movement
//! - Insertion and deletion (Backspace/Delete)
//! - Constraints (read-only, max length)
//! - Reactive state management via `flux-state`
//!
//! # The Headless Editor
//!
//! Because `input-engine` is decoupled from rendering, this module acts as a pure logic
//! layer. It doesn't know about fonts, pixels, or screen coordinates. Instead, it operates
//! on abstract character indices and string buffers.
//!
//! This separation allows:
//! 1.  **Testing**: You can test complex text editing behavior without spinning up a window.
//! 2.  **Portability**: The same logic runs on Windows, macOS, Linux, and potentially the web.
//! 3.  **Flexibility**: You can build any visual representation (TUI, GUI, 3D) on top of this state.
//!
//! # Example
//!
//! ```
//! use input_engine::text::TextInputState;
//! use flux_state::{Runtime, Signal};
//!
//! // 1. Setup reactive state
//! let runtime = Runtime::new();
//! let signal = Signal::new(runtime, "Hello".to_string());
//! let (read, write) = signal.split();
//!
//! // 2. Create the headless editor state
//! let mut state = TextInputState {
//!     read_signal: read.clone(),
//!     write_signal: write,
//!     cursor_position: 5, // After 'o'
//!     readonly: false,
//!     max_length: None,
//! };
//!
//! // 3. Simulate user input
//! state.insert_char('!');
//! assert_eq!(read.get_untracked(), "Hello!");
//!
//! state.move_cursor_left();
//! state.backspace(); // Deletes 'o'
//! assert_eq!(read.get_untracked(), "Hell!");
//! ```

use flux_state::{Computed, ReadSignal, WriteSignal};

/// State for a text input widget.
///
/// This struct holds the reactive signals for the text value, as well as the local state
/// for the cursor position and constraints like `max_length`.
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
    #[allow(clippy::collapsible_if)]
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
    #[allow(clippy::collapsible_if)]
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
    #[allow(clippy::collapsible_if)]
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

use indexmap::IndexMap;
use render_engine::NodeId;

#[allow(clippy::collapsible_if)]
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
            cursor_position: initial.chars().count(), // Default to end
            readonly: false,
            max_length: None,
        }
    }

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

    #[test]
    fn test_insert_char() {
        let mut state = create_state("ac");
        state.cursor_position = 1; // "a|c"
        state.insert_char('b');
        assert_eq!(state.read_signal.get(), "abc");
        assert_eq!(state.cursor_position, 2); // "ab|c"
    }

    #[test]
    fn test_insert_unicode() {
        let mut state = create_state("a");
        state.cursor_position = 1;
        state.insert_char('😀');
        assert_eq!(state.read_signal.get(), "a😀");
        assert_eq!(state.cursor_position, 2);

        state.insert_char('b');
        assert_eq!(state.read_signal.get(), "a😀b");
        assert_eq!(state.cursor_position, 3);
    }

    #[test]
    fn test_backspace() {
        let mut state = create_state("abc"); // cursor at 3
        state.backspace();
        assert_eq!(state.read_signal.get(), "ab");
        assert_eq!(state.cursor_position, 2);

        state.cursor_position = 1; // "a|b"
        state.backspace();
        assert_eq!(state.read_signal.get(), "b");
        assert_eq!(state.cursor_position, 0);

        state.backspace(); // At 0, no-op
        assert_eq!(state.read_signal.get(), "b");
        assert_eq!(state.cursor_position, 0);
    }

    #[test]
    fn test_backspace_unicode() {
        let mut state = create_state("a😀b"); // len 3 chars
        state.cursor_position = 2; // "a😀|b"
        state.backspace(); // Should remove emoji
        assert_eq!(state.read_signal.get(), "ab");
        assert_eq!(state.cursor_position, 1);
    }

    #[test]
    fn test_delete() {
        let mut state = create_state("abc");
        state.cursor_position = 1; // "a|bc"
        state.delete();
        assert_eq!(state.read_signal.get(), "ac");
        assert_eq!(state.cursor_position, 1); // Cursor stays
    }

    #[test]
    fn test_delete_unicode() {
        let mut state = create_state("a😀b");
        state.cursor_position = 1; // "a|😀b"
        state.delete();
        assert_eq!(state.read_signal.get(), "ab");
        assert_eq!(state.cursor_position, 1);
    }

    #[test]
    fn test_max_length() {
        let mut state = create_state("abc");
        state.max_length = Some(3);
        state.insert_char('d');
        assert_eq!(state.read_signal.get(), "abc"); // No change
    }

    #[test]
    fn test_readonly() {
        let mut state = create_state("abc");
        state.readonly = true;
        state.insert_char('d');
        assert_eq!(state.read_signal.get(), "abc");
        state.backspace();
        assert_eq!(state.read_signal.get(), "abc");
        state.delete();
        assert_eq!(state.read_signal.get(), "abc");
    }

    #[test]
    fn test_cursor_clamping() {
        let mut state = create_state("hello");
        state.cursor_position = 5;

        // Simulate external update
        state.write_signal.set("hi".to_string());

        // Trigger clamp via ensure_cursor_valid (called by move/insert/etc)
        state.move_cursor_right(); // should clamp first, then try to move right (blocked)

        assert_eq!(state.cursor_position, 2); // clamped to len("hi")
    }

    #[test]
    fn test_move_cursor() {
        let mut state = create_state("abc");
        state.cursor_position = 0;
        state.move_cursor_left(); // No-op
        assert_eq!(state.cursor_position, 0);

        state.move_cursor_right();
        assert_eq!(state.cursor_position, 1);

        state.cursor_position = 3;
        state.move_cursor_right(); // No-op
        assert_eq!(state.cursor_position, 3);
    }

    // --- Sentry Additional Tests ---

    #[test]
    fn test_boundary_conditions() {
        // Empty string
        let mut state = create_state("");
        state.move_cursor_left();
        assert_eq!(state.cursor_position, 0);
        state.move_cursor_right();
        assert_eq!(state.cursor_position, 0);
        state.backspace();
        assert_eq!(state.read_signal.get(), "");
        state.delete();
        assert_eq!(state.read_signal.get(), "");

        // Insert at start
        state.insert_char('a');
        assert_eq!(state.read_signal.get(), "a");
        assert_eq!(state.cursor_position, 1);

        // Delete at end
        state.delete(); // No-op at end
        assert_eq!(state.read_signal.get(), "a");

        // Backspace to empty
        state.backspace();
        assert_eq!(state.read_signal.get(), "");
        assert_eq!(state.cursor_position, 0);
    }

    #[test]
    fn test_mixed_utf8_mutations() {
        // "a" (1 byte) + "😀" (4 bytes) + "中" (3 bytes)
        // Total chars: 3. Total bytes: 1 + 4 + 3 = 8.
        let mut state = create_state("a😀中");

        // Cursor at start
        state.cursor_position = 0;
        state.delete(); // delete 'a'
        assert_eq!(state.read_signal.get(), "😀中");
        assert_eq!(state.cursor_position, 0);

        state.move_cursor_right(); // skip '😀'
        assert_eq!(state.cursor_position, 1);

        state.insert_char('x'); // insert 'x' after emoji: "😀x中"
        assert_eq!(state.read_signal.get(), "😀x中");
        assert_eq!(state.cursor_position, 2);

        state.move_cursor_right(); // skip '中'
        assert_eq!(state.cursor_position, 3);

        state.backspace(); // delete '中'
        assert_eq!(state.read_signal.get(), "😀x");
        assert_eq!(state.cursor_position, 2);
    }

    #[test]
    fn test_external_signal_truncation() {
        let mut state = create_state("hello world");
        state.cursor_position = 11; // End

        // External truncation
        state.write_signal.set("hi".to_string());

        // Next operation should clamp
        state.insert_char('!');
        // Logic: ensure_cursor_valid() clamps 11 -> 2 ("hi").
        // Then insert '!' at 2 -> "hi!"
        // Cursor becomes 3.

        assert_eq!(state.read_signal.get(), "hi!");
        assert_eq!(state.cursor_position, 3);
    }

    #[test]
    fn test_readonly_mutations_extended() {
        let mut state = create_state("test");
        state.readonly = true;

        state.cursor_position = 2; // "te|st"

        state.insert_char('x');
        assert_eq!(state.read_signal.get(), "test");
        assert_eq!(state.cursor_position, 2);

        state.backspace();
        assert_eq!(state.read_signal.get(), "test");
        assert_eq!(state.cursor_position, 2);

        state.delete();
        assert_eq!(state.read_signal.get(), "test");
        assert_eq!(state.cursor_position, 2);

        // Navigation should still work? The original code doesn't check readonly for nav
        state.move_cursor_left();
        assert_eq!(state.cursor_position, 1);
    }

    #[test]
    fn test_max_length_boundary() {
        let mut state = create_state("abc");
        state.max_length = Some(4);

        // Current len 3. Max 4. Can insert 1.
        state.insert_char('d'); // "abcd"
        assert_eq!(state.read_signal.get(), "abcd");

        // Current len 4. Max 4. Cannot insert.
        state.insert_char('e'); // Ignored
        assert_eq!(state.read_signal.get(), "abcd");
    }
}
