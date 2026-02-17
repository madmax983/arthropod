//! Validation support for form inputs.
//!
//! This module provides the core types and traits for field-level validation
//! in the Arthropod widget system. It is designed to be simple, reusable,
//! and thread-safe.
//!
//! # Core Concepts
//!
//! - **Validator**: A closure/function that takes a string input and returns `Result<(), String>`.
//! - **ValidationResult**: A type alias for `Result<(), String>`.
//!
//! # Usage
//!
//! Validators are typically used with the [`TextInput`](crate::TextInput) widget.
//!
//! ```
//! use widget_core::validation::Validator;
//! use widget_core::TextInput;
//! use flux_state::{Runtime, Signal};
//! use std::sync::Arc;
//!
//! // 1. Define a reusable validator
//! let email_validator = |s: &str| {
//!     if s.contains('@') {
//!         Ok(())
//!     } else {
//!         Err("Invalid email address".to_string())
//!     }
//! };
//!
//! // 2. Apply it to a widget
//! # let runtime = Runtime::new();
//! # let value = Signal::new(runtime, String::new());
//! let input = TextInput::new(value)
//!     .validator(email_validator);
//! ```
//!
//! # Creating Custom Validators
//!
//! You can create factory functions that return validators for complex logic:
//!
//! ```
//! use widget_core::validation::Validator;
//! use std::sync::Arc;
//!
//! fn min_length(min: usize) -> impl Fn(&str) -> Result<(), String> {
//!     move |s: &str| {
//!         if s.len() >= min {
//!             Ok(())
//!         } else {
//!             Err(format!("Must be at least {} characters", min))
//!         }
//!     }
//! }
//!
//! // Usage:
//! // .validator(min_length(8))
//! ```

use std::sync::Arc;

/// A thread-safe validation function.
///
/// Validators accept a string slice (the current input value) and return:
/// - `Ok(())` if valid.
/// - `Err(String)` with an error message if invalid.
///
/// They must be `Send + Sync` to support multi-threaded rendering contexts.
pub type Validator = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// The result of a validation check.
///
/// `Ok(())` means valid, `Err(String)` contains the error message.
pub type ValidationResult = Result<(), String>;
