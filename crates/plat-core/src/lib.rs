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
///
/// These errors represent fundamental failures in the underlying operating system
/// abstraction, such as being unable to allocate a window surface or start the event
/// loop. These are typically fatal application errors that cannot be easily recovered
/// from at runtime.
///
/// ## Examples
///
/// ```
/// use plat_core::{EventLoop, PlatformError};
///
/// fn try_init() -> Result<EventLoop, PlatformError> {
///     // This might fail if the OS denies the windowing context
///     EventLoop::new()
/// }
/// ```
#[derive(Error, Debug)]
pub enum PlatformError {
    /// Represents a failure to ask the OS to allocate a new window surface.
    /// This happens when passing invalid `WindowConfig` constraints (like impossible
    /// dimensions) or when the OS window manager is out of resources.
    #[error("Failed to create window: {0}")]
    WindowCreation(String),

    /// Represents a failure to bootstrap the platform's core graphics or event APIs.
    /// On Windows, this might mean COM initialization failed. On Web, it might mean
    /// the browser environment is missing critical APIs (like `web_sys::window`).
    #[error("Platform initialization failed: {0}")]
    Initialization(String),

    /// Represents a critical failure while pumping events from the OS.
    /// This usually indicates the OS has forcibly severed the application's connection
    /// to the display server (e.g. Wayland compositor crash).
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

    /// Called every frame when the application is in Poll mode.
    fn on_update(&mut self, _delta: std::time::Duration) {}
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
