//! Platform abstraction layer for Arthropod GUI framework.
//!
//! This crate provides cross-platform abstractions for:
//! - Window management
//! - Input handling (keyboard, mouse)
//! - GPU surface creation via raw-window-handle
//!
//! # Platform Support
//! - Windows (Win32)
//! - macOS (Cocoa via objc2)

mod compositor;
mod input;
mod materials;
mod platform;
mod window;

pub use compositor::{Compositor, Layer};
pub use input::*;
pub use materials::*;
pub use platform::web_runtime::{
    map_key_input, map_key_input_with_modifiers, map_modifiers, map_pointer_down,
    map_pointer_down_with_modifiers, map_pointer_move, map_pointer_up,
    map_pointer_up_with_modifiers, map_resize_events, normalize_wheel, wheel_event_from_input,
};
pub use window::*;

use thiserror::Error;

/// Errors that can occur in plat-core operations.
#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Failed to create window: {0}")]
    WindowCreation(String),

    #[error("Platform initialization failed: {0}")]
    Initialization(String),

    #[error("Event loop error: {0}")]
    EventLoop(String),
}

/// Control flow for the event loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlFlow {
    /// Continue running, polling for events.
    #[default]
    Poll,
    /// Wait for events before continuing.
    Wait,
    /// Exit the application.
    Exit,
}

/// Application trait that users implement.
pub trait Application: Sized + 'static {
    /// Called once when the application starts.
    fn new(event_loop: &EventLoop) -> Self;

    /// Called when an event occurs.
    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow);

    /// Called when a redraw is requested for a window.
    fn on_redraw(&mut self, window_id: WindowId);
}

/// The event loop - manages window lifecycle and event dispatch.
pub struct EventLoop {
    inner: platform::EventLoopImpl,
}

impl EventLoop {
    /// Create a new event loop.
    pub fn new() -> Result<Self, PlatformError> {
        Ok(Self {
            inner: platform::EventLoopImpl::new()?,
        })
    }

    /// Create a new window.
    pub fn create_window(&self, config: WindowConfig) -> Result<Window, PlatformError> {
        self.inner.create_window(config)
    }
}

/// Run the application - this is the main entry point.
pub fn run<A: Application>() -> Result<(), PlatformError> {
    platform::run::<A>()
}
