//! Form state types
//!
//! Contains state definitions for forms and form submission.

use indexmap::IndexMap;
use render_engine::NodeId;
use std::collections::HashMap;
use std::sync::Arc;

/// Form data (field name -> field value)
pub type FormData = HashMap<String, String>;

/// Form submit callback type
pub type SubmitCallback = Arc<dyn Fn(FormData) -> Result<(), String> + Send + Sync>;

/// Form state for tracking form fields and validation
#[derive(Clone)]
pub struct FormState {
    pub field_mapping: IndexMap<String, NodeId>, // field name -> field node ID
    pub is_valid: bool,
    pub on_submit: Option<SubmitCallback>,
    pub submit_error: Option<String>,
}
