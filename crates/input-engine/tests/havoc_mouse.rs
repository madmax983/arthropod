#![cfg(feature = "nova")]

use input_engine::gestures::InputPattern;
use input_engine::mouse_gestures::StrokeMatcher;
use plat_core::{ElementState, MouseButton, MouseInput, Point, WindowEvent};

#[test]
fn test_havoc_stroke_three_points_atan2_same_point() {
    let mut matcher = StrokeMatcher::new(MouseButton::Left);
    let event1 = WindowEvent::MouseInput(MouseInput {
        button: MouseButton::Left,
        state: ElementState::Pressed,
        position: Point::new(0.0, 0.0),
        modifiers: Default::default(),
    });
    matcher.update(&event1);

    // We want three points
    matcher.update(&WindowEvent::CursorMoved {
        position: Point::new(10.0, 0.0),
    });
    matcher.update(&WindowEvent::CursorMoved {
        position: Point::new(20.0, 0.0),
    });

    let event_move = WindowEvent::MouseInput(MouseInput {
        button: MouseButton::Left,
        state: ElementState::Released,
        position: Point::new(10.0, 0.0), // p1, p2, p3 where v1_x = v1_y = 0 or v2_x = v2_y = 0
        modifiers: Default::default(),
    });
    matcher.update(&event_move);
}
