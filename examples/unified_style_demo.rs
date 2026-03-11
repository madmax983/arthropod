//! Unified Style Demo - Automatic Hover/Active states via ECS
//!
//! This example demonstrates the new cohesive DX:
//! 1. Using the `style!` macro for declarative visuals and layout.
//! 2. Automatic hover/active states handled by the ECS.
//! 3. Zero manual event handling for visual feedback.
//!
//! Run with: cargo run --example unified_style_demo

use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    App::run("Unified Style Demo", 600, 400, |ctx| {
        // 1. Query system theme and set tokens on context
        // This enables widgets like Button to use themed hover/active states automatically
        let theme = SystemTheme::query().unwrap();
        let tokens = DesignTokens::from_system(&theme);
        ctx.set_design_tokens(tokens.clone());

        // 2. Define a card style (now without its own hover)
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
                txt!("Cohesive DX", size: 32.0),
                txt!("Notice how the buttons below have themed hover effects!", size: 18.0),
                
                Row::new((
                    btn!("Primary", primary),
                    btn!("Secondary", secondary),
                    // Demonstrate a custom hover override on a specific button
                    btn!("Custom Hover").style(style! {
                        background: tokens.accent.clone();
                        color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0);
                        padding: 12.0;
                        border_radius: 20.0; // Extra round

                        &:hover {
                            background: glam::Vec4::new(1.0, 0.0, 0.5, 1.0); // Hot pink hover
                            border_radius: 4.0; // Morph to square on hover
                        }
                    }),
                )).gap(12.0),

                txt!("The card hover was removed to focus on button interactions.", size: 14.0)
                    .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
            ))
            .style(card_style)
        )
    })
}
