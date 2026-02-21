# Flux DevTools: Value Inspector (Spec 007)

**Status:** Draft
**Owner:** Vantage (Product Manager)

## 1. Executive Summary

Debugging reactive applications is difficult because the state is often distributed and implicit. While `flux-devtools` currently visualizes the dependency graph, it lacks the ability to inspect the *values* flowing through that graph. This feature aims to expose current signal and computed values directly in the DevTools UI, significantly reducing the "time to diagnose" for state-related bugs.

## 2. User Story

> **As a** Developer building complex UIs with Arthropod,
> **I want to** see the current value of any Signal or Computed node in the DevTools inspector,
> **So that** I can verify my application state without adding temporary `println!` statements or halting execution.

## 3. Goals & Non-Goals

### ✅ Goals
- Display string representations of values for `Signal` and `Computed` nodes in the DevTools TUI.
- Support standard Rust types (integers, floats, strings, bools) out of the box.
- Provide a mechanism for custom types to be inspectable.
- Handle large values gracefully (truncation in list view, full view in details panel).
- Ensure zero runtime overhead when the feature is disabled (e.g., in release builds without the `devtools` feature).

### 🚫 Non-Goals
- Editing values from the DevTools (Read-only for now).
- Historical value tracking (Time-travel debugging is a separate spec: 002).
- Visualizing complex data structures like graphs or images (Text representation only).

## 4. Acceptance Criteria

### AC1: Value Reflection
- The `flux-state` runtime must expose the current value of a node as a string.
- **Constraint:** If a type does not support inspection or the feature is disabled, it should fallback to a placeholder like `<opaque>` or `<hidden>`.

### AC2: Snapshot Data
- The debug snapshot data used by DevTools must include the value for each node.
- This data should be populated during the inspection call.

### AC3: UI Display
- The DevTools TUI "Details Panel" must have a new section: **Current Value**.
- If the value is present, display it.
- If the value is multiline, preserve formatting.
- If the value is very long (> 1000 chars), truncate it with a "..." suffix, but allow expanding or scrolling.

### AC4: Performance & Safety
- **Constraint:** Inspecting values must not panic, even if the value is being accessed by another thread.
- **Constraint:** Generating the string representation should only happen when the inspector is actively requesting data, not on every update.

## 5. Non-Functional Requirements

- **Overhead:** The feature must have zero performance impact when disabled (e.g., via compilation flags).
- **Thread Safety:** Value inspection must be thread-safe and avoid deadlocks.
- **Scalability:** The inspector must handle graphs with thousands of nodes without lag.

## 6. Metrics for Success

- **Debug Latency:** Developers should be able to identify a wrong state value in < 5 seconds using DevTools.
- **Adoption:** 80% of "complex" bug reports (involving state) should benefit from DevTools screenshots showing the state.
