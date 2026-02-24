# Forge's Journal

This journal records critical learnings, recurring anti-patterns, and architectural insights discovered during refactoring sessions.

## Critical Learnings

### Project-Specific Clippy Allowances
**Learning:** `clippy::collapsible_if` is explicitly allowed in `arthropod` to avoid unstable `let_chains` syntax or complex nested `if let` chains, likely to support stable Rust or project style.
**Action:** Respect `#[allow(clippy::collapsible_if)]` directives and do not refactor them away unless `let_chains` becomes stable/allowed or the logic can be simplified without it.

**[WidgetContext is a God Object]**
**Learning:** `WidgetContext` in `widget-core` handles too many responsibilities (layout, input, painting, validation).
**Action:** Extract logic into the state structs it manages (e.g., `TextInputState`, `FormState`) to improve encapsulation.

**[Inefficient Collection Usage]**
**Learning:** `widget-core` occasionally copies `HashMap` keys into `Vec` for iteration, leading to unnecessary allocations.
**Action:** Use `IndexMap` features like `get_index_of` or iterator chaining to avoid intermediate allocations.

**[String Traversal Performance]**
**Learning:** Helper functions often iterate strings multiple times (e.g., `nth()` followed by `count()`), leading to O(N) where O(1) or single-pass O(N) is possible.
**Action:** Consolidate string traversals into single-pass loops or use iterator state more effectively.

**[Runtime::recompute Complexity]**
**Learning:** `Runtime::recompute` in `flux-state` was a complex function mixing synchronization, recursion checking, and state updates, making it hard to follow.
**Action:** Extracted `check_recursion`, `finish_computation`, and `ComputingGuard` to improve readability and separation of concerns.
