use crate::form_state::{FormData, FormState};
use crate::input_state::TextInputState;
use crate::validation_state::ValidationState;
use indexmap::IndexMap;
use render_engine::NodeId;
use std::collections::HashMap;

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
