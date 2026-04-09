//! Gesture recognition and sequencing.

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
///
/// Triggers the gesture only when the keys are pressed in the exact order specified.
/// If a wrong key is pressed, the sequence resets. However, if the wrong key matches
/// the *start* of the sequence, it immediately begins a new attempt (e.g., in "A B C",
/// pressing "A B A" will reset but keep the last "A" as the start of a new match).
///
/// # Example
///
/// ```
/// use input_engine::{SequenceMatcher, InputPattern};
/// use plat_core::Key;
///
/// // Detect the Konami Code: Up, Up, Down, Down...
/// let konami = vec![
///     Key::Up, Key::Up,
///     Key::Down, Key::Down,
///     Key::Left, Key::Right,
///     Key::Left, Key::Right,
///     Key::B, Key::A
/// ];
///
/// #[derive(Debug, Clone, PartialEq)]
/// enum GameAction { CheatCode }
///
/// let matcher = SequenceMatcher::new(konami, GameAction::CheatCode);
/// ```
pub struct SequenceMatcher<G> {
    sequence: Vec<Key>,
    current_index: usize,
    gesture: G,
}

impl<G: Clone + Send + Sync + PartialEq + std::fmt::Debug> SequenceMatcher<G> {
    /// Constructs a new `SequenceMatcher` to begin tracking a specific series of sequential key inputs.
    ///
    /// You should use this when you need to trigger a unique action (like a cheat code, Easter egg,
    /// or complex command palette invocation) only after the user has perfectly executed a strict sequence of keys.
    /// If the sequence is broken by an incorrect key, the internal state resets automatically.
    ///
    /// ## Examples
    ///
    /// ```
    /// use input_engine::SequenceMatcher;
    /// use plat_core::Key;
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// enum MyAction { FireHadouken }
    ///
    /// // The classic fighting game motion: Down, Down-Right (ignored here), Right, Punch.
    /// // Wait, let's keep it simple: Down, Right, Space.
    /// let sequence = vec![Key::Down, Key::Right, Key::Space];
    /// let _matcher = SequenceMatcher::new(sequence, MyAction::FireHadouken);
    /// ```
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
                        if self
                            .sequence
                            .first()
                            .is_some_and(|&start_key| input.key == start_key)
                        {
                            self.current_index = 1;
                        }
                    }
                }
            }
        }
        None
    }
}

/// A matcher that detects simultaneous key presses (chords).
///
/// Triggers the gesture when *all* required keys are currently pressed.
/// Additional keys being pressed does not prevent the match (non-exclusive).
///
/// # Example
///
/// ```
/// use input_engine::{ChordMatcher, InputPattern};
/// use plat_core::Key;
///
/// // Detect Ctrl + S (Save)
/// #[derive(Debug, Clone, PartialEq)]
/// enum Action { Save }
///
/// let matcher = ChordMatcher::new(vec![Key::Control, Key::S], Action::Save);
/// ```
pub struct ChordMatcher<G> {
    required_keys: Vec<Key>,
    pressed_keys: Vec<Key>,
    gesture: G,
}

impl<G: Clone + Send + Sync + PartialEq + std::fmt::Debug> ChordMatcher<G> {
    /// Constructs a new `ChordMatcher` to begin tracking a simultaneous combination of keys.
    ///
    /// You should use this when you need to detect standard keyboard shortcuts or modifiers combined
    /// with primary action keys (e.g., `Ctrl + S`, `Shift + Alt + Delete`). The matcher will fire
    /// the moment the final required key is depressed, regardless of whether other irrelevant keys
    /// are also currently held down.
    ///
    /// ## Examples
    ///
    /// ```
    /// use input_engine::ChordMatcher;
    /// use plat_core::Key;
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// enum MyShortcut { SaveDocument }
    ///
    /// // Requires both Control and S to be held down simultaneously.
    /// let keys = vec![Key::Control, Key::S];
    /// let _matcher = ChordMatcher::new(keys, MyShortcut::SaveDocument);
    /// ```
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
///
/// Bridges the gap between raw window events and high-level gesture signals.
/// This function creates an [`Effect`] that monitors the input signal and
/// updates the output signal whenever the provided `pattern` matches.
///
/// # Arguments
///
/// * `cx` - The reactive runtime.
/// * `input` - A signal yielding optional `WindowEvent`s (e.g. from an event loop).
/// * `pattern` - A stateful matcher implementing [`InputPattern`].
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
            let gesture_result = {
                let mut pattern = pattern.lock().expect("Mutex poisoned");
                pattern.update(&event)
            };
            write_out.set(gesture_result);
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
    fn test_sequence_matcher_reset_with_start_key() {
        let runtime = Runtime::new();
        let input = Signal::new(runtime.clone(), None);
        let (read_input, write_input) = input.split();

        // Sequence: A, B, C
        let sequence = vec![Key::A, Key::B, Key::C];
        let matcher = SequenceMatcher::new(sequence, MyGesture::Konami);
        let gesture_signal = create_gesture_signal(runtime.clone(), read_input, matcher);

        let press = |key: Key| {
            write_input.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
                key,
                state: ElementState::Pressed,
                modifiers: Default::default(),
                repeat: false,
            })));
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

        // Press A, B, then A
        press(Key::A); // index = 1
        press(Key::B); // index = 2
        press(Key::A); // Expects C, gets A. Resets to 0, but A matches start, so index = 1.

        assert_eq!(gesture_signal.get_untracked(), None);

        // Now press B, C to complete the new sequence
        press(Key::B); // index = 2

        assert_eq!(gesture_signal.get_untracked(), None);

        press_only(Key::C);

        assert_eq!(gesture_signal.get_untracked(), Some(MyGesture::Konami));
    }

    #[test]
    fn test_chord_matcher() {
        let runtime = Runtime::new();
        let input = Signal::new(runtime.clone(), None);
        let (read_input, write_input) = input.split();

        #[derive(Clone, Debug, PartialEq)]
        enum Chord {
            #[allow(dead_code)]
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
        assert_eq!(gesture_signal.get_untracked(), None);
    }

    #[test]
    #[should_panic(expected = "Mutex poisoned")]
    fn test_gesture_signal_mutex_poisoning() {
        let runtime = Runtime::new();
        let input = Signal::new(runtime.clone(), None);
        let (read_input, write_input) = input.split();

        #[derive(Clone, Debug, PartialEq)]
        enum Chord {
            #[allow(dead_code)]
            CtrlS,
        }

        struct PoisoningMatcher;

        impl InputPattern for PoisoningMatcher {
            type Gesture = Chord;

            fn update(&mut self, _event: &WindowEvent) -> Option<Self::Gesture> {
                panic!("Intentional panic to poison mutex");
            }
        }

        let matcher = PoisoningMatcher;
        let _gesture_signal = create_gesture_signal(runtime.clone(), read_input, matcher);

        let press = |key: Key| {
            write_input.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
                key,
                state: ElementState::Pressed,
                modifiers: Default::default(),
                repeat: false,
            })));
        };

        // First event panics and poisons the mutex. We run this in a separate thread
        // to avoid polluting the main test thread's panic hook, but we catch it
        // so we can trigger the actual expect() panic we want to test.
        let write_input_clone = write_input.clone();
        let handle = std::thread::spawn(move || {
            write_input_clone.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
                key: Key::A,
                state: ElementState::Pressed,
                modifiers: Default::default(),
                repeat: false,
            })));
        });

        // Ignore the panic from the thread
        let _ = handle.join();

        // Second event triggers the expect("Mutex poisoned") panic on the main thread
        press(Key::B);
    }
}
