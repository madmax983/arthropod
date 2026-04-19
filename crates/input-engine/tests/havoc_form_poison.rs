use hashbrown::HashMap;
use indexmap::IndexMap;
use input_engine::InputNodeId;
use input_engine::form::{FormState, trigger_submit};
use std::sync::{Arc, Mutex};

#[test]
fn test_havoc_form_poison() {
    let mut form_states = HashMap::new();
    let text_input_states = IndexMap::new();
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
                let _guard = submit_called_clone.lock().unwrap();
                panic!("Die!");
            })),
            submit_error: None,
        },
    );

    let handle = std::thread::spawn({
        let mut form_states = form_states.clone();
        let text_input_states = text_input_states.clone();
        let mut validators = validators.clone();
        move || {
            trigger_submit(
                form_id,
                &mut form_states,
                &text_input_states,
                &mut validators,
            );
        }
    });

    let _ = handle.join();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        form_states.insert(
            form_id,
            FormState {
                field_mapping: IndexMap::new(),
                is_valid: true,
                on_submit: Some(Arc::new(move |_data| {
                    // Try to acquire the same lock. If poisoned, this panics or returns error.
                    let mut guard = submit_called
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    *guard = true;
                    Ok(())
                })),
                submit_error: None,
            },
        );

        trigger_submit(
            form_id,
            &mut form_states,
            &text_input_states,
            &mut validators,
        );
    }));

    assert!(
        result.is_ok(),
        "Form submission failed to recover from poisoned lock"
    );
}
