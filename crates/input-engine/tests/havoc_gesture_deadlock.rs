use flux_state::{Effect, Runtime, Signal};
use input_engine::{SequenceMatcher, create_gesture_signal};
use plat_core::{ElementState, Key, KeyboardInput, WindowEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
enum MyGesture {
    Hit,
}

#[test]
fn test_havoc_gesture_deadlock() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let runtime = Runtime::new();
        let input = Signal::new(runtime.clone(), None);
        let (read_input, write_input) = input.split();

        let matcher = SequenceMatcher::new(vec![Key::A], MyGesture::Hit);

        let gesture_sig = create_gesture_signal(runtime.clone(), read_input, matcher);

        let write_input_clone = write_input.clone();

        let _effect = Effect::new(runtime.clone(), move || {
            if let Some(MyGesture::Hit) = gesture_sig.get() {
                // Trigger another event while handling the gesture
                write_input_clone.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
                    key: Key::B,
                    state: ElementState::Pressed,
                    repeat: false,
                    modifiers: Default::default(),
                })));
            }
        });

        // Trigger the gesture
        write_input.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
            key: Key::A,
            state: ElementState::Pressed,
            repeat: false,
            modifiers: Default::default(),
        })));

        tx.send(()).unwrap();
    });

    if rx.recv_timeout(Duration::from_millis(500)).is_err() {
        panic!(
            "Deadlock detected: effect triggered from gesture callback deadlocked the pattern Mutex."
        );
    }
}
