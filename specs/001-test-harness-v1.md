# 🔭 Vantage: Spec for Test Harness Phase 1

**Status**: Draft
**Owner**: Vantage
**Target Release**: Phase 1 Update

## 1. Context & Problem
The `arthropod-design-doc.md` outlines a "Testing Advantage" where Arthropod owns the stack to provide semantic testing.
Currently, `crates/arthropod-test` only contains low-level GPU capture tools (`RenderDoc`).
There is no way to write a high-level integration test (e.g., "Click button, check text") without spinning up a full window or hacking the event loop.

## 2. User Story
**As a** UI Developer,
**I want to** write headless integration tests using semantic queries (Roles, Labels),
**So that** I can verify my application logic and accessibility structure without brittle, slow manual testing.

## 3. The "So What?" (Business Value)
*   **Velocity**: Catch regressions in CI (< 1s) vs Manual QA (hours).
*   **Quality**: Enforces accessibility (A11y) by design. If you can't find it by Role, neither can a Screen Reader.

## 4. Success Metrics
*   **Performance**: Headless test instantiation < 10ms.
*   **Stability**: 0% flakiness on semantic queries (unlike Selenium/CSS selectors).

## 5. Acceptance Criteria (MVP)

### 5.1 Headless Harness
*   Must be able to instantiate an `App` in headless mode via `TestHarness::new(app)`.
*   Must *not* open a window.
*   Must pump the event loop manually via `.update()` or `.advance_frame()`.

### 5.2 Semantic Locators
*   Must support finding elements by `Role` (e.g., `Role::Button`).
*   Must support filtering by Label/Name (e.g., `.name("Submit")`).
*   **Example**: `app.find(role(Role::Button).name("Save"))`.

### 5.3 Interaction
*   Must support basic actions on found elements:
        *   `.click()`
        *   `.type_text("foo")`
    *   These actions must dispatch actual events to the `EventDispatcher`.

### 5.4 State Assertion
*   Must expose widget state:
        *   `.text()` -> String
        *   `.is_enabled()` -> bool

## 6. Technical Implementation Notes (Engineering)
*   **Dependency Structure**: `arthropod-test` currently depends only on `render-engine`. It will likely need `arthropod` (for App) and `widget-core` (for Role/Widget traits).
*   **Architecture**: The Harness should act as a "Virtual User" that injects events into the `EventLoop` or directly calls `App::update`.

## 7. Out of Scope (Phase 1)
*   Visual Regression (Screenshots) - Phase 2.
*   Fuzz Testing - Phase 3.
*   Time-travel / Undo-Redo testing.
