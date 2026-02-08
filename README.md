# Arthropod 🦀

[![CI](https://github.com/madmax983/arthropod/workflows/CI/badge.svg)](https://github.com/madmax983/arthropod/actions/workflows/ci.yml)
[![Mutation Testing](https://github.com/madmax983/arthropod/workflows/Mutation%20Testing/badge.svg)](https://github.com/madmax983/arthropod/actions/workflows/mutation-testing.yml)
[![Security Audit](https://github.com/madmax983/arthropod/workflows/CI/badge.svg?job=audit)](https://github.com/madmax983/arthropod/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/madmax983/arthropod/branch/trunk/graph/badge.svg)](https://codecov.io/gh/madmax983/arthropod)

> [!WARNING]
> **REQUIRES FEATURE NOVA**: Experimental features (like `NarrativeGenerator`) **must** have the `nova` feature flag enabled. See [Experimental Features](#experimental-features-nova) below.

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
| `arthropod` | The main framework crate. Provides `App` and integration logic. |
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

> **Note:** If you run the demo *without* the `--features nova` flag, it will exit with a helpful error message explaining the requirement.

### Troubleshooting

If you encounter the following compiler error when using `arthropod` in your own code:

```text
error[E0433]: failed to resolve: could not find `experimental` in `arthropod`
```

This indicates that you are trying to use an experimental feature (like `arthropod::experimental`) but the `nova` feature flag is not enabled. Please update your `Cargo.toml` to include `features = ["nova"]`.

## Development & Testing

Arthropod follows strict TDD principles with comprehensive CI/CD:

### Running Tests

```bash
# Run all tests
cargo test --all --all-features

# Run specific crate tests
cargo test -p arthropod-ecs

# Run benchmarks
cargo bench --all
```

### Mutation Testing

We use `cargo-mutants` to ensure test quality:

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation testing (PowerShell)
.\scripts\mutation-test.ps1

# Run on specific package
.\scripts\mutation-test.ps1 -Package flux-state
```

**Mutation testing verifies that tests actually catch bugs** by introducing small code changes (mutants) and checking if tests fail. We maintain >80% mutation score.

### Code Quality Checks

```bash
# Format check
cargo fmt --all -- --check

# Clippy lints (strict mode)
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo audit

# Dependency check
cargo deny check
```

### CI/CD Pipeline

Every PR runs:
- ✅ Format check (`cargo fmt`)
- ✅ Clippy lints (zero warnings)
- ✅ Full test suite (all platforms)
- ✅ Example builds
- ✅ Security audit
- ✅ Dependency validation
- ✅ Mutation testing (weekly + on PR)
- ✅ Performance regression detection

See [`.github/workflows/`](.github/workflows/) for full CI configuration.

## License

MIT or Apache-2.0
