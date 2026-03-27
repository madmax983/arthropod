# 🔭 Vantage: Spec for Multi-Window Support

**Status**: Proposed
**Owner**: Vantage
**Target Release**: Arthropod Core

## 1. Context & Problem
Currently, the Arthropod GUI framework only supports a single application window per execution. The platform layer (`plat-core`) and application core (`arthropod`) are tightly coupled to a single `winit::window::Window` and its associated rendering context.
Modern desktop applications (IDEs, multi-document editors, complex tools) frequently require multiple independent or connected windows to be productive.

## 2. User Story
**As a** Developer building a code editor,
**I want to** open different files in separate floating windows,
**So that** I can utilize multiple monitors for side-by-side editing.

**As a** User of a dashboard application,
**I want to** tear off specific graphs into their own mini-windows,
**So that** I can keep an eye on them while working in other applications.

## 3. The "So What?" (Business Value)
*   **Professional Utility**: Unlocks the framework for complex, professional-grade tools that require multi-monitor workflows.
*   **Competitive Parity**: Brings Arthropod in line with other major GUI frameworks (Tauri, Electron, Slint) which support multi-window out of the box.
*   **Flexibility**: Allows developers to build modular UI components (e.g., tear-off panels, persistent toolboxes).

## 4. Success Metrics
*   **Spawning Time**: < 50ms to spawn a new window with a basic widget tree.
*   **Resource Sharing**: Minimal duplication of heavy resources (fonts, textures) across windows.
*   **Event Routing**: 100% accurate routing of input events to the correct window's ECS world/Scene Graph.
*   **Stability**: No crashes or panics when closing secondary windows while the main window remains open.

## 5. Acceptance Criteria (MVP)

### 5.1 Window Management API
*   The `App` struct (or a new `WindowManager` resource) **must** provide an API to dynamically spawn new windows at runtime.
*   The API **must** accept window configuration (title, size, resizable, etc.) and a root widget builder for that specific window.

### 5.2 Event Routing
*   The platform event loop (`winit`) **must** route window-specific events (resize, close, input, redraw) to the corresponding window's state.
*   Input events (mouse, keyboard) **must** only affect the active/focused window.

### 5.3 Rendering Contexts
*   Each window **must** maintain its own `wgpu::Surface` and `wgpu::SurfaceConfiguration`.
*   The rendering engine **must** be able to execute layout and rendering passes independently for each window.

### 5.4 Application Lifecycle
*   Closing a secondary window **must not** exit the entire application.
*   Closing the "main" window (or the last open window) **should** trigger the application exit sequence by default (configurable).

### 5.5 State Sharing (Basic)
*   Windows **must** be able to read and write to the same global `flux-state` signals, enabling reactive UI updates across different windows simultaneously.

## 6. Technical Notes (For Engineering Context)
*   **Winit Integration**: `winit::event_loop::EventLoop` needs to manage a map of `WindowId` to window state.
*   **ECS World**: Decide whether each window gets its own Bevy ECS `World` or if they share one `World` with components namespaced by `WindowId`. Sharing a world is likely easier for cross-window reactivity but requires careful system scheduling.
*   **WGPU Device**: A single `wgpu::Instance` and `wgpu::Device` should be shared across all windows to share GPU resources, but each needs its own `Surface`.

## 7. Out of Scope (Phase 1)
*   **Cross-Window Drag and Drop**: Dragging a widget from one window and dropping it into another.
*   **Tear-off Tabs**: Native UI for dragging a tab out of a window to spawn a new one (the underlying API should support this, but the UI component is out of scope).
*   **Window State Persistence**: Automatically saving and restoring window positions/sizes across application restarts.
