use plat_core::{
    ElementState, Key, MouseButton, ScrollDelta, Size, WindowEvent, map_key_input,
    map_pointer_down, map_pointer_move, map_pointer_up, map_resize_events, normalize_wheel,
    wheel_event_from_input,
};

#[test]
fn test_wheel_delta_mode_line_is_normalized_to_pixels() {
    let px = normalize_wheel(3.0, 1);
    assert_eq!(px, 48.0);
}

#[test]
fn test_pointer_mapping_emits_mouse_and_cursor_events() {
    let down = map_pointer_down(0, 32.0, 48.0);
    let up = map_pointer_up(2, 64.0, 96.0);
    let moved = map_pointer_move(10.0, 20.0);

    match down {
        WindowEvent::MouseInput(input) => {
            assert_eq!(input.button, MouseButton::Left);
            assert_eq!(input.state, ElementState::Pressed);
        }
        other => panic!("expected MouseInput, got {other:?}"),
    }

    match up {
        WindowEvent::MouseInput(input) => {
            assert_eq!(input.button, MouseButton::Right);
            assert_eq!(input.state, ElementState::Released);
        }
        other => panic!("expected MouseInput, got {other:?}"),
    }

    match moved {
        WindowEvent::CursorMoved { position } => {
            assert_eq!(position.x, 10.0);
            assert_eq!(position.y, 20.0);
        }
        other => panic!("expected CursorMoved, got {other:?}"),
    }
}

#[test]
fn test_keyboard_mapping_maps_common_codes() {
    let key_a = map_key_input("KeyA", "a", ElementState::Pressed, false);
    let key_escape = map_key_input("Escape", "Escape", ElementState::Released, false);

    assert_eq!(key_a.key, Key::A);
    assert_eq!(key_escape.key, Key::Escape);
}

#[test]
fn test_resize_mapping_emits_resized_and_scale_factor_changed() {
    let (resized, scale_changed) = map_resize_events(640.0, 360.0, 2.0);

    match resized {
        WindowEvent::Resized(size) => assert_eq!(size, Size::new(1280, 720)),
        other => panic!("expected Resized, got {other:?}"),
    }

    match scale_changed {
        WindowEvent::ScaleFactorChanged {
            scale_factor,
            new_inner_size,
        } => {
            assert_eq!(scale_factor, 2.0);
            assert_eq!(new_inner_size, Size::new(1280, 720));
        }
        other => panic!("expected ScaleFactorChanged, got {other:?}"),
    }
}

#[test]
fn test_wheel_event_uses_pixel_delta_after_normalization() {
    let event = wheel_event_from_input(1.0, -2.0, 1);
    match event {
        WindowEvent::MouseWheel {
            delta: ScrollDelta::PixelDelta(x, y),
        } => {
            assert_eq!(x, 16.0);
            assert_eq!(y, -32.0);
        }
        other => panic!("expected PixelDelta wheel event, got {other:?}"),
    }
}
