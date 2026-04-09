use input_engine::gestures::{ChordMatcher, InputPattern};
use plat_core::{ElementState, Key, KeyboardInput, WindowEvent};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_chord_matcher_duplicates(
        keys in prop::collection::vec(any::<bool>(), 0..100)
    ) {
        let required_keys = vec![Key::Control, Key::S];
        let mut matcher = ChordMatcher::new(required_keys, "Save");

        for pressed in keys {
            let state = if pressed { ElementState::Pressed } else { ElementState::Released };

            // Fuzz duplicates
            let event = WindowEvent::KeyboardInput(KeyboardInput {
                key: Key::S,
                state,
                modifiers: Default::default(),
                repeat: false,
            });
            matcher.update(&event);
        }
    }
}
