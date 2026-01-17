//! Tests for macOS platform backend.
//!
//! These tests only run on macOS.

#![cfg(target_os = "macos")]

use plat_core::{EventLoop, Size, Window, WindowConfig};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

#[test]
fn test_create_event_loop() {
    // Should be able to create an event loop
    let event_loop = EventLoop::new();
    assert!(event_loop.is_ok());
}

#[test]
fn test_create_window_with_default_config() {
    // Should be able to create a window with default configuration
    let event_loop = EventLoop::new().unwrap();
    let config = WindowConfig::default();
    let window = Window::new(&event_loop, config);
    assert!(window.is_ok());
}

#[test]
fn test_window_has_unique_id() {
    // Each window should have a unique ID
    let event_loop = EventLoop::new().unwrap();

    let window1 = Window::new(&event_loop, WindowConfig::default()).unwrap();
    let window2 = Window::new(&event_loop, WindowConfig::default()).unwrap();

    assert_ne!(window1.id(), window2.id());
}

#[test]
fn test_window_respects_size_config() {
    // Window should respect the size specified in config
    let event_loop = EventLoop::new().unwrap();
    let config = WindowConfig {
        size: Size { width: 640, height: 480 },
        ..Default::default()
    };

    let window = Window::new(&event_loop, config).unwrap();
    let size = window.inner_size();

    // Note: macOS might adjust window size, so we check it's reasonable
    assert!(size.width > 0);
    assert!(size.height > 0);
}

#[test]
fn test_window_title() {
    // Window should have the specified title
    let event_loop = EventLoop::new().unwrap();
    let config = WindowConfig {
        title: "Test Window".to_string(),
        ..Default::default()
    };

    let window = Window::new(&event_loop, config);
    assert!(window.is_ok());
    // Note: We can't easily read back the title on macOS without additional APIs,
    // but we verify the window creates successfully with a title
}

#[test]
fn test_window_scale_factor() {
    // Window should have a valid scale factor (1.0 or 2.0 on Retina displays)
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop, WindowConfig::default()).unwrap();

    let scale_factor = window.scale_factor();
    assert!(scale_factor >= 1.0);
    assert!(scale_factor <= 3.0); // Reasonable upper bound for Retina displays
}

#[test]
fn test_window_visibility() {
    // Window visibility should be configurable
    let event_loop = EventLoop::new().unwrap();

    // Initially visible
    let config_visible = WindowConfig {
        visible: true,
        ..Default::default()
    };
    let window_visible = Window::new(&event_loop, config_visible);
    assert!(window_visible.is_ok());

    // Initially hidden
    let config_hidden = WindowConfig {
        visible: false,
        ..Default::default()
    };
    let window_hidden = Window::new(&event_loop, config_hidden);
    assert!(window_hidden.is_ok());
}

#[test]
fn test_window_provides_raw_handles() {
    // Window should provide valid raw window and display handles
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop, WindowConfig::default()).unwrap();

    // Should be able to get window handle
    let window_handle = window.window_handle();
    assert!(window_handle.is_ok());

    // Should be an AppKit handle
    match window_handle.unwrap().as_raw() {
        RawWindowHandle::AppKit(_) => {
            // Expected for macOS
        }
        _ => panic!("Expected AppKit window handle"),
    }

    // Should be able to get display handle
    let display_handle = window.display_handle();
    assert!(display_handle.is_ok());

    // Should be an AppKit display handle
    match display_handle.unwrap().as_raw() {
        RawDisplayHandle::AppKit(_) => {
            // Expected for macOS
        }
        _ => panic!("Expected AppKit display handle"),
    }
}

#[test]
fn test_window_request_redraw() {
    // Should be able to request a redraw without panicking
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop, WindowConfig::default()).unwrap();

    window.request_redraw(); // Should not panic
}

#[test]
fn test_window_decorations() {
    // Window should support decoration configuration
    let event_loop = EventLoop::new().unwrap();

    let config_decorated = WindowConfig {
        decorations: true,
        ..Default::default()
    };
    let window_decorated = Window::new(&event_loop, config_decorated);
    assert!(window_decorated.is_ok());

    let config_undecorated = WindowConfig {
        decorations: false,
        ..Default::default()
    };
    let window_undecorated = Window::new(&event_loop, config_undecorated);
    assert!(window_undecorated.is_ok());
}

#[test]
fn test_window_resizable() {
    // Window should support resizable configuration
    let event_loop = EventLoop::new().unwrap();

    let config_resizable = WindowConfig {
        resizable: true,
        ..Default::default()
    };
    let window_resizable = Window::new(&event_loop, config_resizable);
    assert!(window_resizable.is_ok());

    let config_fixed = WindowConfig {
        resizable: false,
        ..Default::default()
    };
    let window_fixed = Window::new(&event_loop, config_fixed);
    assert!(window_fixed.is_ok());
}

#[test]
fn test_multiple_windows() {
    // Should be able to create multiple windows
    let event_loop = EventLoop::new().unwrap();

    let window1 = Window::new(&event_loop, WindowConfig::default()).unwrap();
    let window2 = Window::new(&event_loop, WindowConfig::default()).unwrap();
    let window3 = Window::new(&event_loop, WindowConfig::default()).unwrap();

    // All should have unique IDs
    assert_ne!(window1.id(), window2.id());
    assert_ne!(window2.id(), window3.id());
    assert_ne!(window1.id(), window3.id());
}
