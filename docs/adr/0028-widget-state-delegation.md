# ADR 0028: Widget State Delegation

**Status:** Accepted

**Date:** 2026-05-20

**Deciders:** Architecture Team

## Context

ADR 0014 (Flattened Widget Context) established `WidgetContext` as the central "God Object" for the widget build phase. It proposed organizing state into sub-context structs (`InputContext`, `LayoutContext`) owned by `WidgetContext`.

However, as the system grew, two issues emerged:
1.  **Wrapper Fatigue**: The intermediate sub-context structs (`InputContext`) added little value. They were often just thin wrappers around a `HashMap`, requiring extra boilerplate to delegate methods from the main context.
2.  **Logic Bloat**: The `impl WidgetContext` block grew to thousands of lines, mixing state storage with complex logic (e.g., focus navigation algorithms, form validation rules). This made unit testing difficult, as testing a navigation rule required constructing a full `WidgetContext` with all its dependencies.

## Decision

We have refined the `WidgetContext` architecture to separate **State Storage**, **State Definition**, and **Business Logic**.

### 1. Direct State Storage (Flattened)

We removed the intermediate sub-context structs. `WidgetContext` now directly holds the state containers:

```rust
pub struct WidgetContext {
    // Input
    pub(crate) text_input_states: IndexMap<NodeId, TextInputState>,

    // Form
    pub(crate) form_states: HashMap<NodeId, FormState>,
    pub(crate) validators: HashMap<NodeId, ValidationState>,

    // ... other state maps
}
```

### 2. Domain-Driven State Definitions

State data is defined in dedicated structs within domain-specific modules (e.g., `crate::input_state::TextInputState`, `crate::form_state::FormState`). These structs are pure data containers.

### 3. Logic Delegation (Functional Core)

Complex business logic is extracted into `*_logic` modules (e.g., `crate::input_logic`, `crate::form_logic`). These modules contain pure functions that operate on the state maps, rather than on the `WidgetContext` itself.

```rust
// In crate::input_logic
pub fn focus_next(
    states: &IndexMap<NodeId, TextInputState>,
    current_focus: &mut Option<NodeId>
) -> Option<NodeId> {
    // Implementation of focus cycling logic
}

// In WidgetContext
impl WidgetContext {
    pub fn focus_next(&mut self) -> Option<NodeId> {
        // Delegate to logic module
        crate::input_logic::focus_next(&self.text_input_states, &mut self.focused_node)
    }
}
```

## Consequences

### Positive

-   **Testability**: The logic modules (`input_logic`, `form_logic`) can be unit tested in isolation by passing simple `HashMap`s/`IndexMap`s, without needing a mocked `WidgetContext` or `Scene`.
-   **Maintainability**: `WidgetContext` remains large in terms of API surface (for ergonomics), but its implementation is trivial (delegation). The complexity is compartmentalized in logic modules.
-   **Performance**: Removing the sub-context wrapper layer reduces pointer indirection and allocation overhead.

### Negative

-   **Large Struct**: `WidgetContext` is still a large struct with many fields. However, since it is ephemeral (exists only during the build/event phase), this is acceptable.
-   **Facades**: `WidgetContext` methods must explicitly delegate to logic functions, creating some boilerplate facade code.

## References

-   ADR 0014: Flattened Widget Context
-   Modules: `widget_core::input_logic`, `widget_core::form_logic`
