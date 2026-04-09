use input_engine::gestures::{InputPattern, SequenceMatcher};
use plat_core::{ElementState, Key, KeyboardInput, WindowEvent};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_sequence_matcher_fuzz(keys in prop::collection::vec(any::<u8>(), 0..100)) {
        let konami = vec![
            Key::Up, Key::Up, Key::Down, Key::Down, Key::Left, Key::Right, Key::Left, Key::Right, Key::B, Key::A
        ];
        let mut matcher = SequenceMatcher::new(konami, "CheatCode");

        for k in keys {
            let key = match k % 6 {
                0 => Key::Up,
                1 => Key::Down,
                2 => Key::Left,
                3 => Key::Right,
                4 => Key::A,
                _ => Key::B,
            };

            let event = WindowEvent::KeyboardInput(KeyboardInput {
                key,
                state: ElementState::Pressed,
                modifiers: Default::default(),
                repeat: false,
            });
            matcher.update(&event);
        }
    }
}
