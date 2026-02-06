# Forge's Journal

This journal records critical learnings, recurring anti-patterns, and architectural insights discovered during refactoring sessions.

## Critical Learnings

### Project-Specific Clippy Allowances
**Learning:** `clippy::collapsible_if` is explicitly allowed in `arthropod` to avoid unstable `let_chains` syntax or complex nested `if let` chains, likely to support stable Rust or project style.
**Action:** Respect `#[allow(clippy::collapsible_if)]` directives and do not refactor them away unless `let_chains` becomes stable/allowed or the logic can be simplified without it.
**[WidgetContext is a God Object]**
**Learning:** `WidgetContext` in `widget-core` handles too many responsibilities (layout, input, painting, validation).
**Action:** Extract logic into the state structs it manages (e.g., `TextInputState`, `FormState`) to improve encapsulation.
