//! Input event types.

use crate::{Point, Size, WindowId};

/// Top-level event type.
#[derive(Debug, Clone)]
pub enum Event {
    /// Window-specific event.
    Window {
        window_id: WindowId,
        event: WindowEvent,
    },
    /// Application lifecycle event.
    Lifecycle(LifecycleEvent),
}

/// Window-specific events.
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// Window close was requested (e.g., clicking the X button).
    CloseRequested,
    /// Window was resized.
    Resized(Size<u32>),
    /// Window gained or lost focus.
    Focused(bool),
    /// Scale factor changed.
    ScaleFactorChanged {
        scale_factor: f64,
        new_inner_size: Size<u32>,
    },
    /// A redraw was requested.
    RedrawRequested,
    /// Keyboard input.
    KeyboardInput(KeyboardInput),
    /// Mouse button input.
    MouseInput(MouseInput),
    /// Cursor moved within the window.
    CursorMoved { position: Point<f64> },
    /// Cursor entered or left the window.
    CursorEntered(bool),
    /// Mouse wheel/scroll event.
    MouseWheel { delta: ScrollDelta },
}

/// Application lifecycle events.
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// Application is about to resume (became active).
    Resumed,
    /// Application is about to suspend (became inactive).
    Suspended,
}

/// Keyboard input event.
#[derive(Debug, Clone)]
pub struct KeyboardInput {
    pub key: Key,
    pub state: ElementState,
    pub modifiers: Modifiers,
    pub repeat: bool,
}

/// Mouse button input event.
#[derive(Debug, Clone)]
pub struct MouseInput {
    pub button: MouseButton,
    pub state: ElementState,
    pub position: Point<f64>,
    pub modifiers: Modifiers,
}

/// Scroll delta from mouse wheel.
#[derive(Debug, Clone, Copy)]
pub enum ScrollDelta {
    /// Delta in lines.
    LineDelta(f32, f32),
    /// Delta in pixels.
    PixelDelta(f64, f64),
}

/// State of a button or key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementState {
    Pressed,
    Released,
}

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

/// Modifier key state.
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Windows key / Command key
}

/// Keyboard key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Numbers
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Navigation
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,

    // Editing
    Backspace,
    Delete,
    Insert,
    Enter,
    Tab,
    Escape,
    Space,

    // Modifiers (as keys)
    Shift,
    Control,
    Alt,
    Meta,

    // Punctuation
    Period,
    Comma,
    Minus,
    Equal,
    Semicolon,
    Quote,
    Slash,
    Backslash,
    BracketLeft,
    BracketRight,
    Backtick,

    // Other
    Unknown,
    PrintScreen,
}

impl Key {
    /// Convert a key to its character representation (US keyboard layout).
    ///
    /// Returns `Some(char)` for keys that produce printable characters,
    /// `None` for special keys (Enter, Tab, arrows, function keys, etc.).
    ///
    /// # Arguments
    ///
    /// * `shift` - Whether the Shift key is held (affects character output)
    ///
    /// # Example
    ///
    /// ```
    /// use plat_core::Key;
    ///
    /// assert_eq!(Key::A.to_char(false), Some('a'));
    /// assert_eq!(Key::A.to_char(true), Some('A'));
    /// assert_eq!(Key::Key1.to_char(true), Some('!'));
    /// assert_eq!(Key::Enter.to_char(false), None);
    /// ```
    pub fn to_char(&self, shift: bool) -> Option<char> {
        match self {
            // Letters
            Key::A => Some(if shift { 'A' } else { 'a' }),
            Key::B => Some(if shift { 'B' } else { 'b' }),
            Key::C => Some(if shift { 'C' } else { 'c' }),
            Key::D => Some(if shift { 'D' } else { 'd' }),
            Key::E => Some(if shift { 'E' } else { 'e' }),
            Key::F => Some(if shift { 'F' } else { 'f' }),
            Key::G => Some(if shift { 'G' } else { 'g' }),
            Key::H => Some(if shift { 'H' } else { 'h' }),
            Key::I => Some(if shift { 'I' } else { 'i' }),
            Key::J => Some(if shift { 'J' } else { 'j' }),
            Key::K => Some(if shift { 'K' } else { 'k' }),
            Key::L => Some(if shift { 'L' } else { 'l' }),
            Key::M => Some(if shift { 'M' } else { 'm' }),
            Key::N => Some(if shift { 'N' } else { 'n' }),
            Key::O => Some(if shift { 'O' } else { 'o' }),
            Key::P => Some(if shift { 'P' } else { 'p' }),
            Key::Q => Some(if shift { 'Q' } else { 'q' }),
            Key::R => Some(if shift { 'R' } else { 'r' }),
            Key::S => Some(if shift { 'S' } else { 's' }),
            Key::T => Some(if shift { 'T' } else { 't' }),
            Key::U => Some(if shift { 'U' } else { 'u' }),
            Key::V => Some(if shift { 'V' } else { 'v' }),
            Key::W => Some(if shift { 'W' } else { 'w' }),
            Key::X => Some(if shift { 'X' } else { 'x' }),
            Key::Y => Some(if shift { 'Y' } else { 'y' }),
            Key::Z => Some(if shift { 'Z' } else { 'z' }),
            // Numbers (with shift symbols)
            Key::Key0 => Some(if shift { ')' } else { '0' }),
            Key::Key1 => Some(if shift { '!' } else { '1' }),
            Key::Key2 => Some(if shift { '@' } else { '2' }),
            Key::Key3 => Some(if shift { '#' } else { '3' }),
            Key::Key4 => Some(if shift { '$' } else { '4' }),
            Key::Key5 => Some(if shift { '%' } else { '5' }),
            Key::Key6 => Some(if shift { '^' } else { '6' }),
            Key::Key7 => Some(if shift { '&' } else { '7' }),
            Key::Key8 => Some(if shift { '*' } else { '8' }),
            Key::Key9 => Some(if shift { '(' } else { '9' }),
            // Space
            Key::Space => Some(' '),
            // Punctuation
            Key::Period => Some(if shift { '>' } else { '.' }),
            Key::Comma => Some(if shift { '<' } else { ',' }),
            Key::Minus => Some(if shift { '_' } else { '-' }),
            Key::Equal => Some(if shift { '+' } else { '=' }),
            Key::Semicolon => Some(if shift { ':' } else { ';' }),
            Key::Quote => Some(if shift { '"' } else { '\'' }),
            Key::Slash => Some(if shift { '?' } else { '/' }),
            Key::Backslash => Some(if shift { '|' } else { '\\' }),
            Key::BracketLeft => Some(if shift { '{' } else { '[' }),
            Key::BracketRight => Some(if shift { '}' } else { ']' }),
            Key::Backtick => Some(if shift { '~' } else { '`' }),
            // Non-character keys
            _ => None,
        }
    }

    /// Convert Windows virtual key code to Key enum
    #[cfg(target_os = "windows")]
    pub(crate) fn from_vk(vk: u32) -> Self {
        match vk {
            // Letters (A-Z)
            0x41 => Key::A,
            0x42 => Key::B,
            0x43 => Key::C,
            0x44 => Key::D,
            0x45 => Key::E,
            0x46 => Key::F,
            0x47 => Key::G,
            0x48 => Key::H,
            0x49 => Key::I,
            0x4A => Key::J,
            0x4B => Key::K,
            0x4C => Key::L,
            0x4D => Key::M,
            0x4E => Key::N,
            0x4F => Key::O,
            0x50 => Key::P,
            0x51 => Key::Q,
            0x52 => Key::R,
            0x53 => Key::S,
            0x54 => Key::T,
            0x55 => Key::U,
            0x56 => Key::V,
            0x57 => Key::W,
            0x58 => Key::X,
            0x59 => Key::Y,
            0x5A => Key::Z,
            // Numbers (0-9)
            0x30 => Key::Key0,
            0x31 => Key::Key1,
            0x32 => Key::Key2,
            0x33 => Key::Key3,
            0x34 => Key::Key4,
            0x35 => Key::Key5,
            0x36 => Key::Key6,
            0x37 => Key::Key7,
            0x38 => Key::Key8,
            0x39 => Key::Key9,
            // Function keys (F1-F12)
            0x70 => Key::F1,
            0x71 => Key::F2,
            0x72 => Key::F3,
            0x73 => Key::F4,
            0x74 => Key::F5,
            0x75 => Key::F6,
            0x76 => Key::F7,
            0x77 => Key::F8,
            0x78 => Key::F9,
            0x79 => Key::F10,
            0x7A => Key::F11,
            0x7B => Key::F12,
            // Punctuation
            0xBE => Key::Period,       // .
            0xBC => Key::Comma,        // ,
            0xBD => Key::Minus,        // -
            0xBB => Key::Equal,        // =
            0xBA => Key::Semicolon,    // ;
            0xDE => Key::Quote,        // '
            0xBF => Key::Slash,        // /
            0xDC => Key::Backslash,    // \
            0xDB => Key::BracketLeft,  // [
            0xDD => Key::BracketRight, // ]
            0xC0 => Key::Backtick,     // `
            _ => Key::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_char_lowercase_letters() {
        assert_eq!(Key::A.to_char(false), Some('a'));
        assert_eq!(Key::M.to_char(false), Some('m'));
        assert_eq!(Key::Z.to_char(false), Some('z'));
    }

    #[test]
    fn test_to_char_uppercase_letters() {
        assert_eq!(Key::A.to_char(true), Some('A'));
        assert_eq!(Key::M.to_char(true), Some('M'));
        assert_eq!(Key::Z.to_char(true), Some('Z'));
    }

    #[test]
    fn test_to_char_numbers_no_shift() {
        assert_eq!(Key::Key0.to_char(false), Some('0'));
        assert_eq!(Key::Key1.to_char(false), Some('1'));
        assert_eq!(Key::Key5.to_char(false), Some('5'));
        assert_eq!(Key::Key9.to_char(false), Some('9'));
    }

    #[test]
    fn test_to_char_numbers_with_shift() {
        assert_eq!(Key::Key1.to_char(true), Some('!'));
        assert_eq!(Key::Key2.to_char(true), Some('@'));
        assert_eq!(Key::Key3.to_char(true), Some('#'));
        assert_eq!(Key::Key4.to_char(true), Some('$'));
        assert_eq!(Key::Key5.to_char(true), Some('%'));
        assert_eq!(Key::Key6.to_char(true), Some('^'));
        assert_eq!(Key::Key7.to_char(true), Some('&'));
        assert_eq!(Key::Key8.to_char(true), Some('*'));
        assert_eq!(Key::Key9.to_char(true), Some('('));
        assert_eq!(Key::Key0.to_char(true), Some(')'));
    }

    #[test]
    fn test_to_char_punctuation() {
        assert_eq!(Key::Period.to_char(false), Some('.'));
        assert_eq!(Key::Period.to_char(true), Some('>'));
        assert_eq!(Key::Comma.to_char(false), Some(','));
        assert_eq!(Key::Comma.to_char(true), Some('<'));
        assert_eq!(Key::Minus.to_char(false), Some('-'));
        assert_eq!(Key::Minus.to_char(true), Some('_'));
        assert_eq!(Key::Equal.to_char(false), Some('='));
        assert_eq!(Key::Equal.to_char(true), Some('+'));
    }

    #[test]
    fn test_to_char_brackets() {
        assert_eq!(Key::BracketLeft.to_char(false), Some('['));
        assert_eq!(Key::BracketLeft.to_char(true), Some('{'));
        assert_eq!(Key::BracketRight.to_char(false), Some(']'));
        assert_eq!(Key::BracketRight.to_char(true), Some('}'));
    }

    #[test]
    fn test_to_char_space() {
        assert_eq!(Key::Space.to_char(false), Some(' '));
        assert_eq!(Key::Space.to_char(true), Some(' '));
    }

    #[test]
    fn test_to_char_non_printable_returns_none() {
        assert_eq!(Key::Enter.to_char(false), None);
        assert_eq!(Key::Tab.to_char(false), None);
        assert_eq!(Key::Backspace.to_char(false), None);
        assert_eq!(Key::Escape.to_char(false), None);
        assert_eq!(Key::Shift.to_char(false), None);
        assert_eq!(Key::Control.to_char(false), None);
        assert_eq!(Key::Alt.to_char(false), None);
        assert_eq!(Key::F1.to_char(false), None);
        assert_eq!(Key::Left.to_char(false), None);
        assert_eq!(Key::Up.to_char(false), None);
    }

    #[test]
    fn test_to_char_all_letters_covered() {
        let letters = [
            Key::A,
            Key::B,
            Key::C,
            Key::D,
            Key::E,
            Key::F,
            Key::G,
            Key::H,
            Key::I,
            Key::J,
            Key::K,
            Key::L,
            Key::M,
            Key::N,
            Key::O,
            Key::P,
            Key::Q,
            Key::R,
            Key::S,
            Key::T,
            Key::U,
            Key::V,
            Key::W,
            Key::X,
            Key::Y,
            Key::Z,
        ];
        for (i, key) in letters.iter().enumerate() {
            let expected_lower = (b'a' + i as u8) as char;
            let expected_upper = (b'A' + i as u8) as char;
            assert_eq!(key.to_char(false), Some(expected_lower));
            assert_eq!(key.to_char(true), Some(expected_upper));
        }
    }
}
