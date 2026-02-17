# 🔭 Vantage: Spec for Input Fusion System

> "A GUI without input is just a screenshot."

## 1. 👤 User Story

**As a** UI Developer,
**I want** to attach declarative gesture recognizers (like `OnClick`, `OnDrag`, `OnHover`) to any widget,
**So that** I can build rich, interactive interfaces without manually managing raw window events, state machines, and hit-testing logic.

## 2. 🧐 The "So What?" (Business Value)

Currently, implementing a simple "Draggable Box" in Arthropod requires:
1.  Listening to raw `WindowEvent`s in the main loop.
2.  Manually performing hit-testing against the Scene Graph.
3.  Managing `is_dragging` state variables.
4.  Handling edge cases like "mouse released outside window".

**This complexity is a cost.** It leads to:
-   **Brittle Code**: Hard to maintain and debug.
-   **Inconsistent UX**: Every developer re-invents the wheel (e.g., drag threshold).
-   **Slow Iteration**: Developers spend time plumbing events instead of building features.

**Utility is Revenue**: By abstracting this complexity, we enable developers to build higher-quality apps faster.

## 3. 🔍 Gap Analysis

| Feature | `plat-core` (Raw) | `widget-core` (Current) | `Input Fusion` (Target) |
| :--- | :--- | :--- | :--- |
| **Event Source** | OS Window Events | Manual function calls | Unified Input Stream |
| **Hit Testing** | None | Manual / Ad-hoc | Automatic (Scene Graph) |
| **Gestures** | None | Hardcoded (TextInput) | Declarative (`.on_click(...)`) |
| **State Mgmt** | Manual | Widget-internal | ECS Component (`InputState`) |
| **Device Support** | Mouse/Keyboard | Keyboard (mostly) | Mouse, Touch, Pen, Gamepad |

## 4. 📝 Solution Overview

We will introduce a new system, **Input Fusion**, that sits between `plat-core` and `widget-core`.

### Architecture

1.  **Input Source**: Consumes raw `plat-core::WindowEvent`s.
2.  **Pointer Abstraction**: Unifies Mouse, Touch, and Pen into generic `Pointer` events.
3.  **Hit Testing System**: Queries the `Scene` graph to find the target node under the cursor.
4.  **Event Propagation**: Implements a Bubbling/Capturing phase (similar to DOM) to allow parent widgets to intercept events.
5.  **Gesture Recognizers**: ECS Components that attach to entities and interpret event streams into high-level actions.

### Key Features

-   **`InputState` Resource**: A centralized resource in the ECS that tracks the current state of input devices (cursor position, pressed keys).
-   **Declarative API**:
    ```rust
    // Example (Conceptual)
    Button::new("Click Me")
        .on_click(|ctx| println!("Clicked!"))
        .on_hover(|ctx, hovering| ctx.set_color(if hovering { RED } else { BLUE }))
    ```
-   **Focus Management**: A robust system for handling keyboard focus (Tab navigation) and focus groups.

## 5. 📊 Metrics (Success Definition)

-   **Code Reduction**: Implement a "Draggable Box" example with **50% fewer lines of code** compared to the raw implementation.
-   **Latency**: Input processing overhead < 1ms per frame.
-   **Coverage**: Support 100% of standard desktop interactions (Click, DoubleClick, Drag, Scroll, Hover, Focus).

## 6. ✅ Acceptance Criteria

### Must Have (Phase 1)
-   [ ] **Pointer Events**: `PointerDown`, `PointerUp`, `PointerMove` (abstracted from Mouse).
-   [ ] **Hit Testing**: ability to identify the Scene Node under the cursor.
-   [ ] **Event Propagation**: Events bubble up from the target node to the root.
-   [ ] **Basic Gestures**:
    -   `OnClick`: Fires on down + up on the same element.
    -   `OnHover`: Fires when pointer enters/leaves bounds.
    -   `OnDrag`: Fires delta updates while pointer is down and moving.
-   [ ] **ECS Integration**: Input state is stored in ECS components/resources.

### Should Have (Phase 2)
-   [ ] **Keyboard Shortcuts**: Declarative binding of keys to actions (e.g., `Ctrl+S`).
-   [ ] **Focus Trapping**: For modals/dialogs.

### Could Have (Future)
-   [ ] **Multi-touch**: Pinch-to-zoom, Rotate.
-   [ ] **Gamepad Support**: Navigation via D-pad.

## 7. 🚫 Out of Scope

-   **Gesture Recording/Replay**: While useful for testing, it's not core to the runtime.
-   **Haptic Feedback**: Not MVP.
-   **Voice Control**: Not MVP.

## 8. 📅 Timeline / ROI

-   **ROI**: High. This is a foundational capability for any GUI framework. Without it, `widget-core` cannot scale beyond simple forms.
-   **Effort**: Medium (2-3 weeks). Requires careful design of the hit-testing and propagation algorithms.
