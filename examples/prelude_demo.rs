//! Prelude Demo - Shows the benefit of using arthropod::prelude
//!
//! **Run with:** cargo run --example prelude_demo

use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    App::run("Prelude Demo", 500, 400, |ctx| {
        let counter = ctx.signal(0);
        let (read, write) = counter.split();

        // Clone signal for button
        let read_for_button = read.clone();

        // Create a COMPUTED value that derives text from counter
        // Computed automatically updates when counter changes!
        let counter_text = Computed::new(ctx.runtime().clone(), move || {
            format!("Count: {}", read.get())
        });

        Column::new((
            Text::new("Arthropod Prelude")
                .size(24.0)
                .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
            Text::new("Zero ceremony imports")
                .size(14.0)
                .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
            Spacer::fixed(20.0),
            Text::new("Before: Many imports")
                .size(16.0)
                .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
            Text::new("use std::time::...; use plat_core::...; ...")
                .size(12.0)
                .color(Color::rgba(0.6, 0.6, 0.6, 1.0)),
            Spacer::fixed(16.0),
            Text::new("After: One import!")
                .size(16.0)
                .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
            Text::new("use arthropod::prelude::*;")
                .size(14.0)
                .color(Color::rgba(0.0, 0.5, 0.9, 1.0)),
            Spacer::fixed(20.0),
            Row::new((
                Text::computed(counter_text)
                    .size(14.0)
                    .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
                Spacer::flex(),
                Button::new("Click").primary().on_click(move || {
                    let new_count = read_for_button.get() + 1;
                    write.set(new_count);
                }),
            ))
            .gap(12.0),
        ))
        .gap(8.0)
        .padding(30.0)
    })
}
