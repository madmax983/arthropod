use crate::form_state::FormState;
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
    input_engine::form::revalidate_form(node_id, form_states, text_input_states, validators);
}

/// Get all field errors for a form
pub fn get_form_field_errors(
    node_id: NodeId,
    form_states: &HashMap<NodeId, FormState>,
    validators: &HashMap<NodeId, ValidationState>,
) -> HashMap<String, String> {
    input_engine::form::get_form_field_errors(node_id, form_states, validators)
}

/// Trigger form submission
pub fn trigger_submit(
    node_id: NodeId,
    form_states: &mut HashMap<NodeId, FormState>,
    text_input_states: &IndexMap<NodeId, TextInputState>,
    validators: &mut HashMap<NodeId, ValidationState>,
) {
    input_engine::form::trigger_submit(node_id, form_states, text_input_states, validators);
}
