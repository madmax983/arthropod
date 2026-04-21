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

## [Input Fusion]
**Concept:** A reactive gesture recognition system (Sequences, Chords) that bridges `plat-core` input events with `flux-state`.
**Fate:** Merged
**Lesson:** Reactive signals are perfect for state machines like gesture detection. `Effect` ownership is critical; dropping it kills the listener.

## [Fluid Layout]
**Concept:** Physics-based layout interpolation that transforms discrete `layout-engine` updates into smooth, organic animations using `flux-state` and `anim-graph`.
**Fate:** Merged
**Lesson:** Layouts shouldn't snap. By treating layout rects as animatable signals, we can make the entire UI feel "fluid" without changing the core layout logic.
## [Carousel]
**Concept:** A reusable horizontal layout widget to page through a list of children. Exposes the active index via `flux-state` signals, allowing external control and observation.
**Fate:** Merged
**Lesson:** Even without robust conditional rendering or `display: none` in the layout engine, we can construct interactive sliders using `flux-state` signals to keep state, and users can track it.

## Spotlight Overlay
**Concept:** Added a `Spotlight` feature connecting `flux-state` reactivity with `render-engine` scene graph. A fullscreen overlay is drawn, dimming everything *except* a target node. Useful for tutorials and onboarding.
**Fate:** Merged.
**Lesson:** `VectorPath` combined with the `EvenOdd` winding rule is a clean way to punch a hole in a solid rectangle without complex masking logic.
## 2024-03-18 - Parallax UI Effect
**Concept:** Added a `ParallaxNode` component and `update_parallax` system to shift UI elements slightly based on `MousePosition` and `LayoutConstraintsResource` to create an Apple TV style depth effect.
**Fate:** Merged.
**Lesson:** Simple affine transformations combined with existing ECS resources (`MousePosition`, `LayoutConstraintsResource`) can easily create complex, dynamic visual effects without touching core widget rendering logic.
## [Glass Overlay]
**Concept:** Added an experimental `glass_overlay` system connecting `flux-state` reactivity with `render-engine` blur effects (`Effect::BackgroundBlur`). It provides a fullscreen or node-specific "frosted glass" visual layer that dims and blurs elements underneath it.
**Fate:** Merged.
**Lesson:** Applying `Effect::BackgroundBlur` directly onto a new overlay node cleanly achieves the visual frosted glass effect over arbitrary content beneath it without modifying existing nodes.
## [Chaos Monkey Simulator]
**Concept:** Added a `chaos_monkey` system connecting `flux-state` reactivity with `arthropod-ecs`'s `Clickable` components. It automatically finds visible clickables and triggers them pseudo-randomly to simulate user interactions and fuzz UI state changes.
**Fate:** Merged.
**Lesson:** Isolating simulation logic inside an ECS system allows us to fuzz UI state without having to simulate raw input events at the OS level, making it extremely lightweight and deterministic.
## [Draggable UI Nodes]
**Concept:** A new ECS component `DraggableNode` that leverages the `InteractionState` and `MousePosition` resources to let users drag UI elements around the screen with their pointer. This introduces physical interactions to arbitrary widgets via the ECS.
**Fate:** Merged
**Lesson:** Connecting ECS input polling directly with physical node transforms creates a seamless, responsive feeling without blocking the main event dispatch loops.
