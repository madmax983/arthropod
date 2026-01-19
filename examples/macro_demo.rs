//! Widget Macro DSL Demo - Form Example
//!
//! This example demonstrates the same functionality as form_demo.rs,
//! but using the declarative widget macros:
//! - `form!` - Form widget with named fields
//! - `txt!` - Text widget
//! - `btn!` - Button widget
//! - `col!` / `row!` - Layout containers
//!
//! Run with: cargo run --example macro_demo

use flux_state::{Runtime, Signal};
use widget_core::{TextInput, Widget, WidgetContext, btn, col, form, row, txt};

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
    println!("=== Arthropod Form Demo (with Macros) ===\n");

    // Create runtime for reactive state
    let runtime = Runtime::new();

    // Create signals for form fields
    let name = Signal::new(runtime.clone(), String::new());
    let email = Signal::new(runtime.clone(), String::new());
    let password = Signal::new(runtime.clone(), String::new());

    println!("Creating form with 3 fields using macros:");
    println!("  - Name (required)");
    println!("  - Email (required, must be valid email)");
    println!("  - Password (required, minimum 8 characters)\n");

    // Create form using macros!
    // The form! macro accepts a tuple of (name, widget) pairs
    let user_form = form!(
        [
            (
                "name",
                TextInput::new(name)
                    .placeholder("Enter your name")
                    .validator(required)
            ),
            (
                "email",
                TextInput::new(email)
                    .placeholder("your.email@example.com")
                    .validator(|s| {
                        required(s)?;
                        validate_email(s)
                    })
            ),
            (
                "password",
                TextInput::new(password)
                    .placeholder("Password (min 8 chars)")
                    .validator(|s| {
                        required(s)?;
                        min_length(8)(s)
                    })
            ),
        ],
        gap: 16.0,
        padding: 20.0,
        on_submit: |data| {
            println!("\n✅ Form submitted successfully!");
            println!("Data received:");
            for (field, value) in data {
                println!("  {}: {}", field, value);
            }
            Ok(())
        }
    );

    // Build the form
    let mut ctx = WidgetContext::new_test();
    let form_node = user_form.build(&mut ctx);

    println!("✓ Form built successfully!");
    println!("  Form node ID: {:?}", form_node);

    // Check initial validation state
    let is_valid = ctx.is_form_valid(form_node);
    println!(
        "\nInitial validation state: {}",
        if is_valid { "VALID ✓" } else { "INVALID ✗" }
    );

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
    println!(
        "  Value: {}",
        ctx.get_text_input_value(field_nodes[0]).unwrap()
    );

    // Fill in email field
    println!("Filling email field...");
    ctx.focus_node(field_nodes[1]);
    for c in "john.doe@example.com".chars() {
        ctx.send_char(c);
    }
    println!(
        "  Value: {}",
        ctx.get_text_input_value(field_nodes[1]).unwrap()
    );

    // Fill in password field
    println!("Filling password field...");
    ctx.focus_node(field_nodes[2]);
    for c in "secure_password_123".chars() {
        ctx.send_char(c);
    }
    println!(
        "  Value: {}",
        ctx.get_text_input_value(field_nodes[2]).unwrap()
    );

    // Revalidate form
    println!("\nRevalidating form...");
    ctx.revalidate_form(form_node);

    let is_valid = ctx.is_form_valid(form_node);
    println!(
        "Validation state after input: {}",
        if is_valid { "VALID ✓" } else { "INVALID ✗" }
    );

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
            println!(
                "❌ Submit error: {}",
                ctx.get_submit_error(form_node).unwrap()
            );
        }
    } else {
        println!("\n❌ Cannot submit: Form is invalid");
    }

    // Test validation with invalid data
    println!("\n--- Testing validation with invalid data ---\n");

    let runtime2 = Runtime::new();
    let invalid_email = Signal::new(runtime2.clone(), "not-an-email".to_string());

    // Create form with macro
    let invalid_form = form!(
        [("email", TextInput::new(invalid_email).validator(validate_email))],
        on_submit: |_| {
            println!("This should not be called!");
            Ok(())
        }
    );

    let mut ctx2 = WidgetContext::new_test();
    let invalid_form_node = invalid_form.build(&mut ctx2);

    println!("Created form with invalid email: 'not-an-email'");
    let is_valid = ctx2.is_form_valid(invalid_form_node);
    println!(
        "Validation state: {}",
        if is_valid { "VALID ✓" } else { "INVALID ✗" }
    );

    if !is_valid {
        let errors = ctx2.get_form_field_errors(invalid_form_node);
        for (_field, error) in errors {
            println!("  Error: {}", error);
        }
    }

    println!("\nAttempting to submit invalid form...");
    ctx2.trigger_submit(invalid_form_node);
    println!("Submit was blocked (form is invalid) ✓");

    // ========================
    // Bonus: Show other macros
    // ========================
    println!("\n--- Bonus: Other Widget Macros ---\n");

    // Text macros
    let _title = txt!("Form Demo", size: 24.0);
    println!("txt!(\"Form Demo\", size: 24.0) - styled text");

    // Button macros
    let _submit_btn = btn!("Submit", primary, on_click: || println!("Submitted!"));
    println!("btn!(\"Submit\", primary, on_click: ...) - primary button");

    // Layout composition
    let _layout = col!(
        [
            txt!("Registration Form", size: 20.0),
            row!([btn!("Submit", primary), btn!("Cancel", secondary),], gap: 8.0),
        ],
        gap: 16.0,
        padding: 20.0
    );
    println!("col!([txt!(...), row!([btn!(...), btn!(...)], ...)], ...) - nested layout");

    println!("\n=== Form Demo Complete ===");
    println!("\nKey features demonstrated:");
    println!("  ✓ form! macro with named fields");
    println!("  ✓ TextInput with validators");
    println!("  ✓ Dynamic validation (revalidate after input)");
    println!("  ✓ Submit-only-when-valid enforcement");
    println!("  ✓ Submit callback with form data");
    println!("  ✓ txt!, btn!, col!, row! macros for composition");
}
