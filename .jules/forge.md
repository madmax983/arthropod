# Forge's Journal

**[Refactoring MCP Server Tool Execution]**
**Learning:** MCP tool handlers often repeat the same pattern: lock context, serialize params, execute tool, format output.
**Action:** Extracted this logic into a generic `execute_tool_core<T, P>` helper method in `ArthropodServer`. This reduced boilerplate significantly and centralized error handling and context management. Used `Result<String, McpError>` as return type to allow flexible output formatting (e.g., prepending banners).
**[Refactoring Nested `if let` with `is_some_and` or Guard Clauses]**
**Learning:** A recurring pattern in this repository was nesting `if let` blocks resulting in "pyramids of doom" that required `#![allow(clippy::collapsible_if)]`. When trying to flatten these, you can often chain methods such as `.filter()` or `.is_some_and(|node| matches!(...))` to collapse conditions. In some cases, `and_then` can help avoid multiple layers. Careful use of `.map(...)` followed by an `if let Some(...)` match simplifies matching on nested enum variants without triggering the borrow checker unnecessarily.
**Action:** Default to using `.filter()`, `.and_then()`, and guard clauses instead of suppressing the `clippy::collapsible_if` lint to maintain clean structure.

**[Extracting Logic that Mutates Option<&mut T>]**
**Learning:** When extracting logic into helper functions that operate on optional mutable references (`Option<&mut T>`) shared from a larger context struct, you cannot just move the reference into the `Some` variant (e.g., `Some(ctx.tessellation_cache)`). This causes `E0507: cannot move out of borrowed content` because `&mut T` is not `Copy`.
**Action:** Always reborrow the mutable reference explicitly when wrapping it in `Some` to pass to a helper function, e.g., `Some(&mut *ctx.tessellation_cache)`.
**[Refactoring Pyramids of Doom]**\n**Learning:** The clippy lint `collapsible_if` catches nested `if` and `if let` blocks. Instead of suppressing the lint with `#[allow(clippy::collapsible_if)]`, you can safely flatten these using let-chaining (`&& let`), iterator chaining (`.and_then()`, `.filter()`, `is_some_and()`), or match guards.\n**Action:** Avoid `#[allow(clippy::collapsible_if)]` and instead restructure logic into a single condition line using let-chaining or combinators.

**[Borrow Checker vs Nested if let Flattening]**
**Learning:** Flattening `if let` blocks or nested `Option` chains (e.g., in `scene.get_mut()`) can sometimes extend the lifespan of a mutable borrow unintentionally. In `particles.rs` and `widget.rs`, attempting to do `if let Some(...) = scene.get_mut(...)` and then calling `scene.mark_dirty(...)` inside the same block caused `cannot borrow *scene as mutable more than once at a time`. The original code used a boolean flag (`marked`) to explicitly end the mutable borrow scope of `scene` *before* calling the second mutable method.
**Action:** Be extremely cautious when refactoring code that performs multiple mutable borrows on the same struct (like `Scene`). A guard clause or boolean flag to isolate the borrow scope is preferable to collapsing the structure if it violates borrow rules.

**[Unstable let chains]**
**Learning:** Rust `let chains` (`if let ... && let ...`) are currently an unstable, nightly-only feature (tracked in #53667). While they look elegant and successfully avoid the clippy lint `collapsible_if`, using them on standard Rust will immediately break the build.
**Action:** Do not use `&& let` to flatten `if let` blocks in stable Rust projects. Instead, use nested `if let`, `.and_then()`, or boolean condition early returns (`guard clauses`) when feasible, or accept the `#[allow(clippy::collapsible_if)]` if there is no clearer alternative.
