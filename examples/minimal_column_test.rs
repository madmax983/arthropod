//! Minimal Column Test - Simplest possible column layout
//!
//! **Run with:** cargo run --example minimal_column_test

use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    App::run("Minimal Column Test", 400, 300, |_ctx| {
        Column::new((
            Text::new("First Text")
                .size(24.0)
                .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
            Text::new("Second Text")
                .size(24.0)
                .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
            Text::new("Third Text")
                .size(24.0)
                .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
        ))
        .gap(20.0)
        .padding(30.0)
    })
}
