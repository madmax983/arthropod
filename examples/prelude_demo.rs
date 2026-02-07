//! Prelude Demo - Shows the benefit of using arthropod::prelude
//!
//! Compare the old way vs new way of importing types.
//!
//! **Run with:** cargo run --example prelude_demo

use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    App::run("Prelude Demo", 600, 500, |ctx| {
        let counter = ctx.signal(0);
        let (read, write) = counter.split();

        Column::new((
            // Header
            Column::new((
                Text::new("Arthropod Prelude").heading1(),
                Text::new("Zero ceremony imports").body(),
            ))
            .gap(8.0),
            Divider::horizontal().margin(20.0),
            // Before section
            Column::new((
                Text::new("Before: 30+ lines of imports").heading3(),
                Text::new("use std::time::{Duration, Instant};").caption(),
                Text::new("use plat_core::{Event, Window, ...};").caption(),
                Text::new("use render_engine::{Color, Scene, ...};").caption(),
                Text::new("use widget_core::{Button, Text, ...};").caption(),
                Text::new("// ... and many more!").caption(),
            ))
            .gap(4.0),
            Spacer::fixed(20.0),
            // After section
            Column::new((
                Text::new("After: Just one line!").heading3(),
                Text::new("use arthropod::prelude::*;")
                    .body()
                    .color(Color::rgba(0.0, 0.5, 0.9, 1.0)),
            ))
            .gap(8.0),
            Spacer::fixed(20.0),
            Divider::horizontal(),
            Spacer::fixed(20.0),
            // Interactive demo
            Column::new((
                Text::new("Try it out!").heading3(),
                Row::new((
                    Text::new(format!("Counter: {}", read.get())).body(),
                    Spacer::flex(),
                    Button::new("Increment").primary().on_click(move || {
                        let current = read.get();
                        write.set(current + 1);
                    }),
                ))
                .gap(12.0),
            ))
            .gap(12.0),
        ))
        .gap(8.0)
        .padding(40.0)
    })
}
