use crate::form::{FormData, SubmitCallback};
use crate::validation::Validator;
use render_engine::NodeId;
use std::collections::HashMap;

/// Validation state for a node
pub struct ValidationState {
    pub validator: Validator,
    pub error: Option<String>,
}

/// Form state for tracking form fields and validation
pub struct FormState {
    pub field_mapping: HashMap<String, NodeId>, // field name -> field node ID
    pub is_valid: bool,
    pub on_submit: Option<SubmitCallback>,
    pub submit_error: Option<String>,
}

pub struct FormContext {
    pub validators: HashMap<NodeId, ValidationState>,
    pub form_states: HashMap<NodeId, FormState>,
}

impl FormContext {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            form_states: HashMap::new(),
        }
    }

    pub fn set_validator(
        &mut self,
        node_id: NodeId,
        validator: Validator,
        initial_result: Result<(), String>,
    ) {
        self.validators.insert(
            node_id,
            ValidationState {
                validator,
                error: initial_result.err(),
            },
        );
    }

    pub fn has_validation_error(&self, node_id: NodeId) -> bool {
        self.validators
            .get(&node_id)
            .and_then(|state| state.error.as_ref())
            .is_some()
    }

    pub fn get_validation_error(&self, node_id: NodeId) -> Option<String> {
        self.validators
            .get(&node_id)
            .and_then(|state| state.error.clone())
    }

    pub fn add_form_state(
        &mut self,
        node_id: NodeId,
        field_mapping: HashMap<String, NodeId>,
        on_submit: Option<SubmitCallback>,
    ) {
        self.form_states.insert(
            node_id,
            FormState {
                field_mapping,
                is_valid: true, // Will be updated by revalidate_form
                on_submit,
                submit_error: None,
            },
        );
    }

    pub fn is_form_valid(&self, node_id: NodeId) -> bool {
        self.form_states
            .get(&node_id)
            .map(|state| state.is_valid)
            .unwrap_or(true)
    }

    pub fn get_form_field_errors(&self, node_id: NodeId) -> HashMap<String, String> {
        let mut errors = HashMap::new();

        if let Some(form_state) = self.form_states.get(&node_id) {
            for (field_name, field_node_id) in &form_state.field_mapping {
                if let Some(error) = self.get_validation_error(*field_node_id) {
                    errors.insert(field_name.clone(), error);
                }
            }
        }

        errors
    }

    pub fn get_form_state(&self, node_id: NodeId) -> Option<&FormState> {
        self.form_states.get(&node_id)
    }

    pub fn revalidate_form<F>(&mut self, node_id: NodeId, value_provider: F)
    where
        F: Fn(NodeId) -> Option<String>,
    {
        // Re-run validators on all fields with current values
        let field_node_ids: Vec<NodeId> = if let Some(form_state) = self.form_states.get(&node_id) {
            form_state.field_mapping.values().copied().collect()
        } else {
            return;
        };

        // Collect current values first to avoid borrowing issues
        let mut field_values = HashMap::new();
        for field_node_id in &field_node_ids {
            if let Some(current_value) = value_provider(*field_node_id) {
                field_values.insert(*field_node_id, current_value);
            }
        }

        // Re-run validation for each field
        for (field_node_id, current_value) in field_values {
            if let Some(validator_state) = self.validators.get_mut(&field_node_id) {
                // Run validator on current value
                let result = (validator_state.validator)(&current_value);
                validator_state.error = result.err();
            }
        }

        // Get field errors after re-validation
        let field_errors = self.get_form_field_errors(node_id);

        // Update form is_valid state
        if let Some(form_state) = self.form_states.get_mut(&node_id) {
            form_state.is_valid = field_errors.is_empty();
        }
    }

    pub fn trigger_submit<F>(&mut self, node_id: NodeId, value_provider: F)
    where
        F: Fn(NodeId) -> Option<String>,
    {
        // Revalidate first
        self.revalidate_form(node_id, &value_provider);

        // Only submit if valid
        if !self.is_form_valid(node_id) {
            return;
        }

        // Collect form data
        let mut form_data = FormData::new();

        if let Some(form_state) = self.form_states.get(&node_id) {
            for (field_name, field_node_id) in &form_state.field_mapping {
                if let Some(value) = value_provider(*field_node_id) {
                    form_data.insert(field_name.clone(), value);
                }
            }

            // Call submit callback
            if let Some(callback) = &form_state.on_submit {
                let result = callback(form_data);

                // Store submit error if any
                if let Some(form_state_mut) = self.form_states.get_mut(&node_id) {
                    form_state_mut.submit_error = result.err();
                }
            }
        }
    }

    pub fn has_submit_error(&self, node_id: NodeId) -> bool {
        self.form_states
            .get(&node_id)
            .and_then(|state| state.submit_error.as_ref())
            .is_some()
    }

    pub fn get_submit_error(&self, node_id: NodeId) -> Option<String> {
        self.form_states
            .get(&node_id)
            .and_then(|state| state.submit_error.clone())
    }
}
