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
    Container(render, "Render Engine", "Rust Crate", "WGPU rendering pipeline and Scene Graph.")

    Container(layout, "Layout Engine", "Rust Crate", "Flexbox/Grid layout calculations (Taffy wrapper).")
    Container(text, "Text Engine", "Rust Crate", "Font loading, shaping, and glyph caching.")
    Container(style, "Style Engine", "Rust Crate", "Vector paths, gradients, and styling primitives.")
    Container(theme, "Theme Engine", "Rust Crate", "Design tokens and system theme integration.")
    Container(anim, "Anim Graph", "Rust Crate", "Animation primitives (Springs, Tweens).")
    Container(a11y, "A11y Engine", "Rust Crate", "Accessibility tree and platform adapter.")

    Rel(app, plat, "Uses", "Runs event loop")
    Rel(app, ecs, "Manages", "Updates systems")
    Rel(app, render, "Controls", "Initiates render")
    Rel(app, anim, "Integrates", "Runs animation tick")
    Rel(app, a11y, "Syncs", "Updates accessibility tree")

    Rel(widget, flux, "Uses", "Reactive state")
    Rel(widget, layout, "Uses", "Defines constraints")
    Rel(widget, theme, "Uses", "Resolves tokens")
    Rel(widget, text, "Uses", "Measures text")

    Rel(render, text, "Uses", "Rasterizes glyphs")
    Rel(render, style, "Uses", "Tessellates paths")

    Rel(ecs, flux, "Polls", "Reacts to signals")
    Rel(render, ecs, "Reads", "Scene data")
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
        ECS->>ECS: Run Reactive Systems (Apply Signals)
        ECS->>ECS: Run Layout System
        ECS->>ECS: Run Animation System
    end

    App->>Render: render(Scene)
    Render-->>User: Present Frame (GPU)
```

## Key Architectural Decisions

- **Hybrid ECS**: We use a custom `Scene` graph (HashMap-based tree) for hierarchical operations (layout, event bubbling) while using `bevy_ecs` for bulk operations (rendering, animation). See [ADR 0001](./adr/0001-hybrid-ecs-architecture.md).
- **Reactive State**: State is managed via `flux-state` signals. The ECS polls these signals to update components, ensuring UI properties remain in sync with the underlying data model.
- **Platform Abstraction**: `plat-core` isolates OS-specific code, allowing the rest of the engine to remain platform-agnostic.
