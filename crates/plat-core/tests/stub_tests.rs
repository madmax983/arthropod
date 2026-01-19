//! Tests for stub platform (unsupported platforms).
//!
//! These tests verify that the stub implementation correctly reports errors
//! on platforms that aren't yet supported.

#![cfg(not(any(target_os = "windows", target_os = "macos")))]

use plat_core::{Application, ControlFlow, Event, EventLoop, WindowConfig, WindowId};

struct StubApp;

impl Application for StubApp {
    fn new(_event_loop: &EventLoop) -> Self {
        StubApp
    }

    fn on_event(&mut self, _event: Event, _control_flow: &mut ControlFlow) {}

    fn on_redraw(&mut self, _window_id: WindowId) {}
}

#[test]
fn test_event_loop_creation_fails() {
    let result = EventLoop::new();
    assert!(
        result.is_err(),
        "Event loop creation should fail on unsupported platforms"
    );

    match result {
        Err(plat_core::PlatformError::Initialization(msg)) => {
            assert!(
                msg.contains("not supported"),
                "Error message should indicate platform not supported"
            );
        }
        _ => panic!("Expected Initialization error"),
    }
}

#[test]
fn test_run_fails() {
    let result = plat_core::run::<StubApp>();
    assert!(
        result.is_err(),
        "run() should fail on unsupported platforms"
    );

    match result {
        Err(plat_core::PlatformError::Initialization(msg)) => {
            assert!(
                msg.contains("not supported"),
                "Error message should indicate platform not supported"
            );
        }
        _ => panic!("Expected Initialization error"),
    }
}
