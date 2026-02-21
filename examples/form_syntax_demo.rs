//! Form Syntax Demo - Shows new declarative syntax for forms
//!
//! Run with: cargo run --example form_syntax_demo

use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    App::run("Form Syntax Demo", 400, 300, |ctx| {
        let name = ctx.signal(String::new());
        let email = ctx.signal(String::new());

        // Old syntax: form!([ ("name", widget) ])
        // New syntax: form!([ "name": widget ])

        col!(
            [
                txt!("Form Syntax Demo", size: 24.0),

                form!(
                    [
                        "Name": TextInput::new(name).placeholder("Enter name"),
                        "Email": TextInput::new(email).placeholder("Enter email"),
                    ],
                    gap: 16.0,
                    padding: 20.0,
                    on_submit: |data| {
                        println!("Submitted: {:?}", data);
                        Ok(())
                    }
                )
            ],
            gap: 20.0,
            padding: 20.0
        )
    })
}
