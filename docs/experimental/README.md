# Experimental Features (Nova)

> **Status:** Experimental
> **Feature Flag:** `nova`

Arthropod includes several experimental features under the `nova` codename. These features are unstable and subject to breaking changes.

To use any of these features, you must enable the `nova` feature in your `Cargo.toml`:

```toml
[dependencies]
arthropod = { version = "0.1", features = ["nova"] }
```

## Available Features

| Feature | Description | Status | Documentation |
|---------|-------------|--------|---------------|
| **Nova Story Engine** | A reactive narrative runtime for choice-driven stories. | Beta | [Docs](nova-story.md) |
| **Particles** | GPU-accelerated particle system. | Alpha | [Docs](particles.md) |
| **Time Travel (Ghost Replay)** | Record and replay input events for debugging and automation. | Alpha | [Docs](time-travel.md) |
| **Reactive Particles** | Particles that react to signal changes. | Alpha | *Coming Soon* |
| **Elastic** | Physics-based animation primitives. | Alpha | *Coming Soon* |
| **X-Ray** | Debugging tool for visualizing scene structure. | Prototype | *Coming Soon* |
| **Chronos** | Time manipulation primitives. | Prototype | *Coming Soon* |
| **Signal Graph** | Visual graph editor for signal connections. | Prototype | *Coming Soon* |
| **Noise** | Procedural noise generation. | Alpha | *Coming Soon* |

## Usage Warning

These features are guarded by the `nova` feature flag. If you try to use them without enabling the flag, you will encounter compilation warnings and runtime panics with instructions to enable the feature.

Always wrap your experimental code in `#[cfg(feature = "nova")]` if you are writing examples or optional modules.
