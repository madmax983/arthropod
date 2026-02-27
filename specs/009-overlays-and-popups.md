# 🔭 Vantage: Spec for Overlays & Popups (Modals, Dialogs, Toasts, Tooltips)

**Status**: Approved (Draft Rev 2)
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
*   The system **must** provide a way to show a widget as a modal overlay via a dedicated service or API.
*   It **must** render a backdrop (dimmed background) that blocks clicks to the underlying app.
*   It **must** support "Click Backdrop to Close" (configurable, default: true).
*   It **must** support "Press Esc to Close" (configurable, default: true).
*   It **must** trap keyboard focus within the modal.
    *   **Focus Trap**: Tab key must cycle through focusable elements inside the modal only.
    *   **Focus Restoration**: When the modal closes, focus must return to the element that triggered it.
*   It **must** support nested modals (stacking on top of each other), handling multiple backdrops correctly.

### 6.2 Toast Notifications
*   The system **must** allow queuing transient messages (Info, Success, Warning, Error).
*   Toasts **must** appear in a fixed viewport location (e.g., Top-Right or Bottom-Center).
*   Toasts **must** automatically dismiss after a configurable duration (default 3s).
*   Toasts **must** pause dismissal on hover.
*   New toasts **must** stack or queue depending on configuration (e.g., max 3 visible).

### 6.3 Tooltips
*   The system **must** allow attaching a tooltip string or widget to any interactive element via a generic trait or builder method.
*   Tooltips **must** appear on Hover (Desktop) or Long Press (Touch) after a short delay (e.g., 500ms).
*   Tooltips **must** be positioned intelligently to stay within the viewport (flip behavior).
    *   If space is insufficient on Top, flip to Bottom.
    *   If space is insufficient on Left/Right, adjust accordingly.
*   Tooltips **must** implement "Show on Focus" for keyboard users.

### 6.4 Declarative Experience
*   Developers **should** be able to use a declarative API (e.g., chaining `.tooltip("Help text")` on a button).
*   Global overlays (Toasts) **should** be accessible via a global service (e.g., `ToastService`) available in the widget context.

## 7. Desired Developer Experience (Illustrative)

> **Note**: The following code snippets are illustrative examples of the desired API ergonomics. The actual implementation details are subject to Engineering design.

```rust
// 1. Tooltips (Declarative)
Button::new("Delete")
    .tooltip("This action cannot be undone")
    .on_click(...)

// 2. Toasts (Imperative/Signal-based)
// Ideally, triggering a toast should be a one-line call from an event handler.
ctx.toast().show(Toast::success("File saved!"));

// 3. Modals (Declarative with Signal control)
// Managing modal visibility should feel reactive and declarative.
let show_dialog = Signal::new(false);

Overlay::new(show_dialog.read())
    .content(
        Dialog::new()
            .title("Confirm Delete")
            .body("Are you sure?")
            .action("Delete", || delete_item())
            .cancel("Cancel", || show_dialog.set(false))
    )
```

## 8. Visual Hierarchy Strategy

To resolve "Z-Index Wars", the implementation **must** enforce a strict visual hierarchy regardless of the DOM order or z-index values.

**Hierarchy Requirements (Top to Bottom):**
1.  **Debug/DevTools**: Always on top (e.g., FPS counters).
2.  **Tooltips**: Must appear above all other UI elements, including modals and toasts.
3.  **Toasts**: Must appear above modals and standard content.
4.  **Overlays (Modals/Dialogs)**: Must appear above standard content. Nested modals must stack correctly (newest on top).
5.  **Sticky Elements**: Headers/footers.
6.  **Dropdowns**: Must appear above the input that triggered them.
7.  **Standard Content**: The base application UI.

The system **must** automatically manage stacking contexts to ensure this hierarchy is maintained without manual developer intervention (e.g., manually setting z-index: 9999).

## 9. Accessibility & Theme Integration

### 9.1 Accessibility (A11y)
*   **ARIA Roles**:
    *   Modals: Equivalent to `role="dialog"` or `role="alertdialog"`.
    *   Toasts: Equivalent to `role="status"` or `role="alert"`.
    *   Tooltips: Equivalent to `role="tooltip"` (linked via `aria-describedby`).
*   **Screen Readers**: When a modal opens, the screen reader focus must move to the modal container or its title.

### 9.2 Theme Integration
*   Overlays **must** automatically inherit the `SystemTheme` and `DesignTokens` from the root application context.
*   Backdrops **must** use the system's standard overlay backdrop token (typically semi-transparent black).
*   Dialog surfaces **must** use the system's elevated surface token (e.g., `surface_floating`).

## 10. Out of Scope (Phase 1)
*   **Draggable Modals**: Windows that can be moved around the screen.
*   **Resizable Modals**: Dragging edges to resize.
*   **Min/Max/Restore**: Window management controls.
*   **Context Menus**: Right-click menus (Special case of Popups, requires specific hit-test logic).
