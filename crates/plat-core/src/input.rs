//! Input event types.

use crate::{Point, Size, WindowId};

/// The root event type dispatched by the application's main event loop.
///
/// This enum wraps all possible interactions, separating window-specific inputs
/// (like mouse clicks and resizing) from global application lifecycle events
/// (like suspension or resumption).
///
/// ## Examples
///
/// ```
/// use plat_core::{Event, LifecycleEvent};
///
/// let event = Event::Lifecycle(LifecycleEvent::Resumed);
/// match event {
///     Event::Lifecycle(LifecycleEvent::Resumed) => {
///         println!("App is waking up!");
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone)]
pub enum Event {
    /// An event that targets a specific graphical window.
    Window {
        /// A unique, opaque token identifying which window received the input.
        window_id: WindowId,
        /// The specific action or state change that occurred on the window.
        event: WindowEvent,
    },
    /// An event affecting the global state of the application process.
    Lifecycle(LifecycleEvent),
}

/// Window-specific events that are dispatched to the application.
///
/// These events represent the lifecycle, state changes, and hardware interactions
/// directed at a specific window. Handling these events correctly is critical for
/// a responsive and well-behaved application.
///
/// ## Examples
///
/// ```
/// use plat_core::{WindowEvent, Size};
///
/// let event = WindowEvent::Resized(Size::new(800, 600));
/// match event {
///     WindowEvent::Resized(size) => {
///         println!("Window resized to {}x{}", size.width, size.height);
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// Dispatched when the user clicks the close button (X) on the window frame,
    /// or uses a platform-specific shortcut (like Alt+F4) to close it.
    CloseRequested,
    /// Dispatched whenever the window's client area changes size.
    /// You should use this to trigger a layout pass and update rendering surface sizes.
    Resized(Size<u32>),
    /// Dispatched when the window gains or loses keyboard focus.
    /// Use this to pause animations or mute audio when the application is backgrounded.
    Focused(bool),
    /// Dispatched when the window moves to a monitor with a different DPI scale factor,
    /// or if the user changes the system-wide display scaling settings.
    ScaleFactorChanged {
        /// The new scale factor, typically a value like 1.0 (100%), 1.5 (150%), or 2.0 (200%).
        scale_factor: f64,
        /// The new physical size of the window after the scaling has been applied.
        new_inner_size: Size<u32>,
    },
    /// Dispatched when the OS or the window manager requests the application to redraw its contents.
    /// This happens when the window is exposed, resized, or explicitly invalidated.
    RedrawRequested,
    /// Represents a discrete keyboard interaction, such as a key being pressed or released.
    KeyboardInput(KeyboardInput),
    /// Represents a physical mouse button click or release within the window's client area.
    MouseInput(MouseInput),
    /// Dispatched whenever the pointing device (mouse, trackpad) moves within the client area.
    CursorMoved {
        /// The new precise sub-pixel position of the cursor relative to the top-left of the client area.
        position: Point<f64>,
    },
    /// Dispatched when the cursor physically enters or leaves the boundary of the window.
    /// Useful for resetting hover states or triggering edge-pan behaviors.
    CursorEntered(bool),
    /// Dispatched when a scroll wheel on a mouse is rotated, or when a trackpad gesture is interpreted as scrolling.
    MouseWheel {
        /// The direction and magnitude of the scroll movement.
        delta: ScrollDelta,
    },
}

/// Application lifecycle events.
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// Application is about to resume (became active).
    Resumed,
    /// Application is about to suspend (became inactive).
    Suspended,
}

/// Represents a single keyboard interaction event.
///
/// This struct captures the exact state of the keyboard at the moment the event occurred,
/// including the specific key, whether it went down or up, and any modifier keys that were held.
///
/// ## Examples
///
/// ```
/// use plat_core::{KeyboardInput, Key, ElementState, Modifiers};
///
/// let input = KeyboardInput {
///     key: Key::A,
///     state: ElementState::Pressed,
///     modifiers: Modifiers { shift: true, ..Default::default() },
///     repeat: false,
/// };
///
/// if input.state == ElementState::Pressed && input.modifiers.shift {
///     println!("User typed a capital A!");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct KeyboardInput {
    /// The specific logical key that triggered the event.
    pub key: Key,
    /// Indicates whether the physical key was pushed down or released.
    pub state: ElementState,
    /// A snapshot of the Shift, Ctrl, Alt, and Meta keys at the time of the event.
    pub modifiers: Modifiers,
    /// If `true`, this event was automatically generated by the operating system's
    /// key-repeat behavior because the user is holding the key down.
    pub repeat: bool,
}

/// Represents a discrete physical interaction with a mouse button.
///
/// This struct is used to track clicks, drags, and releases. It includes the
/// exact position of the cursor to allow for hit-testing against UI elements.
///
/// ## Examples
///
/// ```
/// use plat_core::{MouseInput, MouseButton, ElementState, Point, Modifiers};
///
/// let input = MouseInput {
///     button: MouseButton::Left,
///     state: ElementState::Pressed,
///     position: Point::new(100.0, 50.0),
///     modifiers: Modifiers::default(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct MouseInput {
    /// The physical button on the mouse that was pressed or released.
    pub button: MouseButton,
    /// Indicates whether the button transitioned to a pressed or released state.
    pub state: ElementState,
    /// The coordinates of the cursor when the button state changed.
    pub position: Point<f64>,
    /// Any keyboard modifiers that were held down during the click (e.g., for Shift-clicking).
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

/// Describes the physical state of a binary input element, such as a key or a mouse button.
///
/// ## Examples
///
/// ```
/// use plat_core::ElementState;
///
/// let state = ElementState::Pressed;
/// assert_eq!(state == ElementState::Pressed, true);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementState {
    /// The element is actively being held down by the user.
    Pressed,
    /// The element is not currently being interacted with.
    Released,
}

/// Identifies a specific button on a pointing device.
///
/// ## Examples
///
/// ```
/// use plat_core::MouseButton;
///
/// let primary_action = MouseButton::Left;
/// let context_menu_action = MouseButton::Right;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// The primary button, typically the left button on a right-handed mouse.
    Left,
    /// The secondary button, typically used to summon context menus.
    Right,
    /// The tertiary button, often located under the scroll wheel.
    Middle,
    /// Represents extra buttons found on specialized mice (e.g., thumb buttons for back/forward).
    Other(u16),
}

/// Represents the current active state of keyboard modifier keys.
///
/// This is heavily used to interpret user intent during text entry or when
/// constructing keyboard shortcuts (e.g., `Ctrl+C`).
///
/// ## Examples
///
/// ```
/// use plat_core::Modifiers;
///
/// let save_shortcut = Modifiers {
///     ctrl: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    /// True if either the left or right Shift key is held down.
    pub shift: bool,
    /// True if either the left or right Control key is held down.
    pub ctrl: bool,
    /// True if either the left or right Alt (or Option) key is held down.
    pub alt: bool,
    /// True if the Windows key (on Windows) or the Command key (on macOS) is held down.
    pub meta: bool, // Windows key / Command key
}

/// Keyboard key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)] // We do not need docs for every single key
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
