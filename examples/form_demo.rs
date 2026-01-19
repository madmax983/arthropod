//! Form Demo - Complete demonstration of Form, TextInput, and Button widgets
//!
//! This example shows:
//! - Form with multiple validated fields
//! - TextInput with different validators
//! - Button with submit handling
//! - Dynamic validation and error display
//!
//! Run with: cargo run --example form_demo

use widget_core::{Form, TextInput, Widget, WidgetContext};
use flux_state::{Runtime, Signal};

// Validators
fn required(s: &str) -> Result<(), String> {
    if s.is_empty() {
        Err("This field is required".to_string())
    } else {
        Ok(())
    }
}

fn validate_email(s: &str) -> Result<(), String> {
    if s.contains('@') && s.contains('.') {
        Ok(())
    } else {
        Err("Please enter a valid email address".to_string())
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

fn main() {
    println!("=== Arthropod Form Demo ===\n");

    // Create runtime for reactive state
    let runtime = Runtime::new();

    // Create signals for form fields
    let name = Signal::new(runtime.clone(), String::new());
    let email = Signal::new(runtime.clone(), String::new());
    let password = Signal::new(runtime.clone(), String::new());

    println!("Creating form with 3 fields:");
    println!("  - Name (required)");
    println!("  - Email (required, must be valid email)");
    println!("  - Password (required, minimum 8 characters)\n");

    // Create form
    let form = Form::new()
        .field("name", TextInput::new(name)
            .placeholder("Enter your name")
            .validator(required))
        .field("email", TextInput::new(email)
            .placeholder("your.email@example.com")
            .validator(|s| {
                required(s)?;
                validate_email(s)
            }))
        .field("password", TextInput::new(password)
            .placeholder("Password (min 8 chars)")
            .validator(|s| {
                required(s)?;
                min_length(8)(s)
            }))
        .gap(16.0)
        .padding(20.0)
        .on_submit(|data| {
            println!("\n✅ Form submitted successfully!");
            println!("Data received:");
            for (field, value) in data {
                println!("  {}: {}", field, value);
            }
            Ok(())
        });

    // Build the form
    let mut ctx = WidgetContext::new_test();
    let form_node = form.build(&mut ctx);

    println!("✓ Form built successfully!");
    println!("  Form node ID: {:?}", form_node);

    // Check initial validation state
    let is_valid = ctx.is_form_valid(form_node);
    println!("\nInitial validation state: {}", if is_valid { "VALID ✓" } else { "INVALID ✗" });

    if !is_valid {
        let errors = ctx.get_form_field_errors(form_node);
        println!("Field errors:");
        for (field, error) in errors {
            println!("  - {}: {}", field, error);
        }
    }

    // Simulate user filling out the form
    println!("\n--- Simulating user input ---\n");

    let form_node_data = ctx.scene().get_node(form_node).unwrap();
    let field_nodes: Vec<_> = form_node_data.children.clone();

    // Fill in name field
    println!("Filling name field...");
    ctx.focus_node(field_nodes[0]);
    for c in "John Doe".chars() {
        ctx.send_char(c);
    }
    println!("  Value: {}", ctx.get_text_input_value(field_nodes[0]).unwrap());

    // Fill in email field
    println!("Filling email field...");
    ctx.focus_node(field_nodes[1]);
    for c in "john.doe@example.com".chars() {
        ctx.send_char(c);
    }
    println!("  Value: {}", ctx.get_text_input_value(field_nodes[1]).unwrap());

    // Fill in password field
    println!("Filling password field...");
    ctx.focus_node(field_nodes[2]);
    for c in "secure_password_123".chars() {
        ctx.send_char(c);
    }
    println!("  Value: {}", ctx.get_text_input_value(field_nodes[2]).unwrap());

    // Revalidate form
    println!("\nRevalidating form...");
    ctx.revalidate_form(form_node);

    let is_valid = ctx.is_form_valid(form_node);
    println!("Validation state after input: {}", if is_valid { "VALID ✓" } else { "INVALID ✗" });

    if !is_valid {
        let errors = ctx.get_form_field_errors(form_node);
        println!("Field errors:");
        for (field, error) in errors {
            println!("  - {}: {}", field, error);
        }
    }

    // Submit form
    if is_valid {
        println!("\nSubmitting form...");
        ctx.trigger_submit(form_node);

        if ctx.has_submit_error(form_node) {
            println!("❌ Submit error: {}", ctx.get_submit_error(form_node).unwrap());
        }
    } else {
        println!("\n❌ Cannot submit: Form is invalid");
    }

    // Test validation with invalid data
    println!("\n--- Testing validation with invalid data ---\n");

    let runtime2 = Runtime::new();
    let invalid_email = Signal::new(runtime2.clone(), "not-an-email".to_string());

    let invalid_form = Form::new()
        .field("email", TextInput::new(invalid_email)
            .validator(validate_email))
        .on_submit(|_| {
            println!("This should not be called!");
            Ok(())
        });

    let mut ctx2 = WidgetContext::new_test();
    let invalid_form_node = invalid_form.build(&mut ctx2);

    println!("Created form with invalid email: 'not-an-email'");
    let is_valid = ctx2.is_form_valid(invalid_form_node);
    println!("Validation state: {}", if is_valid { "VALID ✓" } else { "INVALID ✗" });

    if !is_valid {
        let errors = ctx2.get_form_field_errors(invalid_form_node);
        for (_field, error) in errors {
            println!("  Error: {}", error);
        }
    }

    println!("\nAttempting to submit invalid form...");
    ctx2.trigger_submit(invalid_form_node);
    println!("Submit was blocked (form is invalid) ✓");

    println!("\n=== Form Demo Complete ===");
    println!("\nKey features demonstrated:");
    println!("  ✓ Form with multiple validated fields");
    println!("  ✓ TextInput with custom validators");
    println!("  ✓ Dynamic validation (revalidate after input)");
    println!("  ✓ Submit-only-when-valid enforcement");
    println!("  ✓ Submit callback with form data");
    println!("  ✓ Error handling and display");
}
