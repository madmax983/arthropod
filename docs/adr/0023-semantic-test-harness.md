# ADR 0023: Semantic Test Harness

**Status:** Accepted

**Date:** 2026-01-20

**Deciders:** Vantage, Architecture Team

## Context

Test-Driven Development (TDD) is a core philosophy of Arthropod, but the current tooling in `crates/arthropod-test` is limited to low-level GPU capture tools (RenderDoc).

Developers face several challenges:
1.  **Lack of High-Level Integration Testing:** There is no way to write tests that simulate user interactions (e.g., "Click button", "Check text") without spinning up a full window or hacking the event loop.
2.  **Brittle Tests:** Manual verification or unit tests that depend on internal implementation details are fragile and hard to maintain.
3.  **Accessibility Gaps:** Without semantic testing tools that rely on accessibility roles (Roles, Labels), it's difficult to enforce accessibility standards automatically.
4.  **CI Limitations:** Running integration tests in headless CI environments (GitHub Actions) is difficult without a GPU or display server.

We need a testing framework that enables semantic, headless integration testing to ensure logical correctness and accessibility compliance.

## Decision

We will implement a **Semantic Test Harness** in `crates/arthropod-test` that provides a high-level API for testing Arthropod applications.

### Key Features

1.  **Headless Execution:**
    -   The harness will support a `new_headless(app)` constructor that initializes the application without creating a platform window.
    -   It will use a "Null" or "Software" backend for layout/rendering passes, avoiding `winit` and GPU dependencies in this mode.
    -   It allows manual event loop pumping via `.update()` or `.advance_frame()`.

2.  **Semantic Locators (A11y-First):**
    -   Tests will locate widgets using `a11y-engine::Role` (e.g., Button, Textbox, Checkbox) and `AccessibleName`.
    -   This enforces accessibility by design: if a widget cannot be found by its semantic role, it is considered untestable and inaccessible.
    -   The API will return a `TestHandle` for interaction.

3.  **Interaction Simulation:**
    -   The `TestHandle` will support methods like `.click()`, `.type_text(text)`, and `.focus()`.
    -   These methods will dispatch synthesized events (PointerDown/Up, Keyboard) to the application's event loop, triggering standard `widget-core` handlers.

4.  **State Assertion:**
    -   The `TestHandle` will expose state derived from the `A11yNode`, such as `.text()`, `.is_enabled()`, `.is_checked()`, and `.is_focused()`.

### API Example

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

## Consequences

### Positive

*   **Enables TDD:** Developers can write tests for UI logic before implementation.
*   **Accessibility Enforcement:** Tests fail if widgets lack proper accessibility semantics.
*   **CI Compatibility:** Tests run quickly (< 1s) in headless environments without GPU requirements.
*   **Refactoring Confidence:** Tests rely on behavior (Roles), not implementation structure, making refactoring safer.
*   **Improved Quality:** Catch regressions early in the development cycle.

### Negative

*   **Maintenance Overhead:** Requires maintaining a separate "headless" backend and ensuring it stays in sync with the real platform backend.
*   **Limited Visual Testing:** This harness focuses on logical correctness; visual regression testing (pixels) requires separate tools (e.g., Playwright).
*   **Mocking Complexity:** Some platform-specific features may need to be mocked or stubbed in the headless environment.

## References

*   [Specs 001: Test Harness Phase 1](../../specs/001-test-harness-v1.md)
*   [ADR 0010: Accessibility First Architecture](./0010-accessibility-first-architecture.md)
