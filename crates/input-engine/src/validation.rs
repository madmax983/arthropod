//! Validation support for form inputs.
//!
//! This module provides the core types and traits for field-level validation.

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

/// Validation state for a node.
///
/// Stores the validator logic and the current error state (if any).
#[derive(Clone)]
pub struct ValidationState {
    /// The validator function to run.
    pub validator: Validator,
    /// The current validation error, or None if valid.
    pub error: Option<String>,
}
