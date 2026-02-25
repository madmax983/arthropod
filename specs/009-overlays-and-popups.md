# 🔭 Vantage: Spec for Overlays & Popups (Modals, Dialogs, Toasts, Tooltips)

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
Modern applications rely heavily on temporary UI layers that appear above the main content to focus user attention (Modals), provide feedback (Toasts), or offer context (Tooltips).
Currently, Arthropod developers must manually implement these using `Stack` and `z-index` hacks. This leads to:
1.  **Accessibility Failures**: Focus is not trapped in modals, allowing keyboard users to interact with the background.
2.  **Inconsistent UX**: Each developer reinvents the "Close on Backdrop Click" logic.
3.  **Z-Index Wars**: Popups fighting for visibility against other absolute-positioned elements.

## 2. User Story

### 2.1 The Modal (Dialog)
**As a** User,
**I want to** be asked for confirmation before deleting a project,
**So that** I don't accidentally lose critical data.

**As a** Keyboard User,
**I want to** have my focus trapped within the confirmation dialog,
**So that** I don't accidentally tab to a button behind the dialog and trigger it.

### 2.2 The Toast (Notification)
**As a** User,
**I want to** see a brief "Saved Successfully" message after editing a form,
**So that** I can proceed with confidence without being blocked by a modal.

### 2.3 The Tooltip
**As a** New User,
**I want to** see a helper description when I hover over a complex icon,
**So that** I can learn the interface without reading external documentation.

## 3. The "So What?" (Business Value)
*   **Safety**: Modals are the primary pattern for preventing user error (Confirm/Cancel).
*   **Accessibility Compliance**: A custom `Stack`-based modal without focus management is a WCAG violation (Criteria 2.1.2 No Keyboard Trap). A standard component ensures compliance by default.
*   **Developer Velocity**: "Show Error Message" is a daily task. It should be a one-line function call, not a 50-line widget construction.

## 4. Success Metrics
*   **Interaction Latency**: Opening a modal must not block the main thread > 16ms.
*   **Accessibility**: Screen readers must announce the modal title immediately upon opening (ARIA `alertdialog` role).
*   **Focus Management**: 100% of "Tab" key presses must remain within the modal while it is open.

## 5. Gap Analysis

| Feature | `Stack` + Manual State (Current) | `OverlayService` (Target) |
| :--- | :--- | :--- |
| **Visibility** | Boolean flag in Parent State | Global Service / Signal |
| **Z-Index** | Manual integer juggling | Automatic Stacking Context |
| **Focus** | Leaks to background | Trapped in Overlay |
| **Backdrop** | Manual `Rect` with `OnClick` | Built-in, Configurable |
| **Dismiss** | Manual Implementation | `Esc` key, Backdrop Click |
| **Positioning** | Absolute coordinates | Automatic (Center, Top-Right, Anchored) |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 Modal Dialogs
*   The system **must** provide a way to show a widget as a modal overlay.
*   It **must** render a backdrop (dimmed background) that blocks clicks to the underlying app.
*   It **must** support "Click Backdrop to Close" (configurable).
*   It **must** support "Press Esc to Close" (configurable).
*   It **must** trap keyboard focus within the modal.
*   It **must** support nested modals (stacking on top of each other).

### 6.2 Toast Notifications
*   The system **must** allow queuing transient messages (Info, Success, Warning, Error).
*   Toasts **must** appear in a fixed viewport location (e.g., Top-Right or Bottom-Center).
*   Toasts **must** automatically dismiss after a configurable duration (default 3s).
*   Toasts **must** pause dismissal on hover.

### 6.3 Tooltips
*   The system **must** allow attaching a tooltip string or widget to any interactive element.
*   Tooltips **must** appear on Hover (Desktop) or Long Press (Touch).
*   Tooltips **must** be positioned intelligently to stay within the viewport (e.g., flip from Top to Bottom if near screen edge).

### 6.4 Declarative API
*   Developers **should** be able to use a declarative API (e.g., `Button::new("Save").tooltip("Saves to disk")`).
*   Global overlays (Toasts) **should** be accessible via a service or command (e.g., `Toast::show("Saved!")`).

## 7. Constraint Requirements
*   **Root Level Rendering**: Overlays **must** be rendered at the root of the Scene tree to avoid `overflow: hidden` clipping from parent containers. (Portals).
*   **Theme Integration**: Overlays **must** inherit the current theme (Dark/Light mode) automatically.

## 8. Out of Scope (Phase 1)
*   **Draggable Modals**: Windows that can be moved around the screen.
*   **Resizable Modals**: Dragging edges to resize.
*   **Min/Max/Restore**: Window management controls.
*   **Context Menus**: Right-click menus (Special case of Popups, but complex positioning logic).
