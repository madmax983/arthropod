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
//! use widget_core::{Form, TextInput};
//!
//! fn main() -> Result<(), AppError> {
//!     App::run("My Form", 400, 300, |ctx| {
//!         let name = ctx.signal(String::new());
//!         Form::new((
//!             ("name", TextInput::new(name)),
//!         ))
//!     })
//! }
//! ```

pub mod core;
pub mod integration;
pub mod widget;

// Re-export main types to maintain API compatibility
pub use self::core::{App, AppError};
pub use widget::{AppContext, WidgetExt};
