//! Unified Style Demo - Automatic Hover/Active states via ECS
//!
//! This example demonstrates the new cohesive DX:
//! 1. Using the `style!` macro for declarative visuals and layout.
//! 2. Automatic hover/active states handled by the ECS.
//! 3. Zero manual event handling for visual feedback.
//!
//! Run with: cargo run --example unified_style_demo

use arthropod::prelude::*;
use theme_engine::{style, DesignTokens, SystemTheme, FlexAlign};

fn main() -> Result<(), AppError> {
    App::run("Unified Style Demo", 600, 400, |_ctx| {
        // Query system theme for tokens
        let theme = SystemTheme::query().unwrap();
        let tokens = DesignTokens::from_system(&theme);

        // Define a complex style using the macro
        let card_style = style! {
            background: tokens.surface_secondary.clone();
            padding: 32.0;
            border_radius: tokens.radius_lg;
            gap: 24.0;
            align_items: FlexAlign::Center;

            &:hover {
                background: tokens.surface_elevated.clone();
            }
        };

        // Build the UI
        Center::new(
            Column::new((
                txt!("Cohesive DX", size: 32.0),
                txt!("This card and buttons use the unified style system.", size: 18.0),
                
                Row::new((
                    btn!("Primary", primary),
                    btn!("Secondary", secondary),
                    btn!("Custom Style", on_click: || println!("Custom clicked")),
                )).gap(12.0),

                txt!("Notice how hover effects work automatically!", size: 14.0)
                    .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
            ))
            .style(card_style)
        )
    })
}
