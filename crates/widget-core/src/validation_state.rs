//! Validation state types

use crate::validation::Validator;

/// Validation state for a node
#[derive(Clone)]
pub struct ValidationState {
    pub validator: Validator,
    pub error: Option<String>,
}
