//! Unified Style Demo - Automatic Hover/Active states via ECS
//!
//! This example demonstrates the new cohesive DX:
//! 1. Using the `style!` macro for declarative visuals and layout.
//! 2. Automatic hover/active states handled by the ECS.
//! 3. Zero manual event handling for visual feedback.
//!
//! Run with: cargo run --example unified_style_demo

use arthropod::prelude::*;
use std::time::Instant;

fn main() -> Result<(), AppError> {
    App::run("Unified Style Demo", 600, 500, |ctx| {
        // 1. Query system theme and set tokens on context
        let theme = SystemTheme::query().unwrap();
        let tokens = DesignTokens::from_system(&theme);
        ctx.set_design_tokens(tokens.clone());

        // Create an animated progress signal
        let progress_signal = ctx.signal(0.0_f32);
        let (read_progress, write_progress) = progress_signal.split();

        let start_time = Instant::now();
        ctx.store_effect(Effect::new(ctx.runtime().clone(), move || {
            let elapsed = start_time.elapsed().as_secs_f32();
            let p = (elapsed * 0.5) % 1.0;
            write_progress.set(p);
        }));

        // 2. Define a card style
        let card_style = style! {
            background: tokens.surface_secondary.clone();
            padding: 32.0;
            border_radius: tokens.radius_lg;
            gap: 24.0;
            align_items: FlexAlign::Center;
        };

        // 3. Build the UI
        Center::new(
            Column::new((
                Row::new((
                    icon!("\u{e88a}", size: 32.0, color: tokens.accent), // Home
                    txt!("Cohesive DX", size: 32.0),
                ))
                .gap(12.0),
                txt!(
                    "Notice how the buttons below have themed hover effects!",
                    size: 18.0
                ),
                Row::new((
                    btn!("Primary", primary),
                    btn!("Secondary", secondary),
                    btn!("Custom Hover").style(style! {
                        background: tokens.accent;
                        color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0);
                        padding: widget_core::style::Padding::symmetric(6.0, 12.0);
                        border_radius: 20.0;

                        &:hover {
                            background: glam::Vec4::new(1.0, 0.0, 0.5, 1.0);
                            border_radius: 4.0;
                        }
                    }),
                ))
                .gap(12.0),
                // New primitive: ProgressBar
                Column::new((
                    txt!("Animated Progress", size: 14.0),
                    progress_bar!(read_progress, height: 10.0),
                ))
                .gap(8.0),
                txt!(
                    "The card hover was removed to focus on button interactions.",
                    size: 14.0
                )
                .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
            ))
            .style(card_style),
        )
    })
}
