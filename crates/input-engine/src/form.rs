//! Form state and logic.

use crate::text::TextInputState;
use crate::validation::ValidationState;
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

/// Helper to validate a single field
fn validate_field(
    field_node_id: NodeId,
    text_input_states: &IndexMap<NodeId, TextInputState>,
    validators: &mut HashMap<NodeId, ValidationState>,
) {
    let Some(state) = text_input_states.get(&field_node_id) else {
        return;
    };

    let Some(validator_state) = validators.get_mut(&field_node_id) else {
        return;
    };

    let value = state.read_signal.get_untracked();
    let result = (validator_state.validator)(&value);
    validator_state.error = result.err();
}

/// Helper to collect form data
fn collect_form_data(
    form_state: &FormState,
    text_input_states: &IndexMap<NodeId, TextInputState>,
) -> FormData {
    form_state
        .field_mapping
        .iter()
        .filter_map(|(name, id)| {
            let state = text_input_states.get(id)?;
            Some((name.clone(), state.read_signal.get_untracked()))
        })
        .collect()
}

/// Revalidate a form (check all field validators)
pub fn revalidate_form(
    node_id: NodeId,
    form_states: &mut HashMap<NodeId, FormState>,
    text_input_states: &IndexMap<NodeId, TextInputState>,
    validators: &mut HashMap<NodeId, ValidationState>,
) {
    // Re-run validators on all fields with current values
    if let Some(form_state) = form_states.get(&node_id) {
        for &field_node_id in form_state.field_mapping.values() {
            validate_field(field_node_id, text_input_states, validators);
        }
    }

    // Get field errors after re-validation
    let field_errors = get_form_field_errors(node_id, form_states, validators);

    // Update form is_valid state
    if let Some(form_state) = form_states.get_mut(&node_id) {
        form_state.is_valid = field_errors.is_empty();
    }
}

/// Get all field errors for a form
pub fn get_form_field_errors(
    node_id: NodeId,
    form_states: &HashMap<NodeId, FormState>,
    validators: &HashMap<NodeId, ValidationState>,
) -> HashMap<String, String> {
    let Some(form_state) = form_states.get(&node_id) else {
        return HashMap::new();
    };

    form_state
        .field_mapping
        .iter()
        .filter_map(|(name, id)| {
            // Flattened nested option handling
            let error = validators.get(id)?.error.as_ref()?;
            Some((name.clone(), error.clone()))
        })
        .collect()
}

/// Trigger form submission
pub fn trigger_submit(
    node_id: NodeId,
    form_states: &mut HashMap<NodeId, FormState>,
    text_input_states: &IndexMap<NodeId, TextInputState>,
    validators: &mut HashMap<NodeId, ValidationState>,
) {
    // Revalidate first
    revalidate_form(node_id, form_states, text_input_states, validators);

    let Some(form_state) = form_states.get(&node_id) else {
        return;
    };

    if !form_state.is_valid {
        return;
    }

    let Some(callback) = form_state.on_submit.clone() else {
        return;
    };

    // Collect form data
    let form_data = collect_form_data(form_state, text_input_states);

    // Call submit callback
    let result = callback(form_data);

    // Store submit error if any
    if let Some(form_state_mut) = form_states.get_mut(&node_id) {
        form_state_mut.submit_error = result.err();
    }
}
