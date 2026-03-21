//! Arthropod application builder and runtime
//!
//! Provides a high-level API for creating and managing Arthropod applications.
//! All resources (Scene, Runtime, WgpuBackend) are managed automatically.
//!
//! # High-Level API
//!
//! For widget-based applications, use [`App::run()`]:
//!
//! ```no_run
//! use arthropod::prelude::*;
//!
//! fn main() -> Result<(), AppError> {
//!     App::run("Welcome App", 400, 300, |ctx| {
//!         // 1. Create reactive state (a string)
//!         let name_signal = ctx.signal(String::from("Arthropod"));
//!         let (read_name, _write_name) = name_signal.split();
//!
//!         // 2. Build declarative UI tree using macros
//!         Center::new(
//!             col!(
//!                 [
//!                     txt!("Welcome to", size: 24.0),
//!                     // Use @ syntax to pass the reactive signal
//!                     txt!(@read_name, size: 48.0),
//!                 ],
//!                 gap: 20.0,
//!                 padding: 40.0
//!             )
//!         )
//!     })
//! }
//! ```

/// Core application definition, rendering loops, and ECS context management.
pub mod core;
/// Logic to integrate standard OS window events into the Arthropod application event model.
pub mod integration;
/// Event-driven controller managing dynamic behaviors such as text input focus and form state.
pub mod widget;

// Re-export main types to maintain API compatibility
pub use self::core::{App, AppError};
pub use widget::AppContext;
