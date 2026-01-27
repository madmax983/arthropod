//! Platform-specific implementations.

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::*;

// Fallback for unsupported platforms (allows compilation for testing)
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod stub;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub use self::stub::*;
