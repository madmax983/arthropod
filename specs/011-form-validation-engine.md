# 🔭 Vantage: Spec for Form & Validation Engine

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
Enterprise applications are essentially giant state machines built around data collection.
Currently, building a form in Arthropod requires developers to manually wire up `Signal<String>` for every text input, manually write dirty-checking logic, manually write validation functions that trigger on change, and manually manage the state of the "Submit" button based on those validations.
This leads to:
1.  **Boilerplate Mountain**: A simple login form requires 100+ lines of reactive wiring.
2.  **Inconsistent Validation**: Developers reinvent validation logic (regex checks, async API calls) differently for every form.
3.  **Fragile State**: Managing `is_dirty`, `is_touched`, and `is_submitting` states manually often leads to bugs where users can submit invalid data.

## 2. User Story
**As a** User,
**I want to** see clear, immediate feedback if I type an invalid email address or a password that is too short,
**So that** I don't waste time clicking "Submit" only to get a cryptic server error.

**As a** Developer,
**I want to** bind a data struct directly to a form container and declare validation rules (like `#[validate(email)]`),
**So that** the framework handles two-way binding, dirty checking, and error messages automatically.

## 3. The "So What?" (Business Value)
*   **Developer Velocity**: Forms are the most common UI element in B2B software. Reducing form boilerplate by 80% drastically speeds up feature delivery.
*   **Data Integrity**: A centralized, declarative validation engine guarantees that invalid data never leaves the client, reducing server load and preventing corrupted databases.
*   **Standardized UX**: Users get a consistent experience (e.g., fields turning red on blur if invalid) across the entire application without developers having to remember to code it.

## 4. Success Metrics
*   **Boilerplate Reduction**: A form with 10 fields, 3 conditional rules, and async validation must take < 50 lines of framework code.
*   **Performance**: Typing in a field with 5 complex validation rules must maintain < 16ms input latency (validation logic must not block the main thread).
*   **Reliability**: `FormState::is_valid()` must always be 100% synchronized with the underlying input signals.

## 5. Acceptance Criteria (MVP -> Production)

### 5.1 Form Container & State Management
*   The system **must** provide a `Form` context or widget that groups multiple input fields.
*   The `Form` **must** automatically aggregate the state of its children and provide an overarching `FormState` object containing:
    *   `is_dirty`: True if any field has been modified from its initial value.
    *   `is_valid`: True if all synchronous and asynchronous validations pass.
    *   `is_submitting`: True while the submit handler is executing.

### 5.2 Declarative Validation Rules
*   Developers **must** be able to attach standard validation rules to input fields (e.g., `Required`, `MinLength(8)`, `Email`, `Regex(...)`).
*   The system **must** support custom, user-defined validation functions that return a `Result<(), String>` (where the `String` is the error message).

### 5.3 Asynchronous Validation
*   The engine **must** support asynchronous validation (e.g., checking if a username is taken via an API call).
*   Async validators **must** be cancellable and debounced automatically to prevent race conditions and excessive network requests while typing.
*   The field state **must** expose an `is_validating` flag to show loading spinners on the UI.

### 5.4 Error Message Handling
*   Input widgets **must** be able to easily consume and display their own validation errors (e.g., via a `.error_text()` signal).
*   The framework **should** support configurable validation triggers (e.g., `OnChange`, `OnBlur`, `OnSubmit`).

## 6. Out of Scope (Phase 1)
*   **Auto-generation of UI**: Automatically generating the visual layout of a form from a JSON Schema or Rust Struct (Phase 2).
*   **Cross-Field Validation**: Complex validation rules that depend on the state of *other* fields (e.g., "Password Confirmation must match Password") (Phase 2).
