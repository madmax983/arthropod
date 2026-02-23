//! Form state types
//!
//! This module defines the state structures used by form widgets to manage form data,
//! validation, and submission.
//!
//! The [`FormState`] is attached to the form container entity and tracks all child input fields.

use indexmap::IndexMap;
use render_engine::NodeId;
use std::collections::HashMap;
use std::sync::Arc;

/// Form data map (Field Name -> Field Value).
///
/// Represents a snapshot of the form's data at the time of submission.
pub type FormData = HashMap<String, String>;

/// Callback type for form submission.
///
/// It receives the [`FormData`] and returns a `Result`.
/// - `Ok(())`: Submission successful.
/// - `Err(String)`: Submission failed with an error message (which may be displayed to the user).
pub type SubmitCallback = Arc<dyn Fn(FormData) -> Result<(), String> + Send + Sync>;

/// State for a form widget.
///
/// This component is attached to the form container. It maps field names to their
/// corresponding node IDs (where the actual input state lives) and tracks the
/// overall validity and submission status.
#[derive(Clone)]
pub struct FormState {
    /// Mapping from field name to the NodeId of the input widget.
    ///
    /// This allows the form logic to collect values from all registered inputs.
    pub field_mapping: IndexMap<String, NodeId>,

    /// Whether the form is currently valid.
    ///
    /// This is updated by the validation system based on the state of child inputs.
    pub is_valid: bool,

    /// Optional callback to execute when the form is submitted.
    pub on_submit: Option<SubmitCallback>,

    /// Error message from the last failed submission, if any.
    pub submit_error: Option<String>,
}
