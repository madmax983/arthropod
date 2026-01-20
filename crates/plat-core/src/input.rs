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
            0xBE => Key::Period,     // .
            0xBC => Key::Comma,      // ,
            0xBD => Key::Minus,      // -
            0xBB => Key::Equal,      // =
            0xBA => Key::Semicolon,  // ;
            0xDE => Key::Quote,      // '
            0xBF => Key::Slash,      // /
            0xDC => Key::Backslash,  // \
            0xDB => Key::BracketLeft,  // [
            0xDD => Key::BracketRight, // ]
            0xC0 => Key::Backtick,   // `
            _ => Key::Unknown,
        }
    }
}
