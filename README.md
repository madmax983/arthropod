# Arthropod 🦀

[![CI](https://github.com/madmax983/arthropod/workflows/CI/badge.svg)](https://github.com/madmax983/arthropod/actions/workflows/ci.yml)
[![Mutation Testing](https://github.com/madmax983/arthropod/workflows/Mutation%20Testing/badge.svg)](https://github.com/madmax983/arthropod/actions/workflows/mutation-testing.yml)
[![Security Audit](https://github.com/madmax983/arthropod/workflows/CI/badge.svg?job=audit)](https://github.com/madmax983/arthropod/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/madmax983/arthropod/branch/trunk/graph/badge.svg)](https://codecov.io/gh/madmax983/arthropod)

# ⚠️ IMPORTANT: EXPERIMENTAL FEATURES ⚠️

> [!CAUTION]
> **You are likely here because `NarrativeGenerator` is missing.**
>
> All experimental features (including **Nova Story Engine**, **Particles**, **Time Travel**) are hidden behind the `nova` feature flag.
>
> **TO FIX THIS:**
> 1. Add `features = ["nova"]` to your `Cargo.toml`.
> 2. Run examples with `--features nova`.
>
> ```bash
> cargo run --example story_demo --features nova
> ```

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

Add `arthropod` to your `Cargo.toml`.

> **Note**: `App::run` currently requires **Windows** or **macOS**. Linux support is planned. For Linux, please use `App::new_headless()` or run the TUI-based examples (like `story_demo` which requires the `nova` feature: `cargo run --example story_demo --features nova`).

```rust
use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    return App::run("Hello Arthropod", 400, 300, |_ctx| {
        col!(
            [
                txt!("Hello, World!", size: 24.0),
                btn!("Click Me", primary, on_click: || println!("Button clicked!")),
            ],
            gap: 20.0,
            padding: 20.0
        )
    });

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        println!("App::run requires Windows or macOS. Use App::new_headless() on Linux.");
        Ok(())
    }
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

**📚 Read the full documentation: [Nova Story Engine Guide](docs/experimental/nova-story.md)**

You can run the story demo to see it in action:

```bash
cargo run --example story_demo --features nova
```

> **Note:** If you run the demo *without* the `--features nova` flag, it will exit with a helpful error message explaining the requirement.

### Troubleshooting

If you encounter **deprecation warnings** or **unresolved imports** when using `arthropod` in your own code:

```text
warning: use of deprecated struct `arthropod::experimental::...`: Requires 'nova' feature.
error[E0432]: unresolved import `arthropod::experimental::...`
```

This indicates that you are trying to use an experimental feature (like `NarrativeGenerator`) but the `nova` feature flag is not enabled. Please update your `Cargo.toml` to include `features = ["nova"]`.

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

⚠️ **Windows Users**: cargo-mutants has a known bug on Windows (v26.2.0). Use WSL or check CI results instead.

```bash
# Linux/macOS/WSL
cargo install cargo-mutants
cargo mutants -p flux-state --no-shuffle --timeout 60

# Windows - use WSL or wait for CI
# Mutation testing runs automatically in GitHub Actions
```

**Mutation testing verifies that tests actually catch bugs** by introducing small code changes (mutants) and checking if tests fail. We maintain >80% mutation score.

See [`docs/testing/mutation-testing.md`](docs/testing/mutation-testing.md) for detailed guide and troubleshooting.

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

# Miri (unsafe code validation)
cargo +nightly miri test -p flux-state
```

### CI/CD Pipeline

Every PR runs:
- ✅ Format check (`cargo fmt`)
- ✅ Clippy lints (zero warnings)
- ✅ Full test suite (all platforms)
- ✅ Example builds
- ✅ Security audit
- ✅ Dependency validation
- ✅ Miri (undefined behavior detection)
- ✅ Unsafe code audit
- ✅ Mutation testing (weekly + on PR)
- ✅ Performance regression detection

See [`.github/workflows/`](.github/workflows/) for full CI configuration.

## License

MIT or Apache-2.0
