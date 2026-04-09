# Arthropod Architecture

This document describes the high-level architecture of the Arthropod framework, focusing on the interactions between the core crates, the reactive state model, and the hybrid ECS runtime.

## System Context

Arthropod is an enterprise-grade, cross-platform GUI framework written in Rust. It sits between the user and the underlying operating system and hardware.

```mermaid
C4Context
    title System Context Diagram for Arthropod Application

    Person(user, "User", "A user interacting with the application")
    System(app, "Arthropod App", "The GUI application built with Arthropod")
    System_Ext(os, "Operating System", "Provides Windowing, Input, and Accessibility APIs")
    System_Ext(gpu, "GPU", "Graphics Processing Unit")

    Rel(user, app, "Views and interacts with", "Mouse, Keyboard, Touch")
    Rel(app, os, "Uses", "Platform APIs (Win32/Cocoa/Wayland)")
    Rel(app, gpu, "Sends render commands to", "WGPU (Vulkan/Metal/DX12)")
```

## Container Architecture

The framework is composed of several specialized crates that separate concerns between platform abstraction, rendering, state management, and application logic.

```mermaid
C4Container
    title Container Diagram for Arthropod Framework

    Container(app, "Arthropod (App)", "Rust Crate", "Main entry point and orchestrator. Manages the integration of all subsystems.")
    Container(plat, "Plat Core", "Rust Crate", "Platform abstraction layer (Windowing, Event Loop).")
    Container(flux, "Flux State", "Rust Crate", "Reactive primitives (Signals, Effects).")
    Container(widget, "Widget Core", "Rust Crate", "High-level Widget traits and input handling.")
    Container(ecs, "Arthropod ECS", "Rust Crate", "ECS World, Components, and Systems.")
    Container(prototype, "Prototype Runtime", "Rust Module", "Deterministic runtime executor for imported prototype graphs")
    Container(render, "Render Engine", "Rust Crate", "WGPU rendering pipeline and Scene Graph.")

    Container(layout, "Layout Engine", "Rust Crate", "Flexbox/Grid layout calculations (Taffy wrapper).")
    Container(text, "Text Engine", "Rust Crate", "Font loading, shaping, and glyph caching.")
    Container(style, "Style Engine", "Rust Crate", "Vector paths, gradients, and styling primitives.")
    Container(theme, "Theme Engine", "Rust Crate", "Design tokens and system theme integration.")
    Container(anim, "Anim Graph", "Rust Crate", "Animation primitives (Springs, Tweens).")
    Container(a11y, "A11y Engine", "Rust Crate", "Accessibility tree and platform adapter.")
    Container(input, "Input Engine", "Rust Crate", "Input and gesture recognition (Spec 005).")
    Container(macros, "Widget Macros", "Rust Crate", "Procedural macros for declarative widget definition.")
    Container(mcp, "Arthropod MCP", "Rust Crate", "Model Context Protocol server for live debugging.")
    Container(test, "Arthropod Test", "Rust Crate", "Test harness and debugging tools.")
    Container(material, "Material UI", "Rust Crate", "Material Design 3 component library.")

    Rel(app, plat, "Uses", "Runs event loop")
    Rel(app, ecs, "Manages", "Updates systems")
    Rel(app, render, "Controls", "Initiates render")
    Rel(app, anim, "Integrates", "Runs animation tick")
    Rel(app, a11y, "Syncs", "Updates accessibility tree")
    Rel(app, input, "Integrates", "Propagates events")
    Rel(app, mcp, "Connects to", "TCP (Live Debugging)")

    Rel(test, app, "Wraps", "Headless testing")

    Rel(input, flux, "Uses", "Reactive state")
    Rel(input, plat, "Uses", "Raw events")

    Rel(widget, flux, "Uses", "Reactive state")
    Rel(widget, layout, "Uses", "Defines constraints")
    Rel(widget, theme, "Uses", "Resolves tokens")
    Rel(widget, text, "Uses", "Measures text")
    Rel(widget, macros, "Uses", "Expands macros")
    Rel(widget, input, "Delegates to", "Input processing")

    Rel(app, material, "Uses", "Material components")
    Rel(material, widget, "Builds on", "Widget traits")
    Rel(material, theme, "Uses", "Theme tokens")
    Rel(material, flux, "Uses", "Reactive state")

    Rel(render, text, "Uses", "Rasterizes glyphs")
    Rel(render, style, "Uses", "Tessellates paths")

    Rel(app, prototype, "Delegates to", "Event dispatch & navigation")
    Rel(ecs, flux, "Polls", "Reacts to signals")
    Rel(render, ecs, "Reads", "Scene data")
```

## Prototype Runtime Execution (Sequence)

The `PrototypeRuntime` orchestrates imported interactive prototype semantics, tracking states independently of the core widget hierarchy.

```mermaid
sequenceDiagram
    participant App
    participant PrototypeRuntime

    App->>PrototypeRuntime: event dispatch
    PrototypeRuntime->>PrototypeRuntime: timeout handling
    PrototypeRuntime->>PrototypeRuntime: navigation history
    PrototypeRuntime->>PrototypeRuntime: overlay stack
    PrototypeRuntime->>PrototypeRuntime: back semantics
    PrototypeRuntime->>App: URL effects
```

## Render Engine Architecture

The `render-engine` has been decomposed into specialized components to handle the complexity of multipass rendering and resource management. See [ADR 0026](./adr/0026-modular-wgpu-backend.md).

```mermaid
C4Component
    title Component Diagram for Render Engine

    Container_Boundary(render, "Render Engine") {
        Component(backend, "WgpuBackend", "Coordinator", "Orchestrates frame lifecycle and resources.")
        Component(multipass, "MultipassRenderer", "Worker", "Executes render passes and commands.")
        Component(collector, "InstanceCollector", "Worker", "Converts SceneNodes to instances.")
        Component(clipping, "Clipping", "Worker", "Manages stencil buffer.")
        Component(primitive, "PrimitivePipeline", "Worker", "Renders vector primitives.")

        Rel(backend, multipass, "Delegates to", "Rust")
        Rel(backend, collector, "Delegates to", "Rust")
        Rel(multipass, primitive, "Uses", "Rust")
        Rel(multipass, clipping, "Uses", "Rust")
    }
```

## Runtime Loop (Sequence)

The `WidgetApp` runtime loop orchestrates the flow of events from the OS to the application logic and back to the screen. This sequence diagram illustrates a typical frame update.

```mermaid
sequenceDiagram
    participant User
    participant Plat as Plat Core (OS)
    participant App as WidgetApp
    participant Dispatch as EventDispatcher
    participant Context as WidgetContext
    participant ECS as ECS World
    participant Render as WgpuBackend
    participant MCP as Arthropod MCP

    Note over User, Plat: Interaction Phase
    User->>Plat: Input Event (Click/Key)
    Plat->>App: on_event(event)
    App->>Dispatch: dispatch(event)
    Dispatch->>Context: Update Internal State

    alt State Changed (e.g., Focus/Text)
        Dispatch-->>App: Return DispatchResult
        App->>Plat: request_redraw()
    end

    Note over Plat, Render: Render Phase
    Plat->>App: on_redraw()

    rect rgb(240, 240, 240)
        note right of App: Update Systems
        App->>ECS: app.update()
        ECS->>ECS: Run Reactive Systems (Single-Pass)
        ECS->>ECS: Run Layout System
        ECS->>ECS: Run Animation System
    end

    opt Every 60 Frames (Debugging)
        App->>MCP: send_scene_update(json)
    end

    App->>Render: render(Scene)
    Render-->>User: Present Frame (GPU)
```

## Widget System Structure

The widget system follows a "Functional Core, Imperative Shell" approach. `WidgetContext` acts as a flattened state container, while complex logic is delegated to pure functional modules. See [ADR 0028](./adr/0028-widget-state-delegation.md) and [ADR 0037](./adr/0037-input-engine-unification.md).

```mermaid
classDiagram
    class Widget {
        <<trait>>
        +build(ctx: WidgetContext) NodeId
    }

    class WidgetContext {
        +Scene scene
        -IndexMap~NodeId, TextInputState~ text_input_states
        -HashMap~NodeId, FormState~ form_states
        -HashMap~NodeId, ValidationState~ validators
        +create_node()
        +focus_next()
    }

    class InputEngine {
        <<Namespace>>
        +text
        +form
        +focus
    }

    class TextInputState {
        +ReadSignal~String~ read
        +WriteSignal~String~ write
        +usize cursor_position
    }

    Widget ..> WidgetContext : Uses
    WidgetContext "1" *-- "*" TextInputState : Owns
    WidgetContext ..> InputEngine : Delegates to
```

## Key Architectural Decisions

- **Hybrid ECS**: We use a custom `Scene` graph (HashMap-based tree) for hierarchical operations (layout, event bubbling) while using `bevy_ecs` for bulk operations (rendering, animation). See [ADR 0001](./adr/0001-hybrid-ecs-architecture.md).
- **Reactive State**: State is managed via `flux-state` signals. We use `ReadSignal<T>` directly (which is thread-safe) to propagate changes from the UI to the ECS. See [ADR 0027](./adr/0027-reactive-signal-simplification.md) and [ADR 0024](./adr/0024-single-pass-reactive-updates.md).
- **Platform Abstraction**: `plat-core` isolates OS-specific code, allowing the rest of the engine to remain platform-agnostic.

## Experimental Subsystems (Nova)

The `nova` feature flag unlocks several experimental modules designed for advanced interactivity and debugging. These are guarded to keep the core lean. See [ADR 0029](./adr/0029-experimental-feature-strategy.md).

```mermaid
classDiagram
    namespace Arthropod {
        class App
        class Runtime
    }

    namespace Experimental {
        class StoryRuntime {
            +NarrativeGenerator
            +register_story()
        }
        class FluxRadar {
            +FluxRadarConfig
            +FluxRadarState
            +register_flux_radar()
        }
        class Chronos {
            +Timeline
            +RetroSignal
        }
        class Elastic {
            +ElasticSignal
            +ElasticRegistry
        }
        class Particles {
            +Particle
            +ParticleEmitter
            +ForceField
        }
        class XRay {
            +XRayConfig
            +XRayState
        }
        class Noise {
            +NoiseSignal
            +perlin_2d()
        }
        class SignalGraph {
            +SignalGraph
            +SignalHistory
        }
        class GhostReplay {
            +GhostRecorder
            +GhostReplayer
        }
        class ReactiveParticles {
            +ReactiveParticleEmitter
            +sync_reactive_emitters()
        }
        class MouseGestures {
            +StrokeMatcher
            +MouseGesture
        }
        class KineticText {
            +KineticText
            +TextAnimation
            +MotionSignal
        }
    }

    App ..> Experimental : "nova" feature enables
    FluxRadar ..> Runtime : Inspects Graph
    StoryRuntime ..> App : Modifies World
    Particles ..> App : Adds Systems
    Noise ..> Runtime : Creates Signals
    SignalGraph ..> Runtime : Consumes Signals
    GhostReplay ..> App : Intercepts Events
    ReactiveParticles ..> Particles : Extends
    MouseGestures ..> App : Intercepts Events
    KineticText ..> Runtime : Uses Signals
    KineticText ..> App : Adds Motion
```
