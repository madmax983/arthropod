//! Integration tests for Windows-specific functionality.
//!
//! These tests exercise the event loop and window message handling.

#![cfg(target_os = "windows")]

use plat_core::*;

/// Minimal application for testing basic run functionality
struct MinimalApp {
    _window: Option<Window>,
}

impl Application for MinimalApp {
    fn new(event_loop: &EventLoop) -> Self {
        // Create a window to generate events
        let window = event_loop.create_window(WindowConfig::default()).ok();
        Self { _window: window }
    }

    fn on_event(&mut self, _event: Event, control_flow: &mut ControlFlow) {
        // Exit immediately on first event
        *control_flow = ControlFlow::Exit;
    }

    fn on_redraw(&mut self, _window_id: WindowId) {}
}

#[test]
#[ignore] // Ignored by default because it creates a real event loop
fn test_run_minimal_app() {
    // Test that run() can start and stop without errors
    let result = run::<MinimalApp>();
    assert!(result.is_ok(), "run() should succeed: {:?}", result.err());
}

/// Application that creates a window and immediately exits
struct WindowCreationApp {
    _window: Option<Window>,
}

impl Application for WindowCreationApp {
    fn new(event_loop: &EventLoop) -> Self {
        let window = event_loop.create_window(WindowConfig::default()).ok();
        Self { _window: window }
    }

    fn on_event(&mut self, _event: Event, control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Exit;
    }

    fn on_redraw(&mut self, _window_id: WindowId) {}
}

#[test]
#[ignore] // Ignored by default because it creates a real event loop
fn test_run_with_window_creation() {
    let result = run::<WindowCreationApp>();
    assert!(
        result.is_ok(),
        "run() with window creation should succeed: {:?}",
        result.err()
    );
}

/// Application that tests window visibility changes
struct VisibilityTestApp {
    _window: Option<Window>,
}

impl Application for VisibilityTestApp {
    fn new(event_loop: &EventLoop) -> Self {
        let config = WindowConfig {
            visible: true, // Start visible to generate initial events
            ..Default::default()
        };
        let window = event_loop.create_window(config).ok();
        // Test visibility immediately
        if let Some(ref w) = window {
            w.set_visible(true);
            w.set_visible(false);
            w.set_visible(true);
        }
        Self { _window: window }
    }

    fn on_event(&mut self, _event: Event, control_flow: &mut ControlFlow) {
        // Exit on first event
        *control_flow = ControlFlow::Exit;
    }

    fn on_redraw(&mut self, _window_id: WindowId) {}
}

#[test]
#[ignore] // Ignored by default because it creates a real event loop
fn test_window_visibility_toggle() {
    let result = run::<VisibilityTestApp>();
    assert!(
        result.is_ok(),
        "Visibility toggle test failed: {:?}",
        result.err()
    );
}

/// Application that tests request_redraw
struct RedrawTestApp {
    window: Option<Window>,
    requested_redraw: bool,
}

impl Application for RedrawTestApp {
    fn new(event_loop: &EventLoop) -> Self {
        let window = event_loop.create_window(WindowConfig::default()).ok();
        Self {
            window,
            requested_redraw: false,
        }
    }

    fn on_event(&mut self, _event: Event, control_flow: &mut ControlFlow) {
        if !self.requested_redraw {
            if let Some(window) = &self.window {
                window.request_redraw();
                self.requested_redraw = true;
            }
        }
        *control_flow = ControlFlow::Exit;
    }

    fn on_redraw(&mut self, _window_id: WindowId) {}
}

#[test]
#[ignore] // Ignored by default because it creates a real event loop
fn test_request_redraw() {
    let result = run::<RedrawTestApp>();
    assert!(
        result.is_ok(),
        "Request redraw test failed: {:?}",
        result.err()
    );
}

/// Application that tests control flow modes
struct ControlFlowTestApp {
    _window: Option<Window>,
}

impl Application for ControlFlowTestApp {
    fn new(event_loop: &EventLoop) -> Self {
        // Create a window to generate events
        let window = event_loop.create_window(WindowConfig::default()).ok();
        // Test that we can set different control flow values
        let _poll = ControlFlow::Poll;
        let _wait = ControlFlow::Wait;
        let _exit = ControlFlow::Exit;
        Self { _window: window }
    }

    fn on_event(&mut self, _event: Event, control_flow: &mut ControlFlow) {
        // Test that we can use Poll mode
        *control_flow = ControlFlow::Poll;
        // But then exit immediately
        *control_flow = ControlFlow::Exit;
    }

    fn on_redraw(&mut self, _window_id: WindowId) {}
}

#[test]
#[ignore] // Ignored by default because it creates a real event loop
fn test_control_flow_modes() {
    let result = run::<ControlFlowTestApp>();
    assert!(
        result.is_ok(),
        "Control flow test failed: {:?}",
        result.err()
    );
}

/// Test that multiple windows can be created and tracked
struct MultiWindowApp {
    _windows: Vec<Window>,
}

impl Application for MultiWindowApp {
    fn new(event_loop: &EventLoop) -> Self {
        let mut windows = Vec::new();
        for i in 0..3 {
            if let Ok(window) = event_loop.create_window(WindowConfig {
                title: format!("Window {}", i),
                visible: true, // Make visible to generate events
                ..Default::default()
            }) {
                windows.push(window);
            }
        }
        // Verify unique IDs here
        let ids: Vec<_> = windows.iter().map(|w| w.id()).collect();
        for i in 0..ids.len() {
            for j in i + 1..ids.len() {
                assert_ne!(ids[i], ids[j], "Windows should have unique IDs");
            }
        }
        Self { _windows: windows }
    }

    fn on_event(&mut self, _event: Event, control_flow: &mut ControlFlow) {
        // Exit on first event
        *control_flow = ControlFlow::Exit;
    }

    fn on_redraw(&mut self, _window_id: WindowId) {}
}

#[test]
#[ignore] // Ignored by default because it creates a real event loop
fn test_multiple_windows() {
    let result = run::<MultiWindowApp>();
    assert!(
        result.is_ok(),
        "Multiple windows test failed: {:?}",
        result.err()
    );
}
