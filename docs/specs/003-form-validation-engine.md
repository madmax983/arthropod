# 🔭 Vantage: Spec for Form Validation Engine

**Status:** Draft
**Owner:** Vantage (Product Manager)
**Date:** 2024-05-25

## 1. The "User Story" 👤

> **"As an Enterprise Developer building complex data-entry applications with Arthropod, I want a unified form and validation engine, so that I can easily manage complex form state, perform dirty checking, and execute synchronous/asynchronous validation without writing boilerplate code."**

## 2. The "So What?" (Business Value) 💰

**Problem:**
Currently, developers building forms in Arthropod must manually wire up state for every input field, write custom validation logic, and manually track whether a form is "dirty" (modified) or "submitting". This leads to:
- **Massive Boilerplate:** Hundreds of lines of code just to manage basic text inputs and validation states.
- **Inconsistent UX:** Every developer implements validation differently. Some validate on blur, some on change, leading to a disjointed experience.
- **Poor Performance:** Naive implementations often trigger full form re-renders on every keystroke.
- **Enterprise Blocker:** Complex, multi-step forms are a core requirement for enterprise software. Without a robust solution, Arthropod cannot compete in the enterprise space.

**Solution:**
A dedicated Form Validation Engine built on top of `flux-state` that provides a declarative API for defining form schemas, validation rules, and submission handling, automatically managing pristine/dirty states and field-level error messages.

**Impact:**
- Drastically reduces boilerplate, speeding up application development.
- Provides a consistent, high-quality user experience out of the box.
- Unlocks the enterprise market by satisfying a critical baseline requirement.

## 3. Gap Analysis 🔍

| Feature | React Hook Form | Formik | Arthropod (Current) | Arthropod (Target) |
| :--- | :--- | :--- | :--- | :--- |
| **State Management** | Uncontrolled (Refs) | Controlled (State) | Ad-hoc (Manual Signals) | **Reactive (Signals)** |
| **Re-renders** | Field-level | Form-level | Ad-hoc | **Field-level** |
| **Async Validation** | Yes | Yes | Manual implementation | **Yes (Built-in)** |
| **Schema Validation** | Yes (Yup/Zod) | Yes (Yup) | Manual implementation | **Yes (Integration)** |

## 4. Acceptance Criteria ✅

To consider this feature "Done", the Engineering team must demonstrate:

1.  **State Management:**
    -   Must track `values`, `errors`, `touched`, `is_dirty`, and `is_submitting` states reactively.
    -   Typing in one field must *only* trigger reactive updates for that specific field and its associated error state, not the entire form.
2.  **Validation Triggers:**
    -   Must support configurable validation triggers: `onChange`, `onBlur`, and `onSubmit`.
3.  **Validation Types:**
    -   Must support synchronous validation functions.
    -   Must support asynchronous validation functions (e.g., checking if a username is taken via an API call) with loading states.
4.  **Integration:**
    -   Must integrate seamlessly with existing `widget-core` inputs (e.g., `TextInput`, `Checkbox`).
    -   Must integrate cleanly with `flux-state` to ensure minimal re-renders.

## 5. Out of Scope (Phase 1) 🚫

-   **Multi-step/Wizard Forms:** Phase 2. (Managing state across completely separate views).
-   **Auto-generating UI from Schemas:** Phase 2. (e.g., passing a JSON schema and getting a fully rendered form).
-   **Complex Array/Nested Fields:** Phase 2. (Dynamic lists of inputs).

## 6. Metrics for Success 📊

-   **Performance:** Validation execution time < 5ms for a form with 50 simple fields.
-   **DX Audit:** A developer can implement a login form with validation in under 20 lines of code.
