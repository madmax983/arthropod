# 🔭 Vantage: Spec for Test Harness Phase 1

**Status**: Approved
**Owner**: Vantage
**Target Release**: Phase 1 Update

## 1. Context & Problem
The `arthropod-design-doc.md` outlines a "Testing Advantage" where Arthropod owns the stack to provide semantic testing.
Currently, `crates/arthropod-test` only contains low-level GPU capture tools (`RenderDoc`).
There is no way to write a high-level integration test (e.g., "Click button, check text") without spinning up a full window or hacking the event loop.

This lack of tooling forces developers to rely on manual verification or brittle unit tests, violating our "Test-Driven Development" (TDD) core philosophy.

## 2. Why Now?
*   **TDD Enforcement**: We cannot enforce TDD for high-level UI interactions if the tools don't exist.
*   **CI Stability**: We need a way to run integration tests in headless CI environments (GitHub Actions) without requiring a GPU or display server.
*   **Accessibility First**: By building the test harness on top of the Accessibility Engine (`a11y-engine`), we ensure that "testable code" is also "accessible code".

## 3. User Story
**As a** UI Developer,
**I want to** write headless integration tests using semantic queries (Roles, Labels),
**So that** I can verify my application logic and accessibility structure without brittle, slow manual testing.

**Example Workflow:**
```rust
#[test]
fn test_login_flow() {
    let mut app = TestHarness::new_headless(LoginPage::new());

    // Semantic Query: Find by Role + Name
    app.find(By::Role(Role::Textbox).name("Username"))
       .type_text("admin");

    app.find(By::Role(Role::Textbox).name("Password"))
       .type_text("secret");

    app.find(By::Role(Role::Button).name("Login"))
       .click();

    // Assert State
    assert!(app.find(By::Role(Role::Alert)).has_text("Welcome!"));
}
```

## 4. The "So What?" (Business Value)
*   **Velocity**: Catch regressions in CI (< 1s) vs Manual QA (hours).
*   **Quality**: Enforces accessibility (A11y) by design. If you can't find it by Role, neither can a Screen Reader.
*   **Confidence**: Refactor internal widget implementation without breaking tests, as tests rely on *behavior* (Roles), not *implementation* (Structs).

## 5. Success Metrics
*   **Performance**: Headless test instantiation < 10ms.
*   **Stability**: 0% flakiness on semantic queries (unlike Selenium/CSS selectors).
*   **Coverage**: 100% of standard widgets (Button, Input, Checkbox) must be queryable by Role.

## 6. Acceptance Criteria (MVP)

### 6.1 Headless Harness
*   The system **must** expose a `TestHarness` struct that wraps an `App`.
*   It **must** support a `new_headless(app)` constructor that initializes the application without creating a platform window (no `winit` dependency in this mode).
*   It **must** allow manual event loop pumping via `.update()` or `.advance_frame()`.

### 6.2 Semantic Locators
*   The system **must** support finding elements by `a11y-engine::Role`.
    *   Supported Roles: `Button`, `Textbox`, `Checkbox`, `Radio`, `Slider`, `Group`, `List`, `ListItem`, `Heading`, `Main`, `Navigation`, `Dialog`, `Alert`.
*   The system **must** support filtering by `AccessibleName` (e.g., `.name("Submit")`).
*   It **must** return a `TestHandle` or similar proxy object representing the found element.
*   It **must** return an error or panic with a helpful message if the element is not found (e.g., "No element found with Role::Button and Name 'Submit'. Did you mean 'Save'?").

### 6.3 Interaction
*   The `TestHandle` **must** support basic actions that dispatch synthesized events:
    *   `.click()`: Dispatches PointerDown/Up events at the center of the element's bounds.
    *   `.type_text(text)`: Dispatches Keyboard events (requires the element to be focused).
    *   `.focus()`: Sets keyboard focus to the element.
*   These actions **must** trigger the standard `widget-core` event handlers (e.g., `on_click`).

### 6.4 State Assertion
*   The `TestHandle` **must** expose current widget state derived from the `A11yNode`:
    *   `.text()`: Returns the text content.
    *   `.is_enabled()`: Returns `!state.disabled`.
    *   `.is_checked()`: Returns `state.checked`.
    *   `.is_focused()`: Returns `state.focused`.

## 7. Constraint Requirements
*   **No GPU Requirement**: The headless mode **must not** attempt to initialize WGPU or Vulkan. It should use a "Null" or "Software" backend for layout/rendering passes if necessary.
*   **Platform Agnostic**: Tests written with the harness **must** pass on Windows, macOS, and Linux without modification.

## 8. Out of Scope (Phase 1)
*   **Visual Regression**: Screenshot comparison is handled by a separate tool (Playwright/External). This harness focuses on *logical* correctness.
*   **Fuzz Testing**: Generating random inputs.
*   **Time-travel**: Testing undo/redo stacks.
