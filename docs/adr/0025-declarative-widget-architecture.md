# ADR 0025: Declarative Widget Architecture

**Status:** Accepted

**Date:** 2026-02-21

**Deciders:** Architecture Team

## Context

Building user interfaces requires constructing a hierarchical tree of elements. In Rust, achieving a syntax that is both declarative (easy to read/write) and performant (type-safe, minimal overhead) is challenging due to the ownership model and strict typing.

We evaluated several approaches:
1.  **Immediate Mode (e.g., egui)**: Great for tools, but harder to style and animate for enterprise apps.
2.  **Builder Pattern (e.g., standard struct builders)**: Verbose for deep hierarchies (`.child(A).child(B)`).
3.  **Trait Objects (`Vec<Box<dyn Widget>>`)**: Flexible but incurs heap allocation and vtable dispatch overhead for every node.
4.  **Macro DSLs**: Can be powerful but hard to debug and tool.

We need an architecture that allows for:
-   **Composition**: Widgets composed of other widgets.
-   **Type Safety**: Compile-time checks for widget structure.
-   **Performance**: Minimizing allocations during the build phase.
-   **Integration**: Seamless connection with the underlying ECS and Scene Graph.

## Decision

We adopt a **Trait-Based Declarative Architecture** centered on the `Widget` trait and `WidgetTuple` system.

### 1. The `Widget` Trait

The core unit is the `Widget` trait, which defines how a UI element is instantiated into the Scene Graph and ECS.

```rust
pub trait Widget {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId;
}
```

-   **Input**: `&self` (immutable configuration) and `&mut WidgetContext` (mutable build state).
-   **Output**: `NodeId` (handle to the created scene node).
-   **Responsibility**: A widget's `build` method creates a node, configures its layout/style/components via the context, and returns the ID so the parent can attach it.

### 2. Composition via `WidgetTuple`

To enable static composition without `Vec<Box<dyn Widget>>`, we implement the `WidgetTuple` trait for Rust tuples `(A, B, C)` where each element implements `Widget`.

```rust
pub trait WidgetTuple {
    fn build_all(&self, ctx: &mut WidgetContext, parent: NodeId);
    // ...
}

impl<A: Widget, B: Widget> WidgetTuple for (A, B) { ... }
```

This allows container widgets to accept generic `C: WidgetTuple` arguments, enabling syntax like:

```rust
Row::new((
    Text::new("Label"),
    Button::new("Click Me"),
    Spacer::new(),
))
```

This approach relies on **Monomorphization**: the compiler generates a specialized version of `Row` for the specific tuple type `(Text, Button, Spacer)`.

### 3. Named Tuples for Forms

For widgets that require named slots (like Forms mapping fields to data), we introduce `NamedWidgetTuple`.

```rust
Form::new((
    ("username", TextInput::new(user_signal)),
    ("password", TextInput::new(pass_signal)),
))
```

### 4. Build-Time Context

The `WidgetContext` acts as the accumulator for all widget state during the recursive build process. It abstracts the complexity of the underlying `Scene` and ECS, providing high-level methods like `ctx.set_layout_style()` or `ctx.add_clickable()`.

## Consequences

### Positive

-   **Performance**: Static dispatch (no vtables) for the entire widget tree.
-   **Zero Allocation Composition**: Tuples are allocated on the stack (mostly) or optimized away by the compiler.
-   **Type Safety**: The structure of the UI is known at compile time.
-   **Flexibility**: Custom widgets are just structs implementing `Widget`.

### Negative

-   **Compile Times**: Deeply nested generic types (e.g., `Row<(Col<(Text, Button)>, Text)>`) can increase compilation time.
-   **Binary Size**: Monomorphization can duplicate code for `Row<A>` vs `Row<B>`, though usually negligible for UI code.
-   **Recursion Limits**: Extremely deep trees might hit generic recursion limits (rare in practice).

## References

-   `crates/widget-core/src/widget_trait.rs`: Definition of `Widget` and `WidgetTuple`.
-   `crates/widget-core/src/context/mod.rs`: Implementation of `WidgetContext`.
-   ADR 0012: Widget Macro DSL (built on top of this architecture).
