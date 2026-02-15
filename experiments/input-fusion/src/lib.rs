//! Input Fusion: Gesture Recognition and Sequencing
//!
//! This crate provides tools to fuse raw input events into meaningful gestures.
//! It uses `flux-state` for reactive updates.

use flux_state::{Effect, ReadSignal, Runtime, Signal};
use plat_core::{ElementState, Key, WindowEvent};
use std::sync::{Arc, Mutex};

/// A trait for defining input patterns to match.
pub trait InputPattern: Send + Sync {
    /// The type of gesture produced by this pattern.
    type Gesture: Clone + Send + Sync + PartialEq + std::fmt::Debug;

    /// Update the matcher with a new event.
    /// Returns `Some(gesture)` if the pattern is matched, `None` otherwise.
    fn update(&mut self, event: &WindowEvent) -> Option<Self::Gesture>;
}

/// A matcher that detects a specific sequence of keys.
pub struct SequenceMatcher<G> {
    sequence: Vec<Key>,
    current_index: usize,
    gesture: G,
}

impl<G: Clone + Send + Sync + PartialEq + std::fmt::Debug> SequenceMatcher<G> {
    pub fn new(sequence: Vec<Key>, gesture: G) -> Self {
        Self {
            sequence,
            current_index: 0,
            gesture,
        }
    }
}

impl<G: Clone + Send + Sync + PartialEq + std::fmt::Debug> InputPattern for SequenceMatcher<G> {
    type Gesture = G;

    fn update(&mut self, event: &WindowEvent) -> Option<Self::Gesture> {
        if let WindowEvent::KeyboardInput(input) = event {
            // Only handle KeyPressed
            if input.state == ElementState::Pressed {
                // If input.repeat is true, do we care? Maybe.
                // Assuming we want fresh presses.

                let expected = self.sequence.get(self.current_index);
                if let Some(&expected_key) = expected {
                    if input.key == expected_key {
                        self.current_index += 1;
                        if self.current_index == self.sequence.len() {
                            // Matched!
                            self.current_index = 0;
                            return Some(self.gesture.clone());
                        }
                    } else {
                        // Mismatch, reset
                        self.current_index = 0;
                        // Retry start of sequence?
                        if let Some(&start_key) = self.sequence.first() {
                            if input.key == start_key {
                                self.current_index = 1;
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

/// A matcher that detects simultaneous key presses (chords).
pub struct ChordMatcher<G> {
    required_keys: Vec<Key>,
    pressed_keys: Vec<Key>,
    gesture: G,
}

impl<G: Clone + Send + Sync + PartialEq + std::fmt::Debug> ChordMatcher<G> {
    pub fn new(keys: Vec<Key>, gesture: G) -> Self {
        Self {
            required_keys: keys,
            pressed_keys: Vec::new(),
            gesture,
        }
    }
}

impl<G: Clone + Send + Sync + PartialEq + std::fmt::Debug> InputPattern for ChordMatcher<G> {
    type Gesture = G;

    fn update(&mut self, event: &WindowEvent) -> Option<Self::Gesture> {
        if let WindowEvent::KeyboardInput(input) = event {
            match input.state {
                ElementState::Pressed => {
                    if !self.pressed_keys.contains(&input.key) {
                        self.pressed_keys.push(input.key);
                    }
                    // Check if all required keys are pressed
                    let all_pressed = self
                        .required_keys
                        .iter()
                        .all(|k| self.pressed_keys.contains(k));
                    if all_pressed {
                        return Some(self.gesture.clone());
                    }
                }
                ElementState::Released => {
                    if let Some(pos) = self.pressed_keys.iter().position(|k| *k == input.key) {
                        self.pressed_keys.remove(pos);
                    }
                }
            }
        }
        None
    }
}

/// A signal that holds a detected gesture.
/// Keeps the detection effect alive.
pub struct GestureSignal<G> {
    signal: ReadSignal<Option<G>>,
    _effect: Effect,
}

impl<G> std::ops::Deref for GestureSignal<G> {
    type Target = ReadSignal<Option<G>>;
    fn deref(&self) -> &Self::Target {
        &self.signal
    }
}

/// Create a signal that emits gestures detected from an input stream.
pub fn create_gesture_signal<P, G>(
    cx: Arc<Runtime>,
    input: ReadSignal<Option<WindowEvent>>,
    pattern: P,
) -> GestureSignal<G>
where
    P: InputPattern<Gesture = G> + 'static,
    G: Clone + Send + Sync + PartialEq + std::fmt::Debug + 'static,
{
    let output = Signal::new(cx.clone(), None);
    let (read_out, write_out) = output.split();

    let pattern = Arc::new(Mutex::new(pattern));

    let effect = Effect::new(cx.clone(), move || {
        let event = input.get();
        if let Some(event) = event {
            let mut pattern = pattern.lock().unwrap();
            if let Some(gesture) = pattern.update(&event) {
                write_out.set(Some(gesture));
            } else {
                write_out.set(None);
            }
        }
    });

    GestureSignal {
        signal: read_out,
        _effect: effect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::Runtime;
    use plat_core::{ElementState, KeyboardInput};

    #[derive(Clone, Debug, PartialEq)]
    enum MyGesture {
        Konami,
    }

    #[test]
    fn test_konami_code() {
        let runtime = Runtime::new();
        let input = Signal::new(runtime.clone(), None);
        let (read_input, write_input) = input.split();

        let konami_sequence = vec![
            Key::Up,
            Key::Up,
            Key::Down,
            Key::Down,
            Key::Left,
            Key::Right,
            Key::Left,
            Key::Right,
            Key::B,
            Key::A,
        ];

        let matcher = SequenceMatcher::new(konami_sequence, MyGesture::Konami);

        let gesture_signal = create_gesture_signal(runtime.clone(), read_input, matcher);

        // Helper to simulate key press
        let press = |key: Key| {
            // First set: Pressed
            write_input.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
                key,
                state: ElementState::Pressed,
                modifiers: Default::default(),
                repeat: false,
            })));
            // Second set: Released
            write_input.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
                key,
                state: ElementState::Released,
                modifiers: Default::default(),
                repeat: false,
            })));
        };

        let press_only = |key: Key| {
            write_input.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
                key,
                state: ElementState::Pressed,
                modifiers: Default::default(),
                repeat: false,
            })));
        };

        // Input incorrect sequence
        press(Key::A);
        assert_eq!(gesture_signal.get_untracked(), None);

        // Input Konami Code
        press(Key::Up);
        press(Key::Up);
        press(Key::Down);
        press(Key::Down);
        press(Key::Left);
        press(Key::Right);
        press(Key::Left);
        press(Key::Right);
        press(Key::B);

        // Should still be None
        assert_eq!(gesture_signal.get_untracked(), None);

        // Final key
        press_only(Key::A);

        // Should be detected immediately on press
        assert_eq!(gesture_signal.get_untracked(), Some(MyGesture::Konami));
    }

    #[test]
    fn test_chord_matcher() {
        let runtime = Runtime::new();
        let input = Signal::new(runtime.clone(), None);
        let (read_input, write_input) = input.split();

        #[derive(Clone, Debug, PartialEq)]
        enum Chord {
            CtrlS,
        }

        let matcher = ChordMatcher::new(vec![Key::Control, Key::S], Chord::CtrlS);
        let gesture_signal = create_gesture_signal(runtime.clone(), read_input, matcher);

        let press = |key: Key| {
            write_input.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
                key,
                state: ElementState::Pressed,
                modifiers: Default::default(),
                repeat: false,
            })));
        };

        let release = |key: Key| {
            write_input.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
                key,
                state: ElementState::Released,
                modifiers: Default::default(),
                repeat: false,
            })));
        };

        // Press Ctrl
        press(Key::Control);
        assert_eq!(gesture_signal.get_untracked(), None);

        // Press S (while Ctrl is held)
        press(Key::S);
        assert_eq!(gesture_signal.get_untracked(), Some(Chord::CtrlS));

        // Release S
        release(Key::S);
        // It might still report detecting if update logic returns Some.
        // But our logic returns None on release.
        // Wait, update returns None on release because it falls through match arm?
        // Ah, SequenceMatcher handles release by returning None.
        // ChordMatcher:
        // match input.state { Released => remove, return None? }
        // Yes, my implementation returns None on release (implicitly at end of function).
        // So signal becomes None.
        assert_eq!(gesture_signal.get_untracked(), None);
    }
}
