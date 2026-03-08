# 0037. Input Engine Unification

Date: 2026-03-08

## Status

Accepted

## Context

Input handling and form logic were previously distributed across multiple modules within `widget-core` (specifically `crate::input_logic` and `crate::form_logic`). As the framework evolved, keeping input-related logic within the `widget-core` crate began to cause coupling between widget state definitions and the algorithms processing input and form events. This logic often overlapped with responsibilities that correctly belong in the dedicated `input-engine` crate (established in ADR 0032).

Maintaining separate `input_logic` and `form_logic` modules inside `widget-core` meant that any unified input processing had to traverse crate boundaries unnecessarily or duplicate functionality that the `input-engine` was designed to handle (e.g., text entry, cursor movement, form validation, focus traversal).

## Decision

We have unified the input and form logic by migrating the contents of `widget-core`'s `input_logic` and `form_logic` modules into the `input-engine` crate.

1. **Remove Local Modules:** `crate::input_logic` and `crate::form_logic` were removed from `widget-core`.
2. **Delegate to Input Engine:** `WidgetContext` methods that handle input events (like `send_char`, `send_backspace`, `focus_next`) and form operations (like `revalidate_form`, `trigger_submit`) now delegate directly to the `input_engine` crate (e.g., `input_engine::text`, `input_engine::form`, `input_engine::focus`).

## Consequences

### Positive
- **Centralized Logic:** All core input and form processing logic is now centralized in the `input-engine` crate, reducing code duplication and strictly enforcing the "Input Fusion" specification boundaries.
- **Simplified Widget Core:** `widget-core` is simplified and strictly focuses on widget traits and Context state storage, delegating complex event processing.
- **Improved Cohesion:** Input logic is highly cohesive, living alongside gesture recognition and hit testing.

### Negative
- **Cross-Crate Delegation:** `WidgetContext` in `widget-core` must now delegate across crate boundaries to `input-engine` functions, adding a slight layer of indirection (though performance impact is negligible as these are direct function calls).
