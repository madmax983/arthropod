# 32. Extract Input Engine

Date: 2024-05-20

## Status

Accepted

## Context

Input handling in Arthropod was historically fragmented. `plat-core` provided raw window events (`WindowEvent`), while `widget-core` implemented ad-hoc logic for basic interactions like clicking and typing. This led to several issues:

1.  **Duplication**: Every widget needing complex input had to reimplement hit-testing or state tracking.
2.  **Inconsistency**: Different widgets might interpret "drag" or "click" slightly differently.
3.  **Scalability**: Adding new input methods (e.g., touch, pen, gamepad) would require changes across the entire widget library.
4.  **Accessibility**: Without a centralized input manager, it was difficult to coordinate focus changes with the accessibility tree.

Product Specification `005-input-fusion` ("Input Fusion") was approved to address these issues by defining a unified input processing pipeline.

## Decision

We will extract input handling logic into a dedicated crate, `input-engine`, located at `crates/input-engine`. This crate will serve as the implementation of the "Input Fusion" specification.

The `input-engine` crate will:
1.  **Centralize Input State**: Manage the state of input devices (mouse position, key states) via an `InputManager` resource.
2.  **Implement Gesture Recognition**: Provide high-level gesture recognizers (e.g., `SequenceMatcher`, `ChordMatcher`) that consume raw events and emit semantic actions.
3.  **Handle Hit Testing**: Provide the logic for determining which UI element is under a pointer (using Reverse Painter's Algorithm).
4.  **Manage Focus**: Control the focus chain and navigation (Tab traversal).

The `arthropod` application crate will integrate `input-engine` into the main loop, feeding it events from `plat-core` and propagating the results to the ECS or `widget-core`.

## Consequences

### Positive
*   **Separation of Concerns**: Input logic is isolated from rendering and widget definitions.
*   **Reusability**: Gesture logic can be reused across different widgets and even different platforms.
*   **Testability**: Input scenarios can be tested in isolation without spinning up a full GUI application.
*   **Extensibility**: New input devices or gesture types can be added without modifying core widget code.

### Negative
*   **Complexity**: Introduces a new crate and dependency edge in the workspace graph.
*   **Migration Cost**: Existing widgets in `widget-core` must be refactored to use the new `input-engine` types and logic, moving away from ad-hoc event handling.
*   **Integration Glue**: The `App` must now orchestrate the flow of events between `plat-core`, `input-engine`, and `widget-core`, adding some boilerplate to the main loop.
