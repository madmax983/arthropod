# ADR 0012: Widget Macro DSL with Derive Macros

**Status:** Accepted

**Date:** 2026-01-19

**Deciders:** Architecture Team

## Context

Arthropod needs a declarative widget API that:

1. **Ergonomic**: Concise macro syntax for common patterns (like SwiftUI/Flutter/React)
2. **Type-Safe**: Compile-time validation of widget composition
3. **Zero-Overhead**: No vtable dispatch for widget trees (unlike `Box<dyn Widget>`)
4. **Extensible**: Easy to add new widget macros as the library grows
5. **Maintainable**: Avoid hand-writing repetitive macro_rules! for each widget

Challenges:
- Different widget categories have different patterns (leaf vs container vs form)
- Builder patterns with generics (like `Container<C>`) change types on each method call
- Children can be static (tuples) or dynamic (runtime loops)
- Named children (forms, scaffolds) need string-to-widget mapping

## Decision

We implement a **derive macro system** that automatically generates declarative macros from widget struct definitions, using **tuple-based composition** for zero-overhead type safety.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  Developer API                                                   │
│                                                                  │
│  col!([txt!("Hello"), btn!("OK")], gap: 10.0)                   │
│  form!([("name", TextInput::new(sig))], on_submit: |d| {...})   │
└──────────────────────────────┬──────────────────────────────────┘
                               │ macro expansion
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│  Generated Macros (widget-macros)                               │
│                                                                  │
│  txt! → Text::new("Hello")                                      │
│  btn! → Button::new("OK")                                       │
│  col! → Container::column((child1, child2))                     │
│  form! → Form::new((("name", widget),))                         │
└──────────────────────────────┬──────────────────────────────────┘
                               │ uses
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│  Widget Structs with Derives (widget-core)                      │
│                                                                  │
│  #[derive(Widget)]                               │
│  #[widget(name = "txt")]                                        │
│  pub struct Text { ... }                                        │
│                                                                  │
│  #[derive(Widget)]                               │
│  #[widget(name = "col", constructor = "column")]                │
│  pub struct Container<C: WidgetTuple> { ... }                   │
└──────────────────────────────┬──────────────────────────────────┘
                               │ trait bounds
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│  Tuple Traits (widget-core)                                     │
│                                                                  │
│  trait WidgetTuple { fn build_all(&self, ctx, parent); }        │
│  trait NamedWidgetTuple { fn build_all_named(...); }            │
│                                                                  │
│  impl WidgetTuple for (A,) where A: Widget                      │
│  impl WidgetTuple for (A, B) where A: Widget, B: Widget         │
│  impl<W: Widget> WidgetTuple for W  // Single widget shorthand  │
└─────────────────────────────────────────────────────────────────┘
```

### Key Design: Tuple-Based Composition

Instead of `Box<dyn Widget>` (vtable overhead, heap allocation), we use **tuples with trait bounds**:

```rust
// OLD: Dynamic dispatch, heap allocation
pub struct Container {
    children: Vec<Box<dyn Widget>>,  // Runtime polymorphism
}

// NEW: Static dispatch, zero overhead
pub struct Container<C: WidgetTuple> {
    children: C,  // Compile-time polymorphism
}

// Tuple implementations provide build_all
impl<A: Widget, B: Widget> WidgetTuple for (A, B) {
    fn build_all(&self, ctx: &mut WidgetContext, parent: NodeId) {
        let id0 = self.0.build(ctx);
        ctx.reparent_to(id0, parent);
        let id1 = self.1.build(ctx);
        ctx.reparent_to(id1, parent);
    }
}

// Single widget shorthand (no tuple wrapper needed)
impl<W: Widget> WidgetTuple for W {
    fn build_all(&self, ctx: &mut WidgetContext, parent: NodeId) {
        let id = self.build(ctx);
        ctx.reparent_to(id, parent);
    }
}
```

### Critical Insight: One-Shot Tuple Construction

Macros must construct tuples in a **single expression**, not via builder loops:

```rust
// WRONG: Type changes on each .child() call - cannot iterate
let mut widget = Container::column();  // Container<()>
widget = widget.child(a);               // Container<(A,)>
widget = widget.child(b);               // Container<(A, B)> - TYPE CHANGED!

// CORRECT: One-shot tuple construction
Container::column((a, b))  // Container<(A, B)> from the start
```

This is why the generated macros use this pattern:

```rust
macro_rules! col {
    ([$($children:expr),* $(,)?]) => {{
        $crate::Container::column(($($children,)*))  // Tuple literal
    }};
}
```

### Derive Macro Attributes

**Struct-Level:**

| Attribute | Description | Example |
|-----------|-------------|---------|
| `#[widget(name = "x")]` | Macro name | `name = "txt"` |
| `#[widget(alias = "y")]` | Macro alias | `alias = "text"` |
| `#[widget(constructor = "x")]` | Constructor method | `constructor = "column"` |

**Field-Level:**

| Attribute | Description | Example |
|-----------|-------------|---------|
| `#[positional]` | First positional argument | `#[positional] text: String` |
| `#[positional(reactive)]` | With `@signal` support | `#[positional(reactive)] content: TextContent` |
| `#[widget_children]` | Tuple children | `#[widget_children] children: C` |
| `#[named_children]` | Named tuple children | `#[named_children] fields: F` |
| `#[param]` | Named parameter | `#[param] gap: f32` |
| `#[param(default = X)]` | With default | `#[param(default = 16.0)] size: f32` |
| `#[param(setter = "x")]` | Custom setter | `#[param(setter = "size")] font_size: f32` |
| `#[flag]` | Boolean flag | `#[flag] disabled: bool` |
| `#[callback]` | Callback parameter | `#[callback] on_click: Option<...>` |

### Generated Macro Examples

**Display Widget (Text):**

```rust
#[derive(Widget)]
#[widget(name = "txt", alias = "text")]
pub struct Text {
    #[positional(reactive)]
    content: TextContent,

    #[param(default = 16.0, setter = "size")]
    font_size: f32,

    #[param]
    color: Vec4,
}

// Generated:
txt!("Hello")                           // Text::new("Hello")
txt!("Title", size: 24.0)               // Text::new("Title").size(24.0)
txt!(@signal, size: 20.0)               // Text::reactive(signal).size(20.0)
```

**Container Widget:**

```rust
#[derive(Widget)]
#[widget(name = "col", alias = "column", constructor = "column")]
pub struct Container<C: WidgetTuple> {
    #[widget_children]
    children: C,

    #[param(default = 0.0)]
    gap: f32,

    #[param(default = 0.0)]
    padding: f32,
}

// Generated:
col!([child1, child2])                  // Container::column((child1, child2))
col!([child1, child2], gap: 10.0)       // Container::column((child1, child2)).gap(10.0)
```

**Form Widget (Named Children):**

```rust
#[derive(Widget)]
#[widget(name = "form")]
pub struct Form<F: NamedWidgetTuple> {
    #[named_children]
    fields: F,

    #[param(default = 12.0)]
    gap: f32,

    #[callback]
    on_submit: Option<SubmitCallback>,
}

// Generated:
form!([
    ("name", TextInput::new(sig1)),
    ("email", TextInput::new(sig2)),
], gap: 16.0, on_submit: |data| {...})
```

### Crate Structure

```
crates/
├── widget-core/           # Widget implementations
│   ├── src/
│   │   ├── widget_trait.rs   # Widget, WidgetTuple, NamedWidgetTuple
│   │   ├── container.rs      # Container<C: WidgetTuple>
│   │   ├── form.rs           # Form<F: NamedWidgetTuple>
│   │   ├── text.rs           # Text (with derive)
│   │   └── button.rs         # Button (with derive)
│   └── Cargo.toml
│
└── widget-macros/         # Proc-macro crate
    ├── src/
    │   ├── lib.rs            # Proc-macro entry points
    │   ├── widget.rs         # WidgetMacro derive implementation
    │   ├── widget_enum.rs    # WidgetEnum derive for flag enums
    │   ├── parse.rs          # Attribute parsing
    │   └── generate.rs       # Macro code generation
    └── Cargo.toml
```

## Rationale

### Why Tuples Instead of Vec<Box<dyn Widget>>?

**Performance:**
- No heap allocation per child
- No vtable dispatch
- Compiler can inline widget builds

**Type Safety:**
- Compile-time verification of widget composition
- IDE autocomplete for child types
- Exhaustive pattern matching possible

**Trade-offs:**
- Tuple limit (12 children max for static composition)
- Dynamic content (loops) requires scene-level manipulation

### Why Derive Macros?

**Maintainability:**
- Adding new widget = adding struct with derives
- No hand-written macro_rules! per widget
- Single source of truth for widget definition

**Consistency:**
- All widgets follow same macro patterns
- Attribute names are standardized
- Error messages are uniform

### Why Not JSX-like Syntax?

Considered using a proc-macro for JSX-like syntax:

```rust
// Rejected approach
html! {
    <Column gap=10>
        <Text>"Hello"</Text>
        <Button primary>"OK"</Button>
    </Column>
}
```

**Rejected because:**
- Requires proc-macro (slower compilation)
- Less IDE support (syntax highlighting, autocomplete)
- Doesn't feel like idiomatic Rust
- Our macro_rules! approach is fast and well-supported

### Dynamic Content Pattern

For truly dynamic content (loops with unknown count), use scene-level manipulation:

```rust
// Create container manually for dynamic children
let column_id = ctx.create_node(ctx.root(), NodeContent::Empty);
ctx.set_layout_style(column_id, FlexStyle { direction: Column, .. });

// Build children in loop
for item in items {
    let row = Container::row((
        Text::new(item.name),
        Button::new("Delete"),
    ));
    let row_id = row.build(&mut ctx);
    ctx.reparent_to(row_id, column_id);
}
```

This is a deliberate escape hatch - most UI is static structure, dynamic loops are the exception.

## Consequences

### Positive

- **Zero-Overhead Abstraction**: Tuple-based composition has no runtime cost
- **Ergonomic API**: `col!([...], gap: 10)` is concise and readable
- **Type Safety**: Compiler catches widget composition errors
- **Maintainable**: New widgets just need struct + derives
- **Fast Compilation**: macro_rules! is faster than proc-macros for usage sites

### Negative

- **Learning Curve**: Developers need to understand tuple limits
- **Tuple Size Limit**: Max 12 static children (can extend if needed)
- **Dynamic Content**: Requires different pattern (scene-level)
- **Error Messages**: Proc-macro errors can be cryptic

### Mitigations

- **Documentation**: Clear examples for static vs dynamic patterns
- **Error Handling**: Custom error messages in derive macros
- **Escape Hatches**: Scene-level API for dynamic content
- **Tuple Extensions**: Can add more tuple impls if 12 is insufficient

## Implementation Phases

### Phase 1: Core Infrastructure (Complete)
- [x] `widget-macros` crate with proc-macro structure
- [x] `WidgetMacro` derive macro
- [x] `WidgetEnum` derive for flag enums
- [x] `WidgetTuple` and `NamedWidgetTuple` traits
- [x] Tuple implementations up to 12 elements

### Phase 2: Widget Integration (Complete)
- [x] Apply derives to Text, Button widgets
- [x] Container with tuple-based children
- [x] Form with named tuple children
- [x] Manual macros for col!, row!, form!

### Phase 3: Polish (Current)
- [x] Single-widget WidgetTuple impl (no tuple wrapper needed)
- [x] One-shot tuple construction in generated macros
- [x] Update examples to new API
- [x] Documentation and ADR

### Future Enhancements
- [ ] TextInput derive macro
- [ ] Scaffold pattern for named slots
- [ ] Visual debugging for widget trees
- [ ] IDE plugin for macro expansion preview

## Performance Characteristics

- **Build Time**: ~0 overhead from macro expansion (compile-time)
- **Runtime**: Zero vtable dispatch, zero heap allocation for static trees
- **Memory**: Widgets stored inline in parent structs (stack or parent allocation)

## Testing Strategy

**Unit Tests:**
- Derive macro attribute parsing
- Code generation for all widget patterns
- Tuple trait implementations

**Integration Tests:**
- Widget composition examples compile
- Built widget trees have correct structure
- Forms validate and submit correctly

**Example Applications:**
- `macro_demo.rs`: Showcases all macro patterns
- `widget_gallery.rs`: All widget types with tuple API
- `form_demo.rs`: Form validation flow

## References

- **ADR 0001**: Hybrid ECS Architecture (Scene tree integration)
- **ADR 0003**: Flux State Reactive Model (Signal integration)
- **SwiftUI**: Inspiration for declarative widget syntax
- **Flutter**: Widget composition patterns
- **Rust Macros**: The Little Book of Rust Macros
