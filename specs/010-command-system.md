# 🔭 Vantage: Spec for Command System & Palette

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
Enterprise applications are tools for professionals. Professionals demand efficiency.
Currently, Arthropod relies on ad-hoc event listeners for keyboard shortcuts (e.g., `if key == Key::S && ctrl`). This leads to:
1.  **Hardcoded Shortcuts**: Users cannot remap keys.
2.  **No Discovery**: Users don't know what shortcuts exist.
3.  **Inconsistent State**: Actions (like "Save") are duplicated across Menu Bars, Buttons, and Shortcuts, leading to code duplication.

## 2. User Story
**As a** Power User,
**I want to** press `Ctrl+Shift+P` (or `Cmd+K`) to open a command palette, search for "Toggle Dark Mode", and execute it,
**So that** I can keep my hands on the keyboard and work faster.

**As a** Developer,
**I want to** define an action ("Save File") once and bind it to a menu item, a button, and a keyboard shortcut simultaneously,
**So that** I don't have to maintain three separate code paths.

## 3. The "So What?" (Business Value)
*   **Efficiency**: Reducing time-to-action for power users directly correlates with productivity in data-heavy apps.
*   **Discoverability**: A Command Palette exposes features that might otherwise be buried in nested menus.
*   **Accessibility**: Providing keyboard alternatives for all pointer actions is a WCAG requirement.

## 4. Success Metrics
*   **Latency**: Command Palette must open in < 16ms.
*   **Coverage**: 100% of "Menu Bar" actions must be available in the Command Palette.
*   **Configurability**: 100% of shortcuts must be remappable by the user (Phase 2).

## 5. Gap Analysis

| Feature | `winit` Events (Current) | `CommandSystem` (Target) |
| :--- | :--- | :--- |
| **Definition** | Hardcoded `if/else` | Declarative Registry |
| **Discovery** | None (Read the source) | Searchable Palette |
| **Binding** | Hardcoded | Remappable (Keymap) |
| **Context** | Global only | Scoped (Window/Widget focus) |
| **Metadata** | None | Title, Icon, Category, Description |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 Command Registry
*   The system **must** maintain a central registry of available commands.
*   Each command **must** have a unique ID (e.g., `file.save`), a human-readable title, an optional icon, and an execution handler.
*   Commands **must** be able to define a default keyboard shortcut.

### 6.2 Keybinding Service
*   The system **must** listen for keyboard inputs and match them against the active keymap.
*   It **must** handle modifier keys (`Ctrl`, `Shift`, `Alt`, `Meta`) correctly across platforms (mapping `Cmd` to `Ctrl` on macOS where appropriate).
*   It **must** support "Chords" (e.g., `Ctrl+K, Ctrl+S`) (Phase 2, but architecture should allow it).

### 6.3 Command Palette Widget
*   A built-in `CommandPalette` widget **must** be provided.
*   It **must** be summonable via a global shortcut (default `Ctrl+Shift+P` / `Cmd+P`).
*   It **must** filter commands via fuzzy search on Title and Category.
*   It **must** execute the selected command and close automatically.

### 6.4 Context Awareness
*   Commands **should** be able to declare "When" clauses (e.g., `when: editorFocus`).
*   The palette **must** only show commands valid for the current context.

## 7. API Design (Draft)

```rust
// Registration
commands.register(
    Command::new("editor.format")
        .title("Format Document")
        .category("Editor")
        .shortcut(Modifiers::CTRL | Modifiers::SHIFT, Key::F)
        .action(|ctx| {
            // Logic here
        })
);

// Execution
commands.execute("editor.format");
```

## 8. Out of Scope (Phase 1)
*   **User Keymap Editor**: A UI for users to remap keys. (The backend support is in scope, the UI is not).
*   **Multi-step Commands**: Wizards inside the palette.
*   **Vim Mode**: Complex stateful keybindings.
