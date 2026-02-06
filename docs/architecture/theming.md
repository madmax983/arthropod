# Theming Architecture

This document details the architecture of the `theme-engine` and its integration with `plat-core` and the widget system.

## Overview

The theming system follows a layered architecture, updated to reflect the simplification in [ADR 0019](../adr/0019-theme-engine-simplification.md).

## Class Structure

The core of the system is the `DesignTokens` struct, which resolves abstract semantic roles (like "Primary Surface") into concrete platform-specific values (like "Mica Material" on Windows, or a specific Color on Linux).

```mermaid
classDiagram
    class SystemTheme {
        <<plat-core>>
        +Color accent_color
        +Color text_color
        +Vec~BackdropMaterial~ available_materials
        +bool is_dark_mode
        +query() SystemTheme
    }

    class DesignTokens {
        <<theme-engine>>
        +TokenValue surface_primary
        +TokenValue surface_secondary
        +TokenValue surface_elevated
        +Color text_primary
        +Color accent
        +f32 space_md
        +f32 radius_lg
        +from_system(theme: SystemTheme) DesignTokens
    }

    class TokenValue {
        <<Enumeration>>
        Color(Color)
        Material(BackdropMaterial)
        +as_color() Color
    }

    class BackdropMaterial {
        <<plat-core>>
        <<Enumeration>>
        None
        Mica
        MicaAlt
        Acrylic
    }

    class Color {
        <<glam::Vec4>>
    }

    DesignTokens --> TokenValue : contains
    TokenValue --> BackdropMaterial : wraps
    TokenValue --> Color : wraps
    DesignTokens ..> SystemTheme : creates from
    SystemTheme --> BackdropMaterial : references
```

## Style Resolution Flow

This diagram illustrates how a semantic request for "Primary Surface" is resolved to a platform-native effect or a fallback color.

```mermaid
sequenceDiagram
    participant App
    participant DesignTokens
    participant SystemTheme
    participant PlatCore
    participant Window

    App->>PlatCore: SystemTheme::query()
    activate PlatCore
    PlatCore->>PlatCore: Check OS capabilities
    PlatCore-->>App: SystemTheme (e.g. { materials: [Mica], dark: true })
    deactivate PlatCore

    App->>DesignTokens: from_system(theme)
    activate DesignTokens
    Note right of DesignTokens: Logic selects best material<br/>based on availability
    DesignTokens-->>App: DesignTokens
    deactivate DesignTokens

    Note over App: App building UI...

    App->>DesignTokens: tokens.surface_primary
    DesignTokens-->>App: TokenValue::Material(Mica)

    App->>Window: set_backdrop(Mica)
    Window->>PlatCore: set_backdrop_material(Mica)
    PlatCore->>PlatCore: DwmSetWindowAttribute(...)
```
