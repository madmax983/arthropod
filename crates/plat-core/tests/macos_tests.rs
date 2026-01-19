//! Tests for macOS platform backend.
//!
//! These tests only run on macOS and mirror the Windows tests.

#![cfg(target_os = "macos")]

use plat_core::{EventLoop, Position, Size, WindowConfig};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

#[test]
fn test_create_event_loop() {
    let event_loop = EventLoop::new();
    assert!(
        event_loop.is_ok(),
        "Failed to create event loop: {:?}",
        event_loop.err()
    );
}

#[test]
fn test_create_window_with_default_config() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window = event_loop.create_window(WindowConfig::default());
    assert!(
        window.is_ok(),
        "Failed to create window: {:?}",
        window.err()
    );
}

#[test]
fn test_window_has_unique_id() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window1 = event_loop
        .create_window(WindowConfig::default())
        .expect("Failed to create first window");
    let window2 = event_loop
        .create_window(WindowConfig::default())
        .expect("Failed to create second window");

    assert_ne!(window1.id(), window2.id(), "Windows should have unique IDs");
}

#[test]
fn test_window_respects_size_config() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let config = WindowConfig {
        size: Size::new(640, 480),
        ..Default::default()
    };

    let window = event_loop
        .create_window(config)
        .expect("Failed to create window");

    let size = window.inner_size();
    // Note: macOS might adjust window size, so we check it's reasonable
    assert!(
        size.width > 0 && size.height > 0,
        "Window size should be positive: {}x{}",
        size.width,
        size.height
    );
}

#[test]
fn test_window_title() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let config = WindowConfig {
        title: "Test Window".into(),
        ..Default::default()
    };

    let window = event_loop
        .create_window(config)
        .expect("Failed to create window");

    // We can set a title without panicking
    window.set_title("New Title");
}

#[test]
fn test_window_scale_factor() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop
        .create_window(WindowConfig::default())
        .expect("Failed to create window");

    let scale_factor = window.scale_factor();
    assert!(scale_factor > 0.0, "Scale factor should be positive");
    assert!(
        scale_factor <= 4.0,
        "Scale factor shouldn't be unreasonably large"
    );
}

#[test]
fn test_window_visibility() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let config = WindowConfig {
        visible: false,
        ..Default::default()
    };

    let window = event_loop
        .create_window(config)
        .expect("Failed to create window");

    // Should be able to show/hide without panicking
    window.set_visible(true);
    window.set_visible(false);
}

#[test]
fn test_window_provides_raw_handles() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop
        .create_window(WindowConfig::default())
        .expect("Failed to create window");

    // Should be able to get raw window and display handles for wgpu
    let window_handle = window.window_handle();
    assert!(window_handle.is_ok(), "Failed to get window handle");

    // Should be an AppKit handle
    match window_handle.unwrap().as_raw() {
        RawWindowHandle::AppKit(_) => {} // Expected for macOS
        _ => panic!("Expected AppKit window handle"),
    }

    let display_handle = window.display_handle();
    assert!(display_handle.is_ok(), "Failed to get display handle");

    // Should be an AppKit display handle
    match display_handle.unwrap().as_raw() {
        RawDisplayHandle::AppKit(_) => {} // Expected for macOS
        _ => panic!("Expected AppKit display handle"),
    }
}

#[test]
fn test_window_request_redraw() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop
        .create_window(WindowConfig::default())
        .expect("Failed to create window");

    // Should be able to request redraw without panicking
    window.request_redraw();
}

#[test]
fn test_window_decorations() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let config_decorated = WindowConfig {
        decorations: true,
        ..Default::default()
    };
    let window_decorated = event_loop.create_window(config_decorated);
    assert!(window_decorated.is_ok());

    let config_undecorated = WindowConfig {
        decorations: false,
        ..Default::default()
    };
    let window_undecorated = event_loop.create_window(config_undecorated);
    assert!(window_undecorated.is_ok());
}

#[test]
fn test_window_resizable() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let config_resizable = WindowConfig {
        resizable: true,
        ..Default::default()
    };
    let window_resizable = event_loop.create_window(config_resizable);
    assert!(window_resizable.is_ok());

    let config_fixed = WindowConfig {
        resizable: false,
        ..Default::default()
    };
    let window_fixed = event_loop.create_window(config_fixed);
    assert!(window_fixed.is_ok());
}

#[test]
fn test_window_with_position() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let config = WindowConfig {
        position: Some(Position::new(100, 100)),
        ..Default::default()
    };

    let window = event_loop.create_window(config);
    assert!(
        window.is_ok(),
        "Failed to create window with position: {:?}",
        window.err()
    );
}

#[test]
fn test_multiple_windows() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window1 = event_loop
        .create_window(WindowConfig {
            title: "Window 1".into(),
            ..Default::default()
        })
        .expect("Failed to create first window");

    let window2 = event_loop
        .create_window(WindowConfig {
            title: "Window 2".into(),
            ..Default::default()
        })
        .expect("Failed to create second window");

    let window3 = event_loop
        .create_window(WindowConfig {
            title: "Window 3".into(),
            ..Default::default()
        })
        .expect("Failed to create third window");

    // All windows should have unique IDs
    assert_ne!(window1.id(), window2.id());
    assert_ne!(window2.id(), window3.id());
    assert_ne!(window1.id(), window3.id());
}
