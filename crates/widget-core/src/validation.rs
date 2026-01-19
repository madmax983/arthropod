//! Validation support for form inputs

use std::sync::Arc;

/// Validation function type
pub type Validator = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// Validation result
pub type ValidationResult = Result<(), String>;
