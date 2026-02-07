//! Prelude Demo - Shows the benefit of using arthropod::prelude
//!
//! Compare the old way vs new way of importing types.
//!
//! **Run with:** cargo run --example prelude_demo

use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    App::run("Prelude Demo", 700, 500, |ctx| {
        let counter = ctx.signal(0);
        let (read, write) = counter.split();

        Column::new((
            // Header section
            Card::new((Column::new((
                Text::new("Arthropod Prelude")
                    .size(32.0)
                    .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
                Text::new("Zero ceremony imports")
                    .size(16.0)
                    .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
            ))
            .gap(8.0),))
            .padding(24.0),
            // Before/After comparison
            Card::new((Column::new((
                Text::new("Before: 30+ lines of imports")
                    .size(14.0)
                    .color(Color::rgba(0.6, 0.2, 0.2, 1.0)),
                Text::new("use std::time::{Duration, Instant};")
                    .size(12.0)
                    .color(Color::rgba(0.4, 0.4, 0.4, 1.0)),
                Text::new("use plat_core::{Event, Window, ...};")
                    .size(12.0)
                    .color(Color::rgba(0.4, 0.4, 0.4, 1.0)),
                Text::new("use render_engine::{Color, Scene, ...};")
                    .size(12.0)
                    .color(Color::rgba(0.4, 0.4, 0.4, 1.0)),
                Text::new("use widget_core::{Button, Text, ...};")
                    .size(12.0)
                    .color(Color::rgba(0.4, 0.4, 0.4, 1.0)),
                Text::new("// ... and many more!")
                    .size(12.0)
                    .color(Color::rgba(0.4, 0.4, 0.4, 1.0)),
            ))
            .gap(4.0),))
            .padding(20.0),
            Card::new((Column::new((
                Text::new("After: Just one line!")
                    .size(14.0)
                    .color(Color::rgba(0.2, 0.6, 0.2, 1.0)),
                Text::new("use arthropod::prelude::*;")
                    .size(14.0)
                    .color(Color::rgba(0.0, 0.5, 0.8, 1.0)),
            ))
            .gap(8.0),))
            .padding(20.0),
            // Interactive demo
            Card::new((Column::new((
                Text::new("Try it out!")
                    .size(18.0)
                    .color(Color::rgba(0.2, 0.2, 0.2, 1.0)),
                Divider::horizontal().margin(12.0),
                Row::new((
                    Text::new(format!("Counter: {}", read.get()))
                        .size(16.0)
                        .color(Color::rgba(0.1, 0.1, 0.1, 1.0)),
                    Spacer::flex(),
                    Button::new("Increment")
                        .primary()
                        .on_click(move || {
                            let current = read.get();
                            write.set(current + 1);
                        }),
                ))
                .gap(12.0),
                Spacer::fixed(8.0),
                Text::new("All widgets available instantly!")
                    .size(12.0)
                    .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
            ))
            .gap(8.0),))
            .padding(20.0),
        ))
        .gap(16.0)
        .padding(24.0)
    })
}
