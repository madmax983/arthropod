//! Platform abstraction for accessibility bridges
//!
//! Each platform (Windows UIA, macOS NSAccessibility, Linux AT-SPI) implements this trait
//! to synchronize the platform-agnostic A11yTree with the OS accessibility API.

pub mod accesskit_bridge;
pub mod action_handler;

#[cfg(target_os = "windows")]
pub mod windows;
