//! Prelude Demo - Shows the benefit of using arthropod::prelude
//!
//! Compare the old way vs new way of importing types.
//!
//! **Run with:** cargo run --example prelude_demo

// NEW WAY: Just one import!
use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    App::run("Prelude Demo", 600, 400, |ctx| {
        // All these types available with just one import:
        // - Duration, Instant (std::time)
        // - App, AppContext, AppError (app)
        // - Color, NodeContent, Scene, etc. (render-engine)
        // - Button, Text, Column, Row, etc. (widget-core)
        // - Signal, Runtime, Effect (flux-state)
        // - DesignTokens, SystemTheme (theme-engine)
        // - Event, WindowEvent, Key, MouseButton, etc. (plat-core)
        // - FrameworkContext, Renderable, etc. (arthropod-ecs)

        let counter = ctx.signal(0);
        let (read, write) = counter.split();

        Column::new((
            Text::new("Prelude Demo").heading1(),
            Divider::horizontal().margin(20.0),
            Text::new("All commonly used types available with:"),
            Text::new("use arthropod::prelude::*;")
                .size(14.0)
                .color(Color::rgba(0.0, 0.5, 0.8, 1.0)),
            Spacer::fixed(20.0),
            Text::new(format!("Counter: {}", read.get())),
            Button::new("Increment")
                .primary()
                .on_click(move || {
                    let current = read.get();
                    write.set(current + 1);
                }),
            Spacer::fixed(20.0),
            Text::new("No more ceremony!")
                .caption()
                .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
        ))
        .gap(12.0)
        .padding(40.0)
    })
}

/* OLD WAY (commented out to show what we DON'T need anymore):

use std::time::{Duration, Instant};
use plat_core::{
    Application, BackdropMaterial, ControlFlow, ElementState, Event, EventLoop,
    HasBackdropMaterial, Key, MouseButton, Rect, Size, Window, WindowConfig,
    WindowEvent, WindowId,
};
use render_engine::{
    Color, NodeContent, NodeId, Scene, SceneNode,
    backend::{RenderBackend, WgpuBackend},
};
use theme_engine::{DesignTokens, SystemTheme};
use widget_core::{Button, Text, Column, Row, Divider, Spacer};
use flux_state::{Signal, Runtime, Effect};
use arthropod_ecs::{FrameworkContext, Renderable};
// ... and more!

*/
