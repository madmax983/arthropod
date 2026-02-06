# Arthropod Dependency Graph

This document shows the internal dependency relationships between Arthropod crates.

## Diagram

```mermaid
flowchart TB
    subgraph "Application Layer"
        arthropod[arthropod<br/><i>Framework API</i>]
    end

    subgraph "Tools Layer"
        arthropod-mcp[arthropod-mcp<br/><i>MCP Server</i>]
        arthropod-test[arthropod-test<br/><i>Testing Utils</i>]
    end

    subgraph "Widget Layer"
        widget-core[widget-core<br/><i>Widget System</i>]
        widget-macros[widget-macros<br/><i>Proc Macros</i>]
    end

    subgraph "Integration Layer"
        arthropod-ecs[arthropod-ecs<br/><i>ECS Integration</i>]
    end

    subgraph "Core Systems Layer"
        render-engine[render-engine<br/><i>Scene + GPU</i>]
        a11y-engine[a11y-engine<br/><i>Accessibility</i>]
        anim-graph[anim-graph<br/><i>Animation</i>]
    end

    subgraph "Foundation Layer"
        plat-core[plat-core<br/><i>Platform Abstraction</i>]
        flux-state[flux-state<br/><i>Reactive State</i>]
        layout-engine[layout-engine<br/><i>Flexbox Layout</i>]
        text-engine[text-engine<br/><i>Text Shaping</i>]
        theme-engine[theme-engine<br/><i>Design Tokens</i>]
    end

    %% arthropod dependencies
    arthropod --> plat-core
    arthropod --> render-engine
    arthropod --> anim-graph
    arthropod --> flux-state
    arthropod --> arthropod-ecs
    arthropod --> a11y-engine
    arthropod --> layout-engine
    arthropod --> widget-core

    %% widget-core dependencies
    widget-core --> render-engine
    widget-core --> text-engine
    widget-core --> layout-engine
    widget-core --> flux-state
    widget-core --> widget-macros
    widget-core --> theme-engine

    %% theme-engine dependencies
    theme-engine --> plat-core

    %% arthropod-ecs dependencies
    arthropod-ecs --> render-engine
    arthropod-ecs --> flux-state
    arthropod-ecs --> a11y-engine
    arthropod-ecs --> layout-engine
    arthropod-ecs --> widget-core
    arthropod-ecs --> plat-core

    %% Core systems dependencies
    render-engine --> plat-core
    render-engine --> text-engine
    a11y-engine --> plat-core
    a11y-engine --> render-engine
    anim-graph --> render-engine

    %% Tools dependencies
    arthropod-mcp --> arthropod-ecs
    arthropod-mcp --> render-engine
    arthropod-mcp --> flux-state
    arthropod-mcp --> plat-core
    arthropod-mcp --> arthropod-test
    arthropod-test --> render-engine
    arthropod-test --> plat-core

    %% Styling
    classDef foundation fill:#e8f5e9,stroke:#4caf50,stroke-width:2px
    classDef core fill:#e3f2fd,stroke:#2196f3,stroke-width:2px
    classDef integration fill:#fff3e0,stroke:#ff9800,stroke-width:2px
    classDef widget fill:#f3e5f5,stroke:#9c27b0,stroke-width:2px
    classDef tools fill:#fce4ec,stroke:#e91e63,stroke-width:2px
    classDef app fill:#ffebee,stroke:#f44336,stroke-width:2px

    class plat-core,flux-state,layout-engine,text-engine,theme-engine foundation
    class render-engine,a11y-engine,anim-graph core
    class arthropod-ecs integration
    class widget-core,widget-macros widget
    class arthropod-mcp,arthropod-test tools
    class arthropod app
```

## Layer Descriptions

| Layer | Crates | Purpose |
|-------|--------|---------|
| **Application** | `arthropod` | High-level framework API for end users |
| **Tools** | `arthropod-mcp`, `arthropod-test` | Developer tooling (MCP server, testing utilities) |
| **Widget** | `widget-core`, `widget-macros` | Widget system and DSL macros |
| **Integration** | `arthropod-ecs` | Bridges ECS with scene graph and reactive state |
| **Core Systems** | `render-engine`, `a11y-engine`, `anim-graph` | GPU rendering, accessibility, animation |
| **Foundation** | `plat-core`, `flux-state`, `layout-engine`, `text-engine`, `theme-engine` | Zero internal dependencies, standalone subsystems |

## Key Observations

1. **Foundation crates have zero internal dependencies** - Makes them easy to test and potentially reusable independently.

2. **`render-engine` is the most depended-upon crate** - 8 other crates depend on it. Changes here ripple widely.

3. **`widget-core` has the widest fan-out** - Pulls from 7 internal crates, integrating nearly every subsystem.

4. **Clear layering** - Dependencies only flow downward.

## Dependency Counts

| Crate | Depends On | Depended By |
|-------|------------|-------------|
| `plat-core` | 0 | 8 |
| `flux-state` | 0 | 4 |
| `layout-engine` | 0 | 3 |
| `text-engine` | 0 | 2 |
| `theme-engine` | 1 | 1 |
| `widget-macros` | 0 | 1 |
| `render-engine` | 2 | 8 |
| `a11y-engine` | 2 | 2 |
| `anim-graph` | 1 | 1 |
| `arthropod-ecs` | 6 | 2 |
| `widget-core` | 6 | 2 |
| `arthropod-test` | 2 | 1 |
| `arthropod-mcp` | 5 | 0 |
| `arthropod` | 8 | 0 |
