# widget-core

Comprehensive widget library for the Arthropod GUI framework. Provides a declarative API for building cross-platform user interfaces with reactive state management.

## Features

- **15 Built-in Widgets** - Primitives, layouts, containers, and forms
- **Declarative Macros** - SwiftUI/Flutter-style widget DSL (`txt!`, `btn!`, `col!`, `row!`)
- **Reactive State** - Powered by `flux-state` signals for automatic UI updates
- **Type-Safe Composition** - Tuple-based children with zero runtime overhead
- **Layout Engine Integration** - Flexbox-based layout with gap and padding support
- **Theme Support** - Integrates with `theme-engine` for consistent styling

## Quick Start

```rust
use arthropod::prelude::*;
use widget_core::{btn, col, txt};

fn main() -> Result<(), AppError> {
    App::run("Hello Arthropod", 400, 300, |_ctx| {
        col!(
            [
                txt!("Welcome to Arthropod!", size: 24.0),
                btn!("Click Me", primary, on_click: || println!("Clicked!")),
            ],
            gap: 20.0,
            padding: 20.0
        )
    })
}
```

## Widget Categories

### Primitive Widgets

| Widget | Description | Example |
|--------|-------------|---------|
| **Text** | Static or reactive text display | `Text::new("Hello")` |
| **Button** | Clickable button with styles | `Button::new("OK").primary()` |
| **TextInput** | Single-line text input | `TextInput::new(signal).placeholder("Name")` |
| **Checkbox** | Boolean toggle with label | `Checkbox::new(signal).label("Agree")` |

### Layout Widgets

| Widget | Description | Example |
|--------|-------------|---------|
| **Row** | Horizontal layout | `Row::new((child1, child2)).gap(10.0)` |
| **Column** | Vertical layout | `Column::new((child1, child2)).gap(10.0)` |
| **Spacer** | Flexible or fixed spacing | `Spacer::flex()` or `Spacer::fixed(20.0)` |
| **Divider** | Visual separator line | `Divider::horizontal()` |
| **Padding** | Add space around content | `Padding::all(20.0, child)` |
| **Center** | Center content | `Center::new(child)` |

### Advanced Layouts

| Widget | Description | Example |
|--------|-------------|---------|
| **Stack** | Overlay with z-ordering | `Stack::new((bg, overlay, top))` |
| **Grid** | 2D grid layout | `Grid::new(children, 3 /* columns */)` |
| **List** | Dynamic content from iterators | `list_from(items.iter().map(...))` |

### Container Widgets

| Widget | Description | Example |
|--------|-------------|---------|
| **Card** | Themed container | `Card::new(content).padding(20.0)` |
| **Container** | Generic flexbox (legacy) | `Container::row(children).gap(10.0)` |

### Form Widgets

| Widget | Description | Example |
|--------|-------------|---------|
| **Form** | Validated form | `Form::new((("name", input),)).on_submit(...)` |

## Usage Patterns

### Macro Style (Recommended)

```rust
use widget_core::{txt, btn, col, row, Spacer};

// Justified toolbar
let toolbar = row!([
    btn!("Back"),
    Spacer::flex(),  // Pushes buttons to edges
    btn!("Save", primary),
    btn!("Share", primary),
], gap: 8.0);

// Nested layouts
let layout = col!([
    txt!("Settings", size: 20.0),
    row!([btn!("OK"), btn!("Cancel")], gap: 8.0),
], gap: 16.0, padding: 20.0);
```

### Builder Style

```rust
use widget_core::{Text, Button, Row, Column};

// Explicit construction
let row = Row::new((
    Button::new("Left"),
    Button::new("Center"),
    Button::new("Right"),
))
.gap(10.0)
.padding(12.0);
```

### Reactive State

Use `Computed` for derived reactive values:

```rust
use flux_state::{Runtime, Signal, Computed};
use widget_core::{Text, Button, Column};

let runtime = Runtime::new();
let counter = Signal::new(runtime.clone(), 0);
let (read, write) = counter.split();

// Computed value auto-updates when counter changes
let counter_text = Computed::new(runtime.clone(), move || {
    format!("Count: {}", read.get())
});

let ui = Column::new((
    Text::computed(counter_text),
    Button::new("Increment").on_click(move || {
        write.update(|n| *n + 1);
    }),
))
.gap(10.0);
```

**For simple reactive text without computation**, use `Text::reactive()`:

```rust
use flux_state::Signal;
use widget_core::Text;

let username = Signal::new(runtime.clone(), String::from("Alice"));
let (read, write) = username.split();

Text::reactive(read) // Displays the signal value directly
```

## Practical Examples

### Justified Layout

Push elements to opposite ends with `Spacer::flex()`:

```rust
use widget_core::{Row, Button, Spacer};

Row::new((
    Button::new("Left"),
    Spacer::flex(),  // Fills available space
    Button::new("Right"),
))
```

### Sectioned Layout

Use dividers to visually separate sections:

```rust
use widget_core::{Column, Text, Divider};

Column::new((
    Text::new("Section 1").size(16.0),
    Text::new("Content 1"),
    Divider::horizontal(),
    Text::new("Section 2").size(16.0),
    Text::new("Content 2"),
))
.gap(8.0)
```

### Centered Modal

```rust
use widget_core::{Center, Card, Column, Button};

Center::new(
    Card::new(
        Column::new((
            Text::new("Dialog Title"),
            Text::new("Content here"),
            Button::new("OK").primary(),
        ))
        .gap(12.0)
    )
    .padding(24.0)
)
```

### Image Gallery

```rust
use widget_core::{Grid, Button};

Grid::new(
    (
        Button::new("Img1"), Button::new("Img2"),
        Button::new("Img3"), Button::new("Img4"),
        Button::new("Img5"), Button::new("Img6"),
    ),
    3, // columns (creates 2 rows)
)
.gap(12.0)
```

### Dynamic List

```rust
use widget_core::{list_from, Row, Text, Button, Spacer};

let items = vec!["Apple", "Banana", "Cherry"];

let list = list_from(items.iter().map(|item| {
    Row::new((
        Text::new(format!("• {}", item)),
        Spacer::flex(),
        Button::new("View"),
    ))
    .gap(8.0)
}))
.gap(4.0);
```

### Form with Validation

```rust
use flux_state::{Runtime, Signal};
use widget_core::{Form, TextInput, form};

let runtime = Runtime::new();
let username = Signal::new(runtime.clone(), String::new());
let email = Signal::new(runtime.clone(), String::new());

let form = form!(
    [
        ("username", TextInput::new(username)
            .placeholder("Username")
            .validator(|s| if s.len() >= 3 { Ok(()) } else { Err("Too short".into()) })),
        ("email", TextInput::new(email)
            .placeholder("Email")
            .validator(|s| if s.contains('@') { Ok(()) } else { Err("Invalid".into()) })),
    ],
    on_submit: |data| {
        println!("Submitted: {:?}", data);
        Ok(())
    }
);
```

## Custom Widgets

Implement the `Widget` trait to create custom widgets:

```rust
use widget_core::{Widget, WidgetContext, Row, Text, Button};
use render_engine::NodeId;

struct UserCard {
    name: String,
    age: u32,
}

impl Widget for UserCard {
    fn build(&self, ctx: &mut WidgetContext) -> NodeId {
        Row::new((
            Text::new(format!("{}, {}", self.name, self.age)),
            Button::new("View Profile"),
        ))
        .gap(10.0)
        .build(ctx)
    }
}
```

## Widget Composition

Widgets accept children as tuples for type-safe, zero-overhead composition:

```rust
use widget_core::{Row, Text, Button};

// 2-tuple
Row::new((Text::new("A"), Button::new("B")))

// 3-tuple
Row::new((Text::new("A"), Text::new("B"), Text::new("C")))

// Nested tuples for many children
Row::new((
    Text::new("A"),
    Text::new("B"),
    (
        Text::new("C"),
        Text::new("D"),
        Text::new("E"),
    )
))
```

For runtime-dynamic content, use `List`:

```rust
use widget_core::{list_from, Text};

let items: Vec<_> = (0..100).collect();
let list = list_from(items.iter().map(|i| Text::new(format!("Item {}", i))));
```

## Integration with Arthropod

Widget-core integrates with:

- **flux-state** - Reactive signals for state management
- **layout-engine** - Flexbox layout computation
- **theme-engine** - Theming and design tokens
- **render-engine** - Scene graph and GPU rendering
- **arthropod-ecs** - ECS for widget state and reactivity

See `examples/widget_gallery.rs` for comprehensive demonstrations.

## Examples

Run examples to see widgets in action:

```bash
# Comprehensive widget showcase
cargo run --example widget_gallery

# Macro DSL demo
cargo run --example macro_demo

# Form with validation
cargo run --example form_gui

# Full app
cargo run --example hello_world
```

## API Documentation

Build and view the full API docs:

```bash
cargo doc --open -p widget-core
```

## License

This crate is part of the Arthropod project.
