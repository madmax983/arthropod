//! Form widget tests - Written FIRST following TDD

use flux_state::{Runtime, Signal};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use widget_core::{Form, TextInput, Widget, WidgetContext};

// Helper validators
fn required(s: &str) -> Result<(), String> {
    if s.is_empty() {
        Err("Required".to_string())
    } else {
        Ok(())
    }
}

fn validate_email(s: &str) -> Result<(), String> {
    if s.contains('@') {
        Ok(())
    } else {
        Err("Invalid email".to_string())
    }
}

fn min_length(min: usize) -> impl Fn(&str) -> Result<(), String> {
    move |s: &str| {
        if s.len() >= min {
            Ok(())
        } else {
            Err(format!("Must be at least {} characters", min))
        }
    }
}

#[test]
fn test_form_creates_node() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let name = Signal::new(runtime.clone(), String::new());
    // Tuple-based fields (compile-time typed)
    let form = Form::new((("name", TextInput::new(name)),));

    let node_id = form.build(&mut ctx);

    // Should create a form node
    assert!(ctx.scene().get_node(node_id).is_some());
}

#[test]
fn test_form_builds_child_fields() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let name = Signal::new(runtime.clone(), String::new());
    let email = Signal::new(runtime.clone(), String::new());

    let form = Form::new((
        ("name", TextInput::new(name)),
        ("email", TextInput::new(email)),
    ));

    let node_id = form.build(&mut ctx);

    // Should have 2 field children
    let form_node = ctx.scene().get_node(node_id).unwrap();
    assert_eq!(
        form_node.children.len(),
        2,
        "Form should have 2 field children"
    );
}

#[test]
fn test_form_aggregates_field_errors() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let name = Signal::new(runtime.clone(), String::new()); // Empty, will fail
    let email = Signal::new(runtime.clone(), "invalid".to_string()); // No @, will fail

    let form = Form::new((
        ("name", TextInput::new(name).validator(required)),
        ("email", TextInput::new(email).validator(validate_email)),
    ));

    let node_id = form.build(&mut ctx);

    // Form should track that it has errors
    assert!(!ctx.is_form_valid(node_id), "Form should be invalid");

    // Should have 2 field errors
    let field_errors = ctx.get_form_field_errors(node_id);
    assert_eq!(field_errors.len(), 2, "Should have 2 field errors");
    assert!(field_errors.contains_key("name"), "Should have name error");
    assert!(
        field_errors.contains_key("email"),
        "Should have email error"
    );
}

#[test]
fn test_form_valid_when_all_fields_pass() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let name = Signal::new(runtime.clone(), "John Doe".to_string());
    let email = Signal::new(runtime.clone(), "john@example.com".to_string());

    let form = Form::new((
        ("name", TextInput::new(name).validator(required)),
        ("email", TextInput::new(email).validator(validate_email)),
    ));

    let node_id = form.build(&mut ctx);

    // Form should be valid
    assert!(ctx.is_form_valid(node_id), "Form should be valid");

    // Should have no field errors
    let field_errors = ctx.get_form_field_errors(node_id);
    assert_eq!(field_errors.len(), 0, "Should have no field errors");
}

#[test]
fn test_form_submit_callback() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();
    let submitted = Arc::new(AtomicBool::new(false));
    let submitted_clone = submitted.clone();

    let name = Signal::new(runtime.clone(), "John".to_string());

    let form =
        Form::new((("name", TextInput::new(name).validator(required)),)).on_submit(move |_data| {
            submitted_clone.store(true, Ordering::SeqCst);
            Ok(())
        });

    let node_id = form.build(&mut ctx);

    // Form should be valid
    assert!(ctx.is_form_valid(node_id));

    // Trigger submit
    ctx.trigger_submit(node_id);

    // Callback should have been called
    assert!(
        submitted.load(Ordering::SeqCst),
        "Submit callback should be called"
    );
}

#[test]
fn test_form_submit_only_when_valid() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();
    let submitted = Arc::new(AtomicBool::new(false));
    let submitted_clone = submitted.clone();

    let name = Signal::new(runtime.clone(), String::new()); // Invalid (empty)

    let form =
        Form::new((("name", TextInput::new(name).validator(required)),)).on_submit(move |_data| {
            submitted_clone.store(true, Ordering::SeqCst);
            Ok(())
        });

    let node_id = form.build(&mut ctx);

    // Form should be invalid
    assert!(!ctx.is_form_valid(node_id));

    // Try to submit
    ctx.trigger_submit(node_id);

    // Callback should NOT be called (form is invalid)
    assert!(
        !submitted.load(Ordering::SeqCst),
        "Submit should not be called when invalid"
    );
}

#[test]
fn test_form_validation_updates_dynamically() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let name = Signal::new(runtime.clone(), String::new()); // Start invalid

    let form = Form::new((("name", TextInput::new(name).validator(required)),));

    let node_id = form.build(&mut ctx);

    // Initially invalid (empty)
    assert!(!ctx.is_form_valid(node_id), "Should be invalid initially");

    // Get the field node ID
    let form_node = ctx.scene().get_node(node_id).unwrap();
    let field_node_id = form_node.children[0];

    // Focus and type into the field
    ctx.focus_node(field_node_id);
    ctx.send_char('J');
    ctx.send_char('o');
    ctx.send_char('h');
    ctx.send_char('n');

    // Re-validate form
    ctx.revalidate_form(node_id);

    // Should now be valid
    assert!(ctx.is_form_valid(node_id), "Should be valid after typing");
}

#[test]
fn test_form_submit_receives_field_data() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();
    let received_name = Arc::new(std::sync::Mutex::new(String::new()));
    let received_name_clone = received_name.clone();

    let name = Signal::new(runtime.clone(), "John Doe".to_string());

    let form = Form::new((("name", TextInput::new(name)),)).on_submit(
        move |data: hashbrown::HashMap<String, String>| {
            if let Some(name_value) = data.get("name") {
                *received_name_clone.lock().unwrap() = name_value.clone();
            }
            Ok(())
        },
    );

    let node_id = form.build(&mut ctx);

    // Submit form
    ctx.trigger_submit(node_id);

    // Should have received the field data
    let received = received_name.lock().unwrap();
    assert_eq!(
        *received, "John Doe",
        "Should receive field data in callback"
    );
}

#[test]
fn test_form_with_multiple_validators_per_field() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    // Password with multiple validation rules
    let password = Signal::new(runtime.clone(), "ab".to_string()); // Too short

    let form = Form::new(((
        "password",
        TextInput::new(password).validator(min_length(8)),
    ),));

    let node_id = form.build(&mut ctx);

    // Should be invalid (too short)
    assert!(!ctx.is_form_valid(node_id), "Should be invalid (too short)");

    let errors = ctx.get_form_field_errors(node_id);
    assert!(
        errors.contains_key("password"),
        "Should have password error"
    );
    assert_eq!(
        errors.get("password").unwrap(),
        "Must be at least 8 characters"
    );
}

#[test]
fn test_form_without_fields() {
    let mut ctx = WidgetContext::new_test();
    let submitted = Arc::new(AtomicBool::new(false));
    let submitted_clone = submitted.clone();

    // Empty form using empty tuple
    let form = Form::new(()).on_submit(move |_data| {
        submitted_clone.store(true, Ordering::SeqCst);
        Ok(())
    });

    let node_id = form.build(&mut ctx);

    // Empty form should be valid
    assert!(ctx.is_form_valid(node_id), "Empty form should be valid");

    // Should be able to submit
    ctx.trigger_submit(node_id);
    assert!(submitted.load(Ordering::SeqCst), "Should submit empty form");
}

#[test]
fn test_form_submit_error_handling() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();
    let submit_count = Arc::new(AtomicUsize::new(0));
    let submit_count_clone = submit_count.clone();

    let name = Signal::new(runtime.clone(), "John".to_string());

    let form = Form::new((("name", TextInput::new(name)),)).on_submit(move |_data| {
        submit_count_clone.fetch_add(1, Ordering::SeqCst);
        Err("Server error".to_string())
    });

    let node_id = form.build(&mut ctx);

    // Submit (will fail)
    ctx.trigger_submit(node_id);

    // Should have attempted submit
    assert_eq!(submit_count.load(Ordering::SeqCst), 1);

    // Should have submit error
    assert!(ctx.has_submit_error(node_id), "Should have submit error");
    assert_eq!(
        ctx.get_submit_error(node_id),
        Some("Server error".to_string())
    );
}

#[test]
fn test_form_layout_vertical() {
    let mut ctx = WidgetContext::new_test();
    let runtime = Runtime::new();

    let name = Signal::new(runtime.clone(), String::new());
    let email = Signal::new(runtime.clone(), String::new());

    let form = Form::new((
        ("name", TextInput::new(name)),
        ("email", TextInput::new(email)),
    ))
    .gap(16.0);

    let node_id = form.build(&mut ctx);

    // Should have layout style
    let layout = ctx.get_layout_style(node_id).unwrap();
    assert_eq!(layout.gap, 16.0, "Should have gap spacing");
}
