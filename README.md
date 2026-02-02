# Arthropod 🦀

> Enterprise-grade cross-platform Rust GUI framework.

> [!NOTE]
> **REQUIRES FEATURE NOVA**: Some experimental features (like `NarrativeGenerator`) require enabling the `nova` feature flag. See [Experimental Features](#experimental-features-nova) below.

Arthropod is a high-performance, retained-mode GUI framework built for Rust. It combines a fine-grained reactive state management system with a modular ECS architecture and GPU-accelerated rendering to deliver scalable and performant user interfaces.

## Features

- **⚡ Fine-Grained Reactivity**: Built on `flux-state`, inspired by SolidJS signals. Updates are surgical—only the changed nodes re-render.
- **🏗️ Modular Architecture**: Powered by `bevy_ecs`. Everything is a resource or a component, making the system highly extensible.
- **🚀 GPU Acceleration**: Rendering is handled by `wgpu`, ensuring smooth performance across platforms (Windows, macOS). Linux support is planned.
- **📝 Declarative Widgets**: Use Rust macros (`txt!`, `btn!`, `col!`) to build UI layouts with ease.

## Architecture

Arthropod separates concerns into three distinct layers:

1.  **Widget Layer (`widget-core`)**: The high-level API where you define your UI. It uses the Facade pattern to hide complexity.
2.  **Scene Graph (`render-engine`)**: A retained-mode tree structure that manages layout and rendering primitives.
3.  **ECS Runtime (`arthropod-ecs`)**: The backbone that orchestrates the application lifecycle, handling events, updates, and resource management.

## Quick Start

Add `arthropod` and `widget-core` to your `Cargo.toml`.

```rust
use arthropod::prelude::*;
use widget_core::{txt, btn, col};

fn main() -> Result<(), AppError> {
    App::run("Hello Arthropod", 400, 300, |_ctx| {
        col!(
            [
                txt!("Hello, World!", size: 24.0),
                btn!("Click Me", primary, on_click: || println!("Button clicked!")),
            ],
            gap: 20.0,
            padding: 20.0
        )
    })
}
```

## Crates Overview

| Crate | Description |
|-------|-------------|
| `arthropod` | The main framework crate. Provides `App`, `AppBuilder`, and integration logic. |
| `flux-state` | The reactive runtime. Implements Signals, Effects, and Computed values. |
| `render-engine` | The rendering backend. Manages the Scene Graph and `wgpu` integration. |
| `widget-core` | The widget library. Contains standard widgets like `Text`, `Button`, `TextInput`. |
| `plat-core` | Platform abstraction layer. Handles window creation and event loops. |

## Experimental Features (Nova)

Arthropod includes experimental features under the `nova` codename, such as the `NarrativeGenerator` and particle systems. These features are unstable and not enabled by default.

To access these features, you must enable the `nova` feature flag:

```toml
[dependencies]
arthropod = { version = "0.1", features = ["nova"] }
```

You can run the story demo to see it in action:

```bash
cargo run --example story_demo --features nova
```

## License

MIT or Apache-2.0
