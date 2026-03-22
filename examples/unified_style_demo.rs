//! Unified Style Demo - Automatic Hover/Active states via ECS
//!
//! This example demonstrates the new cohesive DX:
//! 1. Using the `style!` macro for declarative visuals and layout.
//! 2. Automatic hover/active states handled by the ECS.
//! 3. Reactive animations driven by the `anim-graph` integrated runner.
//!
//! Run with: cargo run --example unified_style_demo

use arthropod::prelude::*;
use std::time::Duration;

fn main() -> Result<(), AppError> {
    App::run("Unified Style Demo", 600, 500, |ctx| {
        // 1. Query system theme and set tokens on context
        let theme = SystemTheme::query().unwrap();
        let tokens = DesignTokens::from_system(&theme);
        ctx.set_design_tokens(tokens.clone());

        // 2. Create an animated progress signal using Timeline
        let progress_signal = ctx.signal(0.0_f32);
        let (read_progress, write_progress) = progress_signal.split();

        // A looping 5-second tween from 0→1, driven by the ECS timeline system.
        // Old approach: 33 lines of manual Arc<Mutex<Animation>>, frame signal Effect, manual reset.
        // New approach: one Timeline declaration.
        let timeline =
            arthropod::anim_graph::timeline::Timeline::tween(0.0_f32, 1.0, Duration::from_secs(5))
                .loop_forever();

        ctx.add_timeline_f32(timeline, write_progress);

        // 3. Define a card style
        let card_style = style! {
            background: tokens.surface_secondary.clone();
            padding: 32.0;
            border_radius: tokens.radius_lg;
            gap: 24.0;
            align_items: FlexAlign::Center;
        };

        // 4. Build the UI
        Center::new(
            Column::new((
                Column::new((
                    txt!("Animated Progress", size: 14.0),
                    progress_bar!(read_progress, height: 10.0),
                ))
                .gap(8.0),
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
                txt!(
                    "Everything is synchronized via the integrated anim-graph.",
                    size: 14.0
                )
                .color(Color(tokens.text_secondary)),
            ))
            .style(card_style),
        )
    })
}
