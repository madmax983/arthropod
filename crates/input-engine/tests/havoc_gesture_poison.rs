use flux_state::{Runtime, Signal};
use input_engine::gestures::{InputPattern, create_gesture_signal};
use plat_core::{ElementState, Key, KeyboardInput, WindowEvent};

#[derive(Clone, Debug, PartialEq)]
enum MyGesture {
    Hit,
}

struct PoisonMatcher {
    gesture: MyGesture,
}

impl InputPattern for PoisonMatcher {
    type Gesture = MyGesture;
    fn update(&mut self, _event: &WindowEvent) -> Option<Self::Gesture> {
        panic!("Die!");
    }
}

#[test]
fn test_havoc_gesture_poison() {
    let runtime = Runtime::new();
    let input = Signal::new(runtime.clone(), None);
    let (read_input, write_input) = input.split();

    let matcher = PoisonMatcher {
        gesture: MyGesture::Hit,
    };

    let _gesture_sig = create_gesture_signal(runtime.clone(), read_input, matcher);

    // Trigger the gesture - this should panic and poison the mutex
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        write_input.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
            key: Key::A,
            state: ElementState::Pressed,
            repeat: false,
            modifiers: Default::default(),
        })));
    }));

    assert!(result.is_err());

    // Second trigger - should hit expect("Mutex poisoned") inside the effect
    let result2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        write_input.set(Some(WindowEvent::KeyboardInput(KeyboardInput {
            key: Key::B,
            state: ElementState::Pressed,
            repeat: false,
            modifiers: Default::default(),
        })));
    }));

    assert!(result2.is_err());
}
