//! Integration tests for window creation and management.

use plat_core::{EventLoop, Size, WindowConfig};

#[test]
fn test_create_event_loop() {
    // Test that we can create an event loop
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
    // Note: Actual size might differ slightly due to DPI/borders, but should be close
    assert!(
        (size.width as i32 - 640).abs() < 50,
        "Window width {} is too different from requested 640",
        size.width
    );
    assert!(
        (size.height as i32 - 480).abs() < 50,
        "Window height {} is too different from requested 480",
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
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop
        .create_window(WindowConfig::default())
        .expect("Failed to create window");

    // Should be able to get raw window and display handles for wgpu
    let window_handle = window.window_handle();
    assert!(window_handle.is_ok(), "Failed to get window handle");

    let display_handle = window.display_handle();
    assert!(display_handle.is_ok(), "Failed to get display handle");
}

#[test]
fn test_multiple_windows_class_registration() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    // Create multiple windows to ensure class registration only happens once
    // Second window should not fail due to duplicate class registration
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
