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

    // Other
    Unknown,
}
