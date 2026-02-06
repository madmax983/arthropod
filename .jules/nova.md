## [Confetti Cannon]
**Concept:** A lightweight ECS-based particle system (The "Confetti Cannon") that integrates with the Scene Graph to provide visual feedback (fireworks, rain, confetti) for UI interactions.
**Fate:** In Progress
**Lesson:** Visual flair is essential for "Juice" in UI, and ECS makes it easy to simulate thousands of particles efficiently.

## [MCP Inspector TUI]
**Concept:** A Ratatui-based dashboard to inspect and interact with the arthropod-mcp server without an AI agent. It acts as a manual MCP Client.
**Fate:** Merged
**Lesson:** Visualizing the invisible "brain" of the framework makes debugging much easier.

## [Motion Signals]
**Concept:** A bridge between `flux-state` and `anim-graph` that enables fully reactive animations. Turn any `Signal<T>` into a smooth `Spring` or `Tween` with a single function wrapper, driven by a reactive clock.
**Fate:** Merged
**Lesson:** Reactive state management needs a "Time" primitive to handle animations gracefully. Leaking effects (via RAII wrappers) is necessary for long-lived reactive processes that return values.
