//! Form GUI Example - Registration form with validation
//!
//! Demonstrates the simplified App::run() API:
//! - Create signals for reactive state
//! - Build form with validators
//! - Framework handles everything else
//!
//! Run with: cargo run --example form_gui

use arthropod::prelude::*;
use widget_core::{Form, TextInput};

fn main() -> Result<(), AppError> {
    App::run("Registration Form", 500, 400, |ctx| {
        // Create signals for form fields
        let name = ctx.signal(String::new());
        let email = ctx.signal(String::new());
        let password = ctx.signal(String::new());

        // Build form with validators
        Form::new((
            ("name", TextInput::new(name).placeholder("Name")),
            (
                "email",
                TextInput::new(email)
                    .placeholder("email@example.com")
                    .validator(|s| {
                        if s.contains('@') && s.contains('.') {
                            Ok(())
                        } else {
                            Err("Invalid email".into())
                        }
                    }),
            ),
            (
                "password",
                TextInput::new(password)
                    .placeholder("Password (8+ chars)")
                    .validator(|s| {
                        if s.len() >= 8 {
                            Ok(())
                        } else {
                            Err("Password too short".into())
                        }
                    }),
            ),
        ))
        .gap(16.0)
        .padding(20.0)
        .on_submit(|data| {
            println!("\n✅ Form submitted successfully!");
            println!("Data:");
            for (field, value) in data {
                println!("  {}: {}", field, value);
            }
            Ok(())
        })
    })
}
