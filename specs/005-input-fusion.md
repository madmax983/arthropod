# 🔭 Vantage: Spec for Input Fusion System

> "A GUI without input is just a screenshot."

**Status**: Approved
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

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
-   **Inaccessible Apps**: Custom input handling often bypasses standard accessibility hooks.

**Utility is Revenue**: By abstracting this complexity, we enable developers to build higher-quality, accessible apps faster.

## 3. 🔍 Gap Analysis

| Feature | `plat-core` (Raw) | `widget-core` (Current) | `Input Fusion` (Target) |
| :--- | :--- | :--- | :--- |
| **Event Source** | OS Window Events | Manual function calls | Unified Input Stream |
| **Hit Testing** | None | Manual / Ad-hoc | Automatic (Scene Graph) |
| **Gestures** | None | Hardcoded (TextInput) | Declarative (`.on_click(...)`) |
| **State Mgmt** | Manual | Widget-internal | ECS Component (`InputState`) |
| **Focus** | Manual | `TextInput` only | Global Focus Manager (Tab/Trap) |
| **Accessibility**| None | Manual wiring | Automatic A11y Tree Updates |

## 4. 📝 Solution Overview

We will introduce a new system, **Input Fusion**, that sits between `plat-core` and `widget-core`.

### Architecture

1.  **Input Manager Resource**: A central ECS resource (`InputManager`) that:
    -   Consumes raw `plat-core::WindowEvent`s.
    -   Normalizes them into `PointerEvent` (Mouse/Touch/Pen) and `KeyboardEvent`.
    -   Maintains the current state of devices (cursor position, pressed keys).

2.  **Hit Testing System**:
    -   Queries the `Scene` graph to find the target node under the cursor.
    -   **Algorithm**: Traverses the scene tree in reverse render order (front-to-back) to respect Z-indexing.
    -   **Optimization**: Must use bounding box checks before precise shape checks.

3.  **Event Propagation (The Bubble)**:
    -   Events traverse from the **Target Node** up to the **Root**.
    -   Widgets can "capture" events to stop propagation (e.g., a button eats the click so the container doesn't see it).

4.  **Focus Management**:
    -   Maintains a "Focus Chain" for keyboard navigation.
    -   Handles `Tab` / `Shift+Tab` to cycle through focusable widgets.
    -   Supports **Focus Traps** for modal dialogs (prevent tabbing outside the modal).

5.  **Gesture Recognizers**:
    -   ECS Components attached to entities (e.g., `Clickable`, `Draggable`, `Hoverable`).
    -   The `InputSystem` iterates these components and triggers callbacks when patterns match.

### Integration with Accessibility (A11y)
-   **Focus Sync**: When `InputManager` changes focus, it **must** update the `a11y-engine` selection.
-   **Action Mapping**: A "Click" gesture must also be triggerable via the A11y "Activate" action (e.g., from a Screen Reader).

## 5. 📊 Metrics (Success Definition)

-   **Code Reduction**: Implement a "Draggable Box" example with **50% fewer lines of code** compared to the raw implementation.
-   **Latency**: Input processing overhead < **1ms** per frame.
-   **Hit Test Performance**: Querying the scene graph for a target must take < **0.1ms** for 1000 nodes.
-   **Coverage**: Support 100% of standard interactions (Click, DoubleClick, Drag, Scroll, Hover, Focus).

## 6. ✅ Acceptance Criteria

### Must Have (Phase 1)
-   [ ] **Pointer Abstraction**: Unified `PointerDown`, `PointerUp`, `PointerMove` events.
-   [ ] **Efficient Hit Testing**: O(N) or better lookup of the node under cursor, respecting bounds and visibility.
-   [ ] **Event Bubbling**: Events start at the target and bubble up to the root.
-   [ ] **Declarative Gestures**:
    -   `OnClick`: Fires on down + up on the same element.
    -   `OnHover`: Fires `HoverEnter` / `HoverLeave`.
    -   `OnDrag`: Fires `DragStart` -> `DragMove` -> `DragEnd`.
-   [ ] **Focus System**:
    -   `Tab` navigation between focusable elements.
    -   Visual focus indication (e.g., `pseudoclass: focus`).
-   [ ] **ECS Integration**: Input state is stored in ECS, allowing systems to query "Is Shift held?" or "Is Mouse(Left) pressed?".

### Should Have (Phase 2)
-   [ ] **Keyboard Shortcuts**: Declarative binding (e.g., `Ctrl+S` -> Save).
-   [ ] **Focus Trapping**: `FocusScope` component for Modals.
-   [ ] **Multi-touch**: Pinch/Zoom gestures.

### Could Have (Future)
-   [ ] **Gamepad Support**: Navigation via D-pad.
-   [ ] **Gesture Recording**: For automated testing.

## 7. 🚫 Out of Scope

-   **Haptic Feedback**: Not MVP.
-   **Voice Control**: Not MVP.
-   **Complex Gesture Recognition**: e.g., "Draw a Circle" (unless via custom recognizer).

## 8. 📅 Timeline / ROI

-   **ROI**: **Critical**. This is the "nervous system" of the framework. Without it, we cannot build complex, accessible applications.
-   **Effort**: Medium (2-3 weeks).
    -   Week 1: Hit Testing & Pointer Events.
    -   Week 2: Event Propagation & Basic Gestures (Click/Hover).
    -   Week 3: Focus Management & A11y Integration.
