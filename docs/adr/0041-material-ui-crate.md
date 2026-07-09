# 0041. Material UI Crate

Date: 2026-04-09

## Status

Accepted

## Context

Arthropod needs a comprehensive, high-quality set of UI components to be a viable framework for application development. The core `widget-core` crate provides fundamental primitives (like `Button`, `Text`, `TextInput`), but these are intentionally unopinionated and lack the cohesive visual language, interactive feedback, and complex state management expected of a modern UI library. We selected Material Design 3 (MD3) as the reference design system because it is well-documented, widely understood, and offers a robust token system. However, integrating this directly into `widget-core` would bloat the foundational layer and violate our separation of concerns.

## Decision

We created a dedicated `material-ui` crate to encapsulate the Material Design 3 component library. This crate builds upon the primitives provided by `widget-core` and the styling capabilities of `theme-engine`.

## Consequences

### Positive
- **Separation of Concerns:** `widget-core` remains lean and unopinionated, focusing solely on the widget trait and fundamental primitives.
- **Cohesive Design System:** Applications can use `material-ui` to quickly build complex, visually consistent interfaces following MD3 guidelines.
- **Modularity:** Developers who want to build their own design systems or use a different visual language can simply ignore `material-ui` without paying for its cost.

### Negative
- **Increased Maintenance Surface:** A new crate requires its own set of tests, benchmarks, and documentation.
- **Dependency Graph:** `material-ui` adds another layer to the dependency graph, depending on `widget-core`, `theme-engine`, `style-engine`, `flux-state`, `render-engine`, and `layout-engine`.
