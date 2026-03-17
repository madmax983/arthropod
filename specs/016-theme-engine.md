# 🔭 Vantage: Spec for Theme & Design Token System

**Status**: Draft
**Owner**: Vantage (Product Manager)
**Target Release**: Arthropod Core (Feature Promotion)

## 1. 👤 User Story

**As a** UI Designer,
**I want** to define a centralized set of Design Tokens (colors, typography, spacing, border radii),
**So that** my entire application has a consistent visual identity and I can easily update the branding without manually finding and replacing values in code.

**As a** User,
**I want** to switch between Light Mode and Dark Mode with a single click,
**So that** the application adapts to my environment and reduces eye strain.

## 2. 🧐 The "So What?" (Business Value)

Currently, Arthropod widgets are styled using hardcoded values or localized structures like `VisualStyle` and `FlexStyle`. This makes implementing a unified brand identity across an enterprise application extremely difficult and error-prone.

When a company rebrands (e.g., changes its primary color from Blue to Purple), developers must hunt down every instance of `Color::BLUE`. This is a massive cost in terms of developer time and QA.

Furthermore, Dark Mode is no longer a "nice-to-have"; it is a mandatory requirement for modern software. Without a centralized Theme Engine, Dark Mode requires writing duplicate styling logic for every single widget.

**Utility is Revenue**:
A Theme Engine ensures that:
- Applications look professional and consistent by default.
- Rebranding takes minutes, not weeks.
- Dark Mode is supported out-of-the-box, increasing user satisfaction and accessibility.

## 3. 🔍 Gap Analysis

| Feature | Current State | Market Standard (e.g., CSS/Tailwind) | Target State (Theme Engine) |
| :--- | :--- | :--- | :--- |
| **Color Definition** | Hardcoded `Color` structs | CSS Variables / Design Tokens | Centralized `DesignTokens` |
| **Dark Mode** | Manual logic per widget | Media Queries / Class toggles | Reactive `ThemeMode` signal |
| **Typography** | Hardcoded sizes/fonts | Typographic Scales | Centralized `TypographyScale` |
| **Spacing/Layout** | Hardcoded pixels | Spacing Variables (e.g., `gap-4`) | `SpacingTokens` |
| **Consistency** | Low | High | High |

## 4. 📝 Solution Overview

We will introduce a **Theme & Design Token System** into `widget-core`.

### Key Capabilities

1.  **Design Tokens Registry**:
    -   A hierarchical structure defining semantic values.
    -   *Colors*: `Primary`, `Secondary`, `Background`, `Surface`, `Error`, `TextPrimary`, `TextSecondary`, etc.
    -   *Typography*: `Heading1`, `Body`, `Caption`, `Monospace`.
    -   *Spacing*: `Small`, `Medium`, `Large`, `XLarge`.
    -   *Borders*: `RadiusSmall`, `RadiusMedium`, `RadiusRound`.

2.  **Reactive Theme State**:
    -   The active theme must be stored in a `flux-state` `Signal<Theme>`.
    -   This allows the entire UI to re-render instantly when the user toggles between Light and Dark mode.

3.  **Widget Integration**:
    -   Standard widgets (`Button`, `Text`, `TextInput`, `Container`) must default to using Design Tokens rather than hardcoded colors.
    -   For example, a `Button`'s background should default to `theme.colors.primary`.

4.  **Context-Based Provisioning**:
    -   The Theme must be injected into the UI tree via a `ContextProvider` so that deeply nested widgets can access the active tokens without prop drilling.

## 5. 📊 Metrics (Success Definition)

-   **Adoption Time**: A developer should be able to apply a custom corporate theme (colors and fonts) to a new app in < 10 minutes.
-   **Code Reduction**: Implementing Dark Mode should require 0 additional lines of styling logic within individual widget implementations.
-   **Performance**: Theme toggling must trigger a reactive update that completes in < 16ms (1 frame).

## 6. ✅ Acceptance Criteria

### Must Have (Phase 1)
-   [ ] **Token Definitions**: A robust struct defining semantic colors, typography, and spacing.
-   [ ] **Light & Dark Presets**: The framework must ship with sensible default Light and Dark themes.
-   [ ] **Theme Provider**: A mechanism to provide the `Theme` context to the widget tree.
-   [ ] **Widget Refactoring**: Refactor `Button`, `Text`, and `TextInput` to consume Theme tokens by default.
-   [ ] **Reactive Toggling**: The ability to update the `Theme` signal at runtime and observe the UI update instantly.

### Should Have (Phase 2)
-   [ ] **Theme Builder API**: A fluent API for developers to easily override specific tokens (e.g., `Theme::default().with_primary(Color::RED)`).
-   [ ] **System Preference Detection**: Automatically detect the OS-level Light/Dark preference and apply it on startup.

### Could Have (Future)
-   [ ] **High Contrast Theme**: An accessible theme preset.
-   [ ] **CSS/JSON Import**: The ability to load design tokens from standard formats (e.g., Figma Token JSON).

## 7. 🚫 Out of Scope

-   **Runtime CSS Parsing**: This is not a web browser. We use structured Rust types for styling, not string-based CSS files.
-   **Animation of Theme Changes**: Smoothly animating colors from Light to Dark mode is complex and out of scope for MVP. It should be an instant toggle.
-   **Widget-Specific Themes**: Overriding the theme for a single deeply nested widget (use local inline styles for that, not the global theme engine).