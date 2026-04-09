//! Shared web runtime helpers that are testable on native targets.
#![cfg_attr(not(all(target_arch = "wasm32", feature = "web")), allow(dead_code))]

use crate::{
    ElementState, Key, KeyboardInput, Modifiers, MouseButton, MouseInput, Point, ScrollDelta, Size,
    WindowEvent,
};

const WHEEL_DELTA_MODE_PIXEL: u32 = 0;
const WHEEL_DELTA_MODE_LINE: u32 = 1;
const WHEEL_DELTA_MODE_PAGE: u32 = 2;

const LINE_HEIGHT_PX: f64 = 16.0;
const PAGE_HEIGHT_PX: f64 = 800.0;

/// Coalesces redraw requests so we schedule at most one frame callback at a time.
#[derive(Debug, Default, Clone, Copy)]
pub struct RedrawScheduler {
    requested: bool,
}

impl RedrawScheduler {
    /// Request a redraw.
    ///
    /// Returns `true` when this call transitioned the scheduler from idle to requested.
    pub fn request(&mut self) -> bool {
        if self.requested {
            return false;
        }
        self.requested = true;
        true
    }

    /// Consume a pending redraw request.
    ///
    /// Returns `true` when a request was pending.
    pub fn consume(&mut self) -> bool {
        let was_requested = self.requested;
        self.requested = false;
        was_requested
    }
}

/// Convert browser wheel delta to pixels.
#[must_use]
pub fn normalize_wheel(delta: f64, delta_mode: u32) -> f64 {
    match delta_mode {
        WHEEL_DELTA_MODE_PIXEL => delta,
        WHEEL_DELTA_MODE_LINE => delta * LINE_HEIGHT_PX,
        WHEEL_DELTA_MODE_PAGE => delta * PAGE_HEIGHT_PX,
        _ => delta,
    }
}

/// Build a `MouseWheel` event with normalized pixel deltas.
#[must_use]
pub fn wheel_event_from_input(delta_x: f64, delta_y: f64, delta_mode: u32) -> WindowEvent {
    WindowEvent::MouseWheel {
        delta: ScrollDelta::PixelDelta(
            normalize_wheel(delta_x, delta_mode),
            normalize_wheel(delta_y, delta_mode),
        ),
    }
}

/// Bridges a WASM `web_sys::MouseEvent`'s `mousemove` position into Arthropod's internal
/// `WindowEvent::CursorMoved`. We do this manually rather than relying on `winit` because
/// Arthropod handles its own Web canvas mounting and DOM event subscriptions.
#[must_use]
pub fn map_pointer_move(x: f64, y: f64) -> WindowEvent {
    WindowEvent::CursorMoved {
        position: Point::new(x, y),
    }
}

/// Bridges a WASM `web_sys::MouseEvent`'s `mousedown` into Arthropod's internal
/// `WindowEvent::MouseInput`. Transforms the browser's raw integer button ID
/// (0=left, 1=middle, 2=right) into our semantic `MouseButton` enum.
#[must_use]
pub fn map_pointer_down(button: i16, x: f64, y: f64) -> WindowEvent {
    map_pointer_with_state(button, ElementState::Pressed, x, y, Modifiers::default())
}

/// Bridges a WASM `web_sys::MouseEvent`'s `mouseup` into Arthropod's internal
/// `WindowEvent::MouseInput`.
#[must_use]
pub fn map_pointer_up(button: i16, x: f64, y: f64) -> WindowEvent {
    map_pointer_with_state(button, ElementState::Released, x, y, Modifiers::default())
}

/// Same as `map_pointer_down`, but injects modifier key states (Shift, Ctrl, Alt, Meta)
/// which are read synchronously from the originating Javascript DOM event.
#[must_use]
pub fn map_pointer_down_with_modifiers(
    button: i16,
    x: f64,
    y: f64,
    modifiers: Modifiers,
) -> WindowEvent {
    map_pointer_with_state(button, ElementState::Pressed, x, y, modifiers)
}

/// Same as `map_pointer_up`, but injects modifier key states (Shift, Ctrl, Alt, Meta).
#[must_use]
pub fn map_pointer_up_with_modifiers(
    button: i16,
    x: f64,
    y: f64,
    modifiers: Modifiers,
) -> WindowEvent {
    map_pointer_with_state(button, ElementState::Released, x, y, modifiers)
}

/// Packs the four boolean modifier flags extracted from a Javascript DOM event
/// (`shiftKey`, `ctrlKey`, `altKey`, `metaKey`) into a single `Modifiers` struct.
#[must_use]
pub fn map_modifiers(shift: bool, ctrl: bool, alt: bool, meta: bool) -> Modifiers {
    Modifiers {
        shift,
        ctrl,
        alt,
        meta,
    }
}

/// Bridges a WASM `web_sys::KeyboardEvent` into Arthropod's `KeyboardInput`.
/// The `code` maps to physical key positions (e.g. "KeyW" regardless of layout),
/// while the `key` maps to the printed character (for fallback resolution).
#[must_use]
pub fn map_key_input(code: &str, key: &str, state: ElementState, repeat: bool) -> KeyboardInput {
    map_key_input_with_modifiers(code, key, state, repeat, Modifiers::default())
}

/// Same as `map_key_input`, but injects modifier key states.
#[must_use]
pub fn map_key_input_with_modifiers(
    code: &str,
    key: &str,
    state: ElementState,
    repeat: bool,
    modifiers: Modifiers,
) -> KeyboardInput {
    KeyboardInput {
        key: map_web_key(code, key),
        state,
        modifiers,
        repeat,
    }
}

/// Map CSS px and DPR to physical-size resize events.
#[must_use]
pub fn map_resize_events(css_width: f64, css_height: f64, dpr: f64) -> (WindowEvent, WindowEvent) {
    let width = ((css_width * dpr).round() as i64).max(1) as u32;
    let height = ((css_height * dpr).round() as i64).max(1) as u32;
    let size = Size::new(width, height);

    (
        WindowEvent::Resized(size),
        WindowEvent::ScaleFactorChanged {
            scale_factor: dpr,
            new_inner_size: size,
        },
    )
}

#[must_use]
fn map_pointer_with_state(
    button: i16,
    state: ElementState,
    x: f64,
    y: f64,
    modifiers: Modifiers,
) -> WindowEvent {
    WindowEvent::MouseInput(MouseInput {
        button: map_mouse_button(button),
        state,
        position: Point::new(x, y),
        modifiers,
    })
}

#[must_use]
fn map_mouse_button(button: i16) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Middle,
        2 => MouseButton::Right,
        other => MouseButton::Other(other as u16),
    }
}

#[must_use]
fn map_web_key(code: &str, key: &str) -> Key {
    match code {
        "KeyA" => Key::A,
        "KeyB" => Key::B,
        "KeyC" => Key::C,
        "KeyD" => Key::D,
        "KeyE" => Key::E,
        "KeyF" => Key::F,
        "KeyG" => Key::G,
        "KeyH" => Key::H,
        "KeyI" => Key::I,
        "KeyJ" => Key::J,
        "KeyK" => Key::K,
        "KeyL" => Key::L,
        "KeyM" => Key::M,
        "KeyN" => Key::N,
        "KeyO" => Key::O,
        "KeyP" => Key::P,
        "KeyQ" => Key::Q,
        "KeyR" => Key::R,
        "KeyS" => Key::S,
        "KeyT" => Key::T,
        "KeyU" => Key::U,
        "KeyV" => Key::V,
        "KeyW" => Key::W,
        "KeyX" => Key::X,
        "KeyY" => Key::Y,
        "KeyZ" => Key::Z,
        "Digit0" => Key::Key0,
        "Digit1" => Key::Key1,
        "Digit2" => Key::Key2,
        "Digit3" => Key::Key3,
        "Digit4" => Key::Key4,
        "Digit5" => Key::Key5,
        "Digit6" => Key::Key6,
        "Digit7" => Key::Key7,
        "Digit8" => Key::Key8,
        "Digit9" => Key::Key9,
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "ArrowLeft" => Key::Left,
        "ArrowRight" => Key::Right,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "Insert" => Key::Insert,
        "Enter" | "NumpadEnter" => Key::Enter,
        "Tab" => Key::Tab,
        "Escape" => Key::Escape,
        "Space" => Key::Space,
        "ShiftLeft" | "ShiftRight" => Key::Shift,
        "ControlLeft" | "ControlRight" => Key::Control,
        "AltLeft" | "AltRight" => Key::Alt,
        "MetaLeft" | "MetaRight" => Key::Meta,
        "Period" => Key::Period,
        "Comma" => Key::Comma,
        "Minus" => Key::Minus,
        "Equal" => Key::Equal,
        "Semicolon" => Key::Semicolon,
        "Quote" => Key::Quote,
        "Slash" => Key::Slash,
        "Backslash" => Key::Backslash,
        "BracketLeft" => Key::BracketLeft,
        "BracketRight" => Key::BracketRight,
        "Backquote" => Key::Backtick,
        "PrintScreen" => Key::PrintScreen,
        _ => map_key_fallback(key),
    }
}

#[must_use]
fn map_key_fallback(key: &str) -> Key {
    if key.len() == 1 {
        let byte = key.as_bytes()[0];
        return match byte {
            b'a' | b'A' => Key::A,
            b'b' | b'B' => Key::B,
            b'c' | b'C' => Key::C,
            b'd' | b'D' => Key::D,
            b'e' | b'E' => Key::E,
            b'f' | b'F' => Key::F,
            b'g' | b'G' => Key::G,
            b'h' | b'H' => Key::H,
            b'i' | b'I' => Key::I,
            b'j' | b'J' => Key::J,
            b'k' | b'K' => Key::K,
            b'l' | b'L' => Key::L,
            b'm' | b'M' => Key::M,
            b'n' | b'N' => Key::N,
            b'o' | b'O' => Key::O,
            b'p' | b'P' => Key::P,
            b'q' | b'Q' => Key::Q,
            b'r' | b'R' => Key::R,
            b's' | b'S' => Key::S,
            b't' | b'T' => Key::T,
            b'u' | b'U' => Key::U,
            b'v' | b'V' => Key::V,
            b'w' | b'W' => Key::W,
            b'x' | b'X' => Key::X,
            b'y' | b'Y' => Key::Y,
            b'z' | b'Z' => Key::Z,
            b'0' => Key::Key0,
            b'1' => Key::Key1,
            b'2' => Key::Key2,
            b'3' => Key::Key3,
            b'4' => Key::Key4,
            b'5' => Key::Key5,
            b'6' => Key::Key6,
            b'7' => Key::Key7,
            b'8' => Key::Key8,
            b'9' => Key::Key9,
            b' ' => Key::Space,
            b'.' => Key::Period,
            b',' => Key::Comma,
            b'-' => Key::Minus,
            b'=' => Key::Equal,
            b';' => Key::Semicolon,
            b'\'' => Key::Quote,
            b'/' => Key::Slash,
            b'\\' => Key::Backslash,
            b'[' => Key::BracketLeft,
            b']' => Key::BracketRight,
            b'`' => Key::Backtick,
            _ => Key::Unknown,
        };
    }

    match key {
        "Escape" => Key::Escape,
        "Enter" => Key::Enter,
        "Tab" => Key::Tab,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "Insert" => Key::Insert,
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "ArrowLeft" => Key::Left,
        "ArrowRight" => Key::Right,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "Shift" => Key::Shift,
        "Control" => Key::Control,
        "Alt" => Key::Alt,
        "Meta" => Key::Meta,
        _ => Key::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redraw_scheduler_requests_next_frame_once() {
        let mut sched = RedrawScheduler::default();
        assert!(sched.request());
        assert!(!sched.request());
        assert!(sched.consume());
        assert!(!sched.consume());
    }

    #[test]
    fn test_redraw_scheduler_consume_resets_pending_state() {
        let mut sched = RedrawScheduler::default();
        assert!(sched.request());
        assert!(sched.consume());
        assert!(!sched.consume());
        assert!(
            sched.request(),
            "should allow requesting again after consume"
        );
    }

    #[test]
    fn test_normalize_wheel_line_mode() {
        assert_eq!(normalize_wheel(3.0, WHEEL_DELTA_MODE_LINE), 48.0);
    }

    #[test]
    fn test_map_resize_events_uses_dpr() {
        let (resized, scale) = map_resize_events(200.0, 100.0, 1.5);

        match resized {
            WindowEvent::Resized(size) => assert_eq!(size, Size::new(300, 150)),
            other => panic!("expected resized event, got {other:?}"),
        }

        match scale {
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => {
                assert_eq!(scale_factor, 1.5);
                assert_eq!(new_inner_size, Size::new(300, 150));
            }
            other => panic!("expected scale event, got {other:?}"),
        }
    }
}
