# 🔭 Vantage: Spec for Context Menus

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Phase 2.6)

## 1. Context & Problem
Currently, Arthropod has Overlays and Popups (Spec 009) and a Command System (Spec 010), but it lacks a standard mechanism for Context Menus (Right-click menus). Context menus are a ubiquitous pattern in desktop applications, allowing users to perform actions relevant to a specific item or area without navigating away or using keyboard shortcuts.
Without a built-in context menu system, developers must manually build floating overlays, handle hit testing, manage "click outside to close", and struggle with Z-index layering.

## 2. User Story

**As a** User,
**I want to** right-click on an item (e.g., a file, a row in a data grid, or a text selection),
**So that** I can see and quickly access a list of actions relevant to that specific item (e.g., Copy, Paste, Delete, Properties).

**As a** Developer,
**I want to** declaratively attach a context menu to any widget,
**So that** the framework automatically handles right-click detection, menu positioning, Z-index layering, and dismissal when the user clicks elsewhere.

## 3. The "So What?" (Business Value)
*   **Discoverability & Ergonomics**: Context menus provide immediate, localized access to features without cluttering the main UI with buttons. They are a core expectation for professional, data-dense applications.
*   **Developer Velocity**: Standardizing context menus eliminates the need for every developer to reinvent the wheel for positioning, hit testing, and overlay management.

## 4. Success Metrics
*   **Interaction Latency**: Right-clicking to show a context menu must not block the main thread > 16ms.
*   **Reliability**: Context menus must always appear fully within the viewport, flipping or adjusting position if necessary.
*   **Accessibility**: Must support keyboard navigation (Up/Down arrows to select items, Enter to trigger, Escape to close).

## 5. Acceptance Criteria

✅ **Acceptance Criteria:**
*   **Trigger Mechanism**: The system must detect `MouseButton::Right` clicks (or long presses on touch devices) on widgets that have a context menu attached.
*   **Positioning Engine**: The menu must open at the cursor's location. If the menu would overflow the screen boundaries (bottom or right), it must intelligently reposition itself (e.g., open upwards or to the left) to remain fully visible.
*   **Z-Index & Overlays**: The menu must render on the highest visual layer (above all other content, modals, and tooltips), utilizing the existing or planned Overlay System.
*   **Dismissal Rules**: The menu must close automatically when:
    *   An action item within the menu is clicked.
    *   The user clicks anywhere outside the menu bounds.
    *   The user presses the `Escape` key.
*   **Declarative API**: Developers should be able to easily attach a menu to a widget (e.g., `.context_menu(my_menu_definition)`).
*   **Integration with Command System**: Menu items should ideally be able to reference registered Commands (from Spec 010) to automatically display keyboard shortcuts and trigger the associated actions.

## 6. Out of Scope

🚫 **Out of Scope (Phase 1):**
*   **Nested Sub-menus**: Menus that open other menus when hovered (e.g., "New -> Folder", "New -> File"). Only single-level menus are required for MVP.
*   **Custom OS Menus**: Integration with native operating system context menus (we are building an in-app rendered menu).
*   **Rich Content**: Context menus containing complex widgets (like sliders or color pickers). The MVP should focus on simple text items, icons, and separators.
