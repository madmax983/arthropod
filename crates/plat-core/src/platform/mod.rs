//! Platform-specific implementations.

pub(crate) mod web_runtime;

#[cfg(all(target_arch = "wasm32", feature = "web"))]
mod web;
#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub use self::web::*;

#[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
pub mod windows;
#[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
pub use self::windows::*;

#[cfg(all(target_os = "macos", not(target_arch = "wasm32")))]
mod macos;
#[cfg(all(target_os = "macos", not(target_arch = "wasm32")))]
pub use self::macos::*;

// Fallback for unsupported platforms (allows compilation for testing)
#[cfg(not(any(
    all(target_arch = "wasm32", feature = "web"),
    target_os = "windows",
    target_os = "macos"
)))]
mod stub;
#[cfg(not(any(
    all(target_arch = "wasm32", feature = "web"),
    target_os = "windows",
    target_os = "macos"
)))]
pub use self::stub::*;
