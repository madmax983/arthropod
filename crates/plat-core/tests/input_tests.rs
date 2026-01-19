//! Unit tests for input event types.

use plat_core::*;

#[test]
fn test_control_flow_default() {
    let cf = ControlFlow::default();
    assert_eq!(cf, ControlFlow::Poll);
}

#[test]
fn test_control_flow_equality() {
    assert_eq!(ControlFlow::Poll, ControlFlow::Poll);
    assert_eq!(ControlFlow::Wait, ControlFlow::Wait);
    assert_eq!(ControlFlow::Exit, ControlFlow::Exit);

    assert_ne!(ControlFlow::Poll, ControlFlow::Wait);
    assert_ne!(ControlFlow::Wait, ControlFlow::Exit);
    assert_ne!(ControlFlow::Poll, ControlFlow::Exit);
}

#[test]
fn test_element_state_equality() {
    assert_eq!(ElementState::Pressed, ElementState::Pressed);
    assert_eq!(ElementState::Released, ElementState::Released);
    assert_ne!(ElementState::Pressed, ElementState::Released);
}

#[test]
fn test_mouse_button_equality() {
    assert_eq!(MouseButton::Left, MouseButton::Left);
    assert_eq!(MouseButton::Right, MouseButton::Right);
    assert_eq!(MouseButton::Middle, MouseButton::Middle);
    assert_eq!(MouseButton::Other(1), MouseButton::Other(1));

    assert_ne!(MouseButton::Left, MouseButton::Right);
    assert_ne!(MouseButton::Other(1), MouseButton::Other(2));
}

#[test]
fn test_modifiers_default() {
    let mods = Modifiers::default();
    assert!(!mods.shift);
    assert!(!mods.ctrl);
    assert!(!mods.alt);
    assert!(!mods.meta);
}

#[test]
fn test_modifiers_construction() {
    let mods = Modifiers {
        shift: true,
        ctrl: false,
        alt: true,
        meta: false,
    };
    assert!(mods.shift);
    assert!(!mods.ctrl);
    assert!(mods.alt);
    assert!(!mods.meta);
}

#[test]
fn test_key_equality() {
    assert_eq!(Key::A, Key::A);
    assert_eq!(Key::Enter, Key::Enter);
    assert_eq!(Key::F1, Key::F1);
    assert_eq!(Key::Unknown, Key::Unknown);

    assert_ne!(Key::A, Key::B);
    assert_ne!(Key::F1, Key::F2);
}

#[test]
fn test_keyboard_input_construction() {
    let input = KeyboardInput {
        key: Key::A,
        state: ElementState::Pressed,
        modifiers: Modifiers {
            shift: true,
            ctrl: false,
            alt: false,
            meta: false,
        },
        repeat: false,
    };

    assert_eq!(input.key, Key::A);
    assert_eq!(input.state, ElementState::Pressed);
    assert!(input.modifiers.shift);
    assert!(!input.repeat);
}

#[test]
fn test_mouse_input_construction() {
    let input = MouseInput {
        button: MouseButton::Left,
        state: ElementState::Pressed,
        position: Point::new(10.5, 20.3),
        modifiers: Modifiers::default(),
    };

    assert_eq!(input.button, MouseButton::Left);
    assert_eq!(input.state, ElementState::Pressed);
    assert_eq!(input.position.x, 10.5);
    assert_eq!(input.position.y, 20.3);
}

#[test]
fn test_scroll_delta_line() {
    let delta = ScrollDelta::LineDelta(1.0, -2.0);
    match delta {
        ScrollDelta::LineDelta(x, y) => {
            assert_eq!(x, 1.0);
            assert_eq!(y, -2.0);
        }
        _ => panic!("Expected LineDelta"),
    }
}

#[test]
fn test_scroll_delta_pixel() {
    let delta = ScrollDelta::PixelDelta(10.5, -20.3);
    match delta {
        ScrollDelta::PixelDelta(x, y) => {
            assert_eq!(x, 10.5);
            assert_eq!(y, -20.3);
        }
        _ => panic!("Expected PixelDelta"),
    }
}

#[test]
fn test_window_event_close_requested() {
    let event = WindowEvent::CloseRequested;
    match event {
        WindowEvent::CloseRequested => {}
        _ => panic!("Expected CloseRequested"),
    }
}

#[test]
fn test_window_event_resized() {
    let event = WindowEvent::Resized(Size::new(800, 600));
    match event {
        WindowEvent::Resized(size) => {
            assert_eq!(size.width, 800);
            assert_eq!(size.height, 600);
        }
        _ => panic!("Expected Resized"),
    }
}

#[test]
fn test_window_event_focused() {
    let event = WindowEvent::Focused(true);
    match event {
        WindowEvent::Focused(focused) => assert!(focused),
        _ => panic!("Expected Focused"),
    }
}

#[test]
fn test_window_event_scale_factor_changed() {
    let event = WindowEvent::ScaleFactorChanged {
        scale_factor: 2.0,
        new_inner_size: Size::new(1600, 1200),
    };
    match event {
        WindowEvent::ScaleFactorChanged {
            scale_factor,
            new_inner_size,
        } => {
            assert_eq!(scale_factor, 2.0);
            assert_eq!(new_inner_size.width, 1600);
            assert_eq!(new_inner_size.height, 1200);
        }
        _ => panic!("Expected ScaleFactorChanged"),
    }
}

#[test]
fn test_window_event_redraw_requested() {
    let event = WindowEvent::RedrawRequested;
    match event {
        WindowEvent::RedrawRequested => {}
        _ => panic!("Expected RedrawRequested"),
    }
}

#[test]
fn test_window_event_keyboard_input() {
    let keyboard_input = KeyboardInput {
        key: Key::Enter,
        state: ElementState::Pressed,
        modifiers: Modifiers::default(),
        repeat: false,
    };
    let event = WindowEvent::KeyboardInput(keyboard_input.clone());
    match event {
        WindowEvent::KeyboardInput(input) => {
            assert_eq!(input.key, Key::Enter);
            assert_eq!(input.state, ElementState::Pressed);
        }
        _ => panic!("Expected KeyboardInput"),
    }
}

#[test]
fn test_window_event_mouse_input() {
    let mouse_input = MouseInput {
        button: MouseButton::Left,
        state: ElementState::Pressed,
        position: Point::new(100.0, 200.0),
        modifiers: Modifiers::default(),
    };
    let event = WindowEvent::MouseInput(mouse_input.clone());
    match event {
        WindowEvent::MouseInput(input) => {
            assert_eq!(input.button, MouseButton::Left);
            assert_eq!(input.state, ElementState::Pressed);
        }
        _ => panic!("Expected MouseInput"),
    }
}

#[test]
fn test_window_event_cursor_moved() {
    let event = WindowEvent::CursorMoved {
        position: Point::new(10.5, 20.3),
    };
    match event {
        WindowEvent::CursorMoved { position } => {
            assert_eq!(position.x, 10.5);
            assert_eq!(position.y, 20.3);
        }
        _ => panic!("Expected CursorMoved"),
    }
}

#[test]
fn test_window_event_cursor_entered() {
    let event = WindowEvent::CursorEntered(true);
    match event {
        WindowEvent::CursorEntered(entered) => assert!(entered),
        _ => panic!("Expected CursorEntered"),
    }
}

#[test]
fn test_window_event_mouse_wheel() {
    let event = WindowEvent::MouseWheel {
        delta: ScrollDelta::LineDelta(1.0, -1.0),
    };
    match event {
        WindowEvent::MouseWheel { delta } => match delta {
            ScrollDelta::LineDelta(x, y) => {
                assert_eq!(x, 1.0);
                assert_eq!(y, -1.0);
            }
            _ => panic!("Expected LineDelta"),
        },
        _ => panic!("Expected MouseWheel"),
    }
}

#[test]
fn test_lifecycle_event_resumed() {
    let event = LifecycleEvent::Resumed;
    match event {
        LifecycleEvent::Resumed => {}
        _ => panic!("Expected Resumed"),
    }
}

#[test]
fn test_lifecycle_event_suspended() {
    let event = LifecycleEvent::Suspended;
    match event {
        LifecycleEvent::Suspended => {}
        _ => panic!("Expected Suspended"),
    }
}

#[test]
fn test_event_window() {
    // Get a real WindowId from an actual window
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop
        .create_window(WindowConfig::default())
        .expect("Failed to create window");
    let window_id = window.id();

    let event = Event::Window {
        window_id,
        event: WindowEvent::CloseRequested,
    };

    match event {
        Event::Window {
            window_id: id,
            event,
        } => {
            assert_eq!(id, window_id);
            match event {
                WindowEvent::CloseRequested => {}
                _ => panic!("Expected CloseRequested"),
            }
        }
        _ => panic!("Expected Window event"),
    }
}

#[test]
fn test_event_lifecycle() {
    let event = Event::Lifecycle(LifecycleEvent::Resumed);

    match event {
        Event::Lifecycle(lifecycle) => match lifecycle {
            LifecycleEvent::Resumed => {}
            _ => panic!("Expected Resumed"),
        },
        _ => panic!("Expected Lifecycle event"),
    }
}
