//! Form state and logic.

use crate::InputNodeId;
use crate::text::TextInputState;
use crate::validation::ValidationState;
use hashbrown::HashMap;
use indexmap::IndexMap;
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
    pub field_mapping: IndexMap<String, InputNodeId>,

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
    field_node_id: InputNodeId,
    text_input_states: &IndexMap<InputNodeId, TextInputState>,
    validators: &mut HashMap<InputNodeId, ValidationState>,
) -> bool {
    let Some(state) = text_input_states.get(&field_node_id) else {
        return true;
    };

    let Some(validator_state) = validators.get_mut(&field_node_id) else {
        return true;
    };

    let value = state.read_signal.get_untracked();
    let result = (validator_state.validator)(&value);
    validator_state.error = result.err();

    validator_state.error.is_none()
}

/// Helper to collect form data
fn collect_form_data(
    form_state: &FormState,
    text_input_states: &IndexMap<InputNodeId, TextInputState>,
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
    node_id: InputNodeId,
    form_states: &mut HashMap<InputNodeId, FormState>,
    text_input_states: &IndexMap<InputNodeId, TextInputState>,
    validators: &mut HashMap<InputNodeId, ValidationState>,
) {
    // Re-run validators on all fields with current values and compute overall validity
    if let Some(form_state) = form_states.get_mut(&node_id) {
        let mut is_valid = true;
        for &field_node_id in form_state.field_mapping.values() {
            if !validate_field(field_node_id, text_input_states, validators) {
                is_valid = false;
            }
        }
        form_state.is_valid = is_valid;
    }
}

/// Get all field errors for a form
pub fn get_form_field_errors(
    node_id: InputNodeId,
    form_states: &HashMap<InputNodeId, FormState>,
    validators: &HashMap<InputNodeId, ValidationState>,
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
    node_id: InputNodeId,
    form_states: &mut HashMap<InputNodeId, FormState>,
    text_input_states: &IndexMap<InputNodeId, TextInputState>,
    validators: &mut HashMap<InputNodeId, ValidationState>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};
    use std::sync::Mutex;

    fn create_text_state(initial: &str) -> TextInputState {
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, initial.to_string());
        let (read, write) = signal.split();
        TextInputState {
            read_signal: read,
            write_signal: write,
            cursor_position: initial.chars().count(),
            readonly: false,
            max_length: None,
        }
    }

    #[test]
    fn should_validate_fields_and_update_form_state() {
        let mut form_states = HashMap::new();
        let mut text_input_states = IndexMap::new();
        let mut validators = HashMap::new();

        let form_id = InputNodeId(10);
        let field_id = InputNodeId(11);

        let mut field_mapping = IndexMap::new();
        field_mapping.insert("username".to_string(), field_id);

        form_states.insert(
            form_id,
            FormState {
                field_mapping,
                is_valid: true, // Initially true, should become false
                on_submit: None,
                submit_error: None,
            },
        );

        text_input_states.insert(field_id, create_text_state("short"));

        // Validator: Must be > 5 chars
        validators.insert(
            field_id,
            ValidationState {
                validator: Arc::new(|val| {
                    if val.len() > 5 {
                        Ok(())
                    } else {
                        Err("Too short".to_string())
                    }
                }),
                error: None,
            },
        );

        revalidate_form(
            form_id,
            &mut form_states,
            &text_input_states,
            &mut validators,
        );

        assert!(!form_states.get(&form_id).unwrap().is_valid);
        let errors = get_form_field_errors(form_id, &form_states, &validators);
        assert_eq!(errors.get("username"), Some(&"Too short".to_string()));

        // Make it valid
        text_input_states
            .get(&field_id)
            .unwrap()
            .write_signal
            .set("longenough".to_string());

        revalidate_form(
            form_id,
            &mut form_states,
            &text_input_states,
            &mut validators,
        );
        assert!(form_states.get(&form_id).unwrap().is_valid);
        let errors = get_form_field_errors(form_id, &form_states, &validators);
        assert!(errors.is_empty());
    }

    #[test]
    fn should_collect_form_data_correctly() {
        let field1_id = InputNodeId(11);
        let field2_id = InputNodeId(12);

        let mut field_mapping = IndexMap::new();
        field_mapping.insert("username".to_string(), field1_id);
        field_mapping.insert("email".to_string(), field2_id);

        let form_state = FormState {
            field_mapping,
            is_valid: true,
            on_submit: None,
            submit_error: None,
        };

        let mut text_input_states = IndexMap::new();
        text_input_states.insert(field1_id, create_text_state("alice"));
        text_input_states.insert(field2_id, create_text_state("alice@example.com"));

        let data = collect_form_data(&form_state, &text_input_states);
        assert_eq!(data.len(), 2);
        assert_eq!(data.get("username"), Some(&"alice".to_string()));
        assert_eq!(data.get("email"), Some(&"alice@example.com".to_string()));
    }

    #[test]
    fn should_trigger_submit_only_when_valid() {
        let mut form_states = HashMap::new();
        let mut text_input_states = IndexMap::new();
        let mut validators = HashMap::new();

        let form_id = InputNodeId(10);
        let field_id = InputNodeId(11);

        let mut field_mapping = IndexMap::new();
        field_mapping.insert("code".to_string(), field_id);

        let submit_called = Arc::new(Mutex::new(false));
        let submit_called_clone = submit_called.clone();

        form_states.insert(
            form_id,
            FormState {
                field_mapping,
                is_valid: true,
                on_submit: Some(Arc::new(move |_data| {
                    *submit_called_clone
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
                    Err("Server error".to_string())
                })),
                submit_error: None,
            },
        );

        text_input_states.insert(field_id, create_text_state("invalid_code"));

        // Validator fails for "invalid_code"
        validators.insert(
            field_id,
            ValidationState {
                validator: Arc::new(|val| {
                    if val == "1234" {
                        Ok(())
                    } else {
                        Err("Bad code".to_string())
                    }
                }),
                error: None,
            },
        );

        // Try submit - it should revalidate and fail before calling submit callback
        trigger_submit(
            form_id,
            &mut form_states,
            &text_input_states,
            &mut validators,
        );

        assert!(
            !(*submit_called
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner))
        );
        assert!(!form_states.get(&form_id).unwrap().is_valid);
        assert_eq!(form_states.get(&form_id).unwrap().submit_error, None);

        // Fix the input
        text_input_states
            .get(&field_id)
            .unwrap()
            .write_signal
            .set("1234".to_string());

        // Try submit again
        trigger_submit(
            form_id,
            &mut form_states,
            &text_input_states,
            &mut validators,
        );

        assert!(
            *submit_called
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        );
        assert!(form_states.get(&form_id).unwrap().is_valid);
        assert_eq!(
            form_states.get(&form_id).unwrap().submit_error,
            Some("Server error".to_string())
        );
    }
}
