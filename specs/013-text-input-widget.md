# 🔭 Vantage: Spec for Standard TextInput Widget

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Phase 2.6)

## 1. Context & Problem
Currently, Arthropod can render text and handle basic layout, but it completely lacks a mechanism for users to input textual data. An enterprise UI framework without a text input is useless for 99% of business applications (forms, search, data entry). We need a robust, accessible, and reactive text input component.

## 2. User Story

**As a** User,
**I want to** click into a field, type characters, and edit my input using standard keyboard controls (arrows, backspace),
**So that** I can enter data like my username, search queries, or numerical values into the application.

**As a** Developer,
**I want to** bind a text input to a reactive `Signal<String>`,
**So that** the application state automatically stays in sync with what the user types, without manual event wiring.

## 3. The "So What?" (Business Value)
*   **Fundamental Utility**: Data entry is the core job-to-be-done of almost all enterprise software. Without it, we are a read-only dashboard.
*   **Developer Ergonomics**: Two-way data binding via Signals will make form creation drastically faster compared to traditional MVC frameworks.

## 4. Success Metrics
*   **Typing Latency**: Input to screen latency must be < 16ms (feels instant).
*   **State Sync**: 100% reliability in syncing the input view with the underlying `Signal<String>`.

## 5. Acceptance Criteria

✅ **Acceptance Criteria:**
*   **Input Handling**: Must capture alphanumeric keyboard input and update the internal string state.
*   **Cursor Management**: Must display a blinking text caret.
*   **Basic Navigation**: Must support Left/Right arrow keys to move the cursor, and Home/End to jump to boundaries.
*   **Editing**: Must support Backspace (delete before cursor) and Delete (delete after cursor).
*   **Reactivity**: Must accept a `Signal<String>` for two-way binding. External changes to the signal must update the text field, and user typing must update the signal.
*   **Visual States**: Must visually distinguish between default, hovered, focused, and disabled states.
*   **Placeholder**: Must support rendering placeholder text (hints) when the input is empty.

## 6. Out of Scope

🚫 **Out of Scope (Phase 1):**
*   Multi-line input (Textarea).
*   Rich text or syntax highlighting.
*   Text selection (highlighting characters with Shift+Arrow or dragging).
*   OS Clipboard integration (Copy/Paste/Cut).
*   Input masking or validation rules (e.g., forcing a phone number format).
