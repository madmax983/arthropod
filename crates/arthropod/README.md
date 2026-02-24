# arthropod

The main entry point for the Arthropod GUI framework. This crate provides the high-level application runtime, event loop integration, and widget composition system.

## Overview

Arthropod is a retained-mode, reactive GUI framework built for performance and developer experience. It combines the ergonomics of modern declarative UI frameworks with the power of an Entity-Component-System (ECS) architecture.

### Key Components

- **`App`**: The application runtime manager. Handles window creation, event loop, and resource initialization.
- **`WidgetContext`**: The context passed to your UI builder closure. Used to create widgets and manage reactive state.
- **`flux-state`**: The reactive engine. `Signal`, `Computed`, and `Effect` primitives are re-exported for convenience.
- **`widget-core`**: The standard widget library. Macros like `col!`, `row!`, `txt!`, `btn!` are re-exported here.

## Quick Start

Add `arthropod` to your `Cargo.toml`:

```toml
[dependencies]
arthropod = "0.1"
```

Create a simple counter application:

```rust
use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    App::run("Arthropod Counter", 400, 300, |ctx| {
        // Create reactive state
        let count = ctx.signal(0);
        let (read, write) = count.split();

        // Create UI
        col!(
            [
                txt!("Counter Demo", size: 24.0),

                // Reactive text updates automatically
                Text::computed(Computed::new(ctx.runtime().clone(), move || {
                    format!("Count: {}", read.get())
                })),

                row!(
                    [
                        btn!("-", on_click: move || write.update(|c| *c -= 1)),
                        btn!("+", primary, on_click: move || write.update(|c| *c += 1)),
                    ],
                    gap: 10.0
                ),
            ],
            gap: 20.0,
            padding: 20.0,
            alignment: Alignment::Center
        )
    })
}
```

## Architecture

Arthropod uses a unique "Hybrid ECS" architecture:

1.  **Widget Phase**: Your UI code runs, building a transient widget tree.
2.  **Integration Phase**: The widget tree is reconciled into a persistent Scene Graph (`render-engine`) and ECS entities (`arthropod-ecs`).
3.  **Update Phase**: The ECS scheduler runs systems for layout, animation, input handling, and rendering.
4.  **Render Phase**: The Scene Graph is traversed and rendered via `wgpu`.

This separation allows for:
- **Zero-cost Abstractions**: Complex widgets compile down to simple scene nodes.
- **High Performance**: Layout and rendering are batched and optimized.
- **Extensibility**: You can hook into the ECS to add custom behaviors or systems.

## Experimental Features (Nova)

Advanced features like the Story Engine, Particle Systems, and Time Travel Debugging are available under the `nova` feature flag.

```toml
[dependencies]
arthropod = { version = "0.1", features = ["nova"] }
```

## License

MIT or Apache-2.0
