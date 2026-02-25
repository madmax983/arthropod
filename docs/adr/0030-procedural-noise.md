# 30. Procedural Noise Generation

Date: 2024-05-20

## Status

Accepted

## Context

Arthropod's visual language aims for "organic" and "fluid" interactions. Standard UI animations typically use easings (springs, cubic-bezier) to transition between discrete states. However, continuous effects—such as idle camera sway, particle turbulence, or "breathing" visual elements—require a source of randomness that exhibits spatial and temporal coherence.

Using standard pseudo-random number generators (PRNGs) like `rand::random()` produces "white noise," which changes abruptly and lacks continuity. This results in jittery, unnatural motion.

To achieve organic motion, we need **Gradient Noise** (specifically Perlin Noise). While external crates like `noise-rs` exist, they often:
1.  Introduce heavy dependency trees.
2.  Use complex trait hierarchies suited for terrain generation engines, not lightweight UI effects.
3.  Lack native integration with our `flux-state` reactive system.

## Decision

We will implement a lightweight, zero-dependency Perlin Noise generator within the `arthropod::experimental::noise` module.

The implementation will:
1.  **Be Self-Contained**: Include a static permutation table and the standard Perlin gradient/fade algorithms (using the improved 6t^5 - 15t^4 + 10t^3 fade function) without external math libraries.
2.  **Support 1D and 2D Noise**:
    *   `perlin_1d(x)`: For time-based variation (e.g., opacity fluctuation).
    *   `perlin_2d(x, y)`: For spatial variation (e.g., texture coordinates, 2D shake).
3.  **Provide Reactive Wrappers**:
    *   `NoiseSignal`: Wraps a `ReadSignal<f32>` (input) and returns a `Computed<f32>` (noise).
    *   `NoiseSignal2D`: Wraps a `ReadSignal<Vec2>` (input) and returns a `Computed<f32>` (noise).

This module will be guarded by the `nova` feature flag, as it is part of the experimental subsystem.

## Consequences

### Positive
*   **Organic Feel**: Enables continuous, non-repetitive animations that feel more "alive" than looped keyframes.
*   **Reactive Integration**: Because it outputs `flux-state` signals, noise values can directly drive style properties (color, offset, scale) without manual polling in a system.
*   **Zero Bloat**: No extra dependencies added to the build graph.

### Negative
*   **Maintenance**: We own the math implementation. Any bugs in the permutation table or interpolation logic must be fixed by us.
*   **Performance**: This is a CPU-based implementation. While fast enough for UI elements (hundreds of instances), it is not suitable for generating full-screen textures or terrain meshes in real-time (which should be done in shaders).
*   **Scope Limitation**: We are only implementing Perlin noise. Simplex noise (which has lower computational complexity in higher dimensions) is not included to keep the implementation simple.
