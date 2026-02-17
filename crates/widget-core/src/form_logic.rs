use crate::form_state::{FormData, FormState};
use crate::input_state::TextInputState;
use crate::validation_state::ValidationState;
use indexmap::IndexMap;
use render_engine::NodeId;
use std::collections::HashMap;

/// Revalidate a form (check all field validators)
pub fn revalidate_form(
    node_id: NodeId,
    form_states: &mut HashMap<NodeId, FormState>,
    text_input_states: &IndexMap<NodeId, TextInputState>,
    validators: &mut HashMap<NodeId, ValidationState>,
) {
    // Re-run validators on all fields with current values
    let field_node_ids: Vec<NodeId> = if let Some(form_state) = form_states.get(&node_id) {
        form_state.field_mapping.values().copied().collect()
    } else {
        return;
    };

    // Collect current values first to avoid borrowing issues
    let mut field_values = HashMap::new();
    for field_node_id in &field_node_ids {
        if let Some(state) = text_input_states.get(field_node_id) {
            field_values.insert(*field_node_id, state.read_signal.get_untracked());
        }
    }

    // Re-run validation for each field
    for (field_node_id, current_value) in field_values {
        if let Some(validator_state) = validators.get_mut(&field_node_id) {
            // Run validator on current value
            let result = (validator_state.validator)(&current_value);
            validator_state.error = result.err();
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
    let mut errors = HashMap::new();

    if let Some(form_state) = form_states.get(&node_id) {
        for (field_name, field_node_id) in &form_state.field_mapping {
            // Check if node has validation error
            let error = validators
                .get(field_node_id)
                .and_then(|state| state.error.clone());

            if let Some(error) = error {
                errors.insert(field_name.clone(), error);
            }
        }
    }

    errors
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

    // Only submit if valid
    let is_valid = form_states
        .get(&node_id)
        .map(|state| state.is_valid)
        .unwrap_or(true);

    if !is_valid {
        return;
    }

    // Collect form data and callback
    let mut form_data = FormData::new();
    let mut callback_opt = None;

    if let Some(form_state) = form_states.get(&node_id) {
        callback_opt = form_state.on_submit.clone();

        for (field_name, field_node_id) in &form_state.field_mapping {
            if let Some(state) = text_input_states.get(field_node_id) {
                form_data.insert(field_name.clone(), state.read_signal.get_untracked());
            }
        }
    }

    // Call submit callback
    if let Some(callback) = callback_opt {
        let result = callback(form_data);

        // Store submit error if any
        if let Some(form_state_mut) = form_states.get_mut(&node_id) {
            form_state_mut.submit_error = result.err();
        }
    }
}
