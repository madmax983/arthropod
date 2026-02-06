# Widget Basics Tutorial

Learn how to build user interfaces with Arthropod's widget system. This tutorial covers the fundamentals of widget composition, layout, and reactive state.

## Table of Contents

1. [Your First Widget](#your-first-widget)
2. [Layout Basics](#layout-basics)
3. [Spacing and Padding](#spacing-and-padding)
4. [Practical Patterns](#practical-patterns)
5. [Reactive State](#reactive-state)
6. [Forms and Validation](#forms-and-validation)
7. [Dynamic Content](#dynamic-content)
8. [Next Steps](#next-steps)

## Your First Widget

The simplest Arthropod app displays text in a window:

```rust
use arthropod::prelude::*;
use widget_core::{txt, col};

fn main() -> Result<(), AppError> {
    App::run("My First App", 400, 300, |_ctx| {
        txt!("Hello, Arthropod!")
    })
}
```

**What's happening:**
- `App::run()` creates a window and event loop
- The closure returns a widget tree
- `txt!()` macro creates a `Text` widget
- Arthropod renders the widget to the screen

## Layout Basics

### Vertical Layouts (Column)

Stack widgets vertically:

```rust
use widget_core::{col, txt, btn};

col!([
    txt!("Title", size: 24.0),
    txt!("Subtitle", size: 16.0),
    btn!("Action", primary),
], gap: 12.0)
```

**Key concepts:**
- `col!` arranges children top-to-bottom
- `gap` adds spacing between children
- Children are specified as a tuple `[child1, child2, ...]`

### Horizontal Layouts (Row)

Arrange widgets side-by-side:

```rust
use widget_core::{row, btn};

row!([
    btn!("OK", primary),
    btn!("Cancel", secondary),
], gap: 8.0)
```

**Key concepts:**
- `row!` arranges children left-to-right
- Use for toolbars, button groups, inline content

### Nesting Layouts

Combine rows and columns for complex layouts:

```rust
use widget_core::{col, row, txt, btn};

col!([
    txt!("Settings", size: 20.0),
    row!([
        btn!("Save", primary),
        btn!("Cancel", secondary),
        btn!("Reset"),
    ], gap: 8.0),
    txt!("Footer"),
], gap: 16.0, padding: 20.0)
```

**Pattern:**
- Outer `col!` provides vertical structure
- Inner `row!` groups related horizontal items
- `padding: 20.0` adds space around the entire layout

## Spacing and Padding

### Gap Between Children

Use `gap` to add spacing between widgets:

```rust
use widget_core::{col, btn};

col!([
    btn!("One"),
    btn!("Two"),
    btn!("Three"),
], gap: 10.0)  // 10px between each button
```

### Padding Around Content

Use `padding` for breathing room:

```rust
use widget_core::{col, txt};

col!([
    txt!("Padded Content"),
], padding: 20.0)  // 20px on all sides
```

### Padding Widget

For more control, use the `Padding` widget:

```rust
use widget_core::{Padding, Button};

// Uniform padding
Padding::all(20.0, Button::new("Click Me"))

// Horizontal and vertical
Padding::symmetric(40.0, 20.0, Button::new("Click Me"))  // h: 40, v: 20

// Custom per side
Padding::new(Button::new("Click Me"))
    .left(10.0)
    .right(20.0)
    .top(5.0)
    .bottom(15.0)
```

### Spacer Widget

Create flexible space to push widgets apart:

```rust
use widget_core::{row, btn, Spacer};

row!([
    btn!("Left"),
    Spacer::flex(),      // Fills available space
    btn!("Right"),
])
```

**Result:** "Left" button on left edge, "Right" button on right edge.

## Practical Patterns

### Justified Toolbar

Push items to opposite ends:

```rust
use widget_core::{row, btn, Spacer};

row!([
    btn!("Back"),
    Spacer::flex(),
    btn!("Save", primary),
    btn!("Share", primary),
], gap: 8.0, padding: 12.0)
```

### Centered Content

```rust
use widget_core::{Center, txt};

Center::new(txt!("Centered!", size: 24.0))
```

### Sectioned Layout

Use dividers to separate sections:

```rust
use widget_core::{col, txt, Divider};

col!([
    txt!("Section 1", size: 18.0),
    txt!("Content for section 1"),
    Divider::horizontal(),
    txt!("Section 2", size: 18.0),
    txt!("Content for section 2"),
], gap: 12.0, padding: 16.0)
```

### Card-Based Layout

Group related content in themed cards:

```rust
use widget_core::{Card, Column, Text, Button};

Card::new(
    Column::new((
        Text::new("Card Title").size(18.0),
        Text::new("Card content goes here."),
        Button::new("Action").primary(),
    ))
    .gap(8.0)
)
.padding(20.0)
```

### Grid Layout

Arrange items in a grid:

```rust
use widget_core::{Grid, Button};

Grid::new(
    (
        Button::new("1"), Button::new("2"), Button::new("3"),
        Button::new("4"), Button::new("5"), Button::new("6"),
    ),
    3, // columns (creates 2 rows)
)
.gap(10.0)
.padding(16.0)
```

### Stack for Overlays

Layer widgets on top of each other:

```rust
use widget_core::{Stack, Padding, Center, Row, Text, Spacer};

Stack::new((
    // Background
    Padding::all(100.0, Text::new("")),
    // Middle layer - centered text
    Center::new(Text::new("Overlay").size(24.0)),
    // Top layer - notification badge
    Row::new((
        Spacer::flex(),
        Text::new("🔔 3").size(12.0),
    )),
))
```

## Reactive State

### Signals for State Management

Use `Signal` for reactive values that automatically update the UI:

```rust
use arthropod::prelude::*;
use flux_state::{Runtime, Signal};
use widget_core::{col, txt, btn};

fn main() -> Result<(), AppError> {
    App::run("Counter", 300, 200, |ctx| {
        let counter = ctx.signal(0);
        let (read, write) = counter.split();

        col!([
            // Reactive text - updates when counter changes
            Text::reactive(read.map(|n| format!("Count: {}", n))),

            // Button updates the signal
            btn!("Increment", primary, on_click: move || {
                write.update(|n| n + 1);
            }),
        ], gap: 10.0)
    })
}
```

**Key concepts:**
- `ctx.signal(value)` creates a reactive signal
- `split()` separates into read and write handles
- `Text::reactive()` subscribes to signal changes
- `write.update()` modifies the value (triggers UI update)

### Multiple Reactive Widgets

Signals can drive multiple widgets:

```rust
use flux_state::{Runtime, Signal};
use widget_core::{Column, Text, Button, Row};

let runtime = Runtime::new();
let counter = Signal::new(runtime.clone(), 0);
let (read, write) = counter.split();

Column::new((
    // Both subscribe to the same signal
    Text::reactive(read.map(|n| format!("Count: {}", n))),
    Text::reactive(read.map(|n| format!("Double: {}", n * 2))),

    Row::new((
        Button::new("+1").on_click({
            let write = write.clone();
            move || write.update(|n| n + 1)
        }),
        Button::new("Reset").on_click(move || write.set(0)),
    )).gap(8.0),
))
.gap(10.0)
```

## Forms and Validation

### Basic Form

```rust
use flux_state::{Runtime, Signal};
use widget_core::{form, TextInput};

let runtime = Runtime::new();
let username = Signal::new(runtime.clone(), String::new());
let password = Signal::new(runtime.clone(), String::new());

let form = form!(
    [
        ("username", TextInput::new(username).placeholder("Username")),
        ("password", TextInput::new(password).placeholder("Password")),
    ],
    on_submit: |data| {
        println!("Login: {:?}", data);
        Ok(())
    }
);
```

### Form with Validation

Add validators to ensure data quality:

```rust
use flux_state::{Runtime, Signal};
use widget_core::{form, TextInput};

let runtime = Runtime::new();
let email = Signal::new(runtime.clone(), String::new());

let form = form!(
    [
        ("email", TextInput::new(email)
            .placeholder("your.email@example.com")
            .validator(|s| {
                if s.is_empty() {
                    Err("Email is required".into())
                } else if !s.contains('@') {
                    Err("Invalid email format".into())
                } else {
                    Ok(())
                }
            })),
    ],
    on_submit: |data| {
        println!("Valid email submitted: {}", data.get("email").unwrap());
        Ok(())
    }
);
```

**Validation features:**
- Validators run on blur and submission
- Form only submits if all fields valid
- Error messages display automatically

### Multi-Field Form

```rust
use flux_state::{Runtime, Signal};
use widget_core::{form, TextInput};

let runtime = Runtime::new();
let name = Signal::new(runtime.clone(), String::new());
let email = Signal::new(runtime.clone(), String::new());
let age = Signal::new(runtime.clone(), String::new());

fn required(s: &str) -> Result<(), String> {
    if s.is_empty() {
        Err("This field is required".into())
    } else {
        Ok(())
    }
}

let form = form!(
    [
        ("name", TextInput::new(name)
            .placeholder("Full Name")
            .validator(required)),
        ("email", TextInput::new(email)
            .placeholder("Email")
            .validator(|s| {
                required(s)?;
                if s.contains('@') { Ok(()) } else { Err("Invalid email".into()) }
            })),
        ("age", TextInput::new(age)
            .placeholder("Age")
            .validator(|s| {
                required(s)?;
                s.parse::<u32>().map(|_| ()).map_err(|_| "Must be a number".into())
            })),
    ],
    gap: 12.0,
    padding: 20.0,
    on_submit: |data| {
        println!("Registration data: {:?}", data);
        Ok(())
    }
);
```

## Dynamic Content

### List from Iterator

Generate widgets from runtime data:

```rust
use widget_core::{list_from, Row, Text, Button, Spacer};

let items = vec!["Apple", "Banana", "Cherry", "Date"];

let list = list_from(items.iter().map(|item| {
    Row::new((
        Text::new(format!("• {}", item)),
        Spacer::flex(),
        Button::new("Select").secondary(),
    ))
    .gap(8.0)
}))
.gap(4.0);
```

### Dynamic List with State

```rust
use flux_state::{Runtime, Signal};
use widget_core::{list_from, Row, Text, Button, Spacer};

let runtime = Runtime::new();
let items = Signal::new(runtime.clone(), vec![
    "Task 1".to_string(),
    "Task 2".to_string(),
    "Task 3".to_string(),
]);

// Read current items
let current_items = items.get_untracked();

let list = list_from(current_items.iter().enumerate().map(|(i, item)| {
    let items = items.clone();
    Row::new((
        Text::new(item.clone()),
        Spacer::flex(),
        Button::new("Delete").secondary().on_click(move || {
            items.update(|list| {
                list.remove(i);
            });
        }),
    ))
    .gap(8.0)
}))
.gap(4.0);
```

### Grid from Data

```rust
use widget_core::{Grid, Card, Column, Text};

let data = vec![
    ("Metric 1", "42"),
    ("Metric 2", "128"),
    ("Metric 3", "99"),
    ("Metric 4", "256"),
];

Grid::new(
    data.iter().map(|(label, value)| {
        Card::new(
            Column::new((
                Text::new(*label).size(14.0),
                Text::new(*value).size(24.0),
            ))
            .gap(4.0)
        )
        .padding(16.0)
    }).collect::<Vec<_>>(),  // Collect into tuple
    2,  // 2 columns
)
.gap(12.0)
```

## Next Steps

### Example Applications

Study the example applications to see complete patterns:

```bash
# Comprehensive widget showcase
cargo run --example widget_gallery

# Form with validation
cargo run --example form_gui

# Declarative macro style
cargo run --example macro_demo

# Complete application
cargo run --example hello_world
```

### Advanced Topics

Once comfortable with basics, explore:

1. **Custom Widgets** - Implement the `Widget` trait for reusable components
2. **Theme Customization** - Use `theme-engine` for custom color schemes
3. **Effects** - Side effects triggered by signal changes
4. **Derived Signals** - Compute values from multiple signals
5. **Animation** - Animate widget properties (coming soon with `anim-graph`)

### Reference Documentation

- **Widget API Docs**: `cargo doc --open -p widget-core`
- **Widget Catalog**: See `crates/widget-core/README.md`
- **Architecture**: See `docs/adr/` for design decisions
- **Examples**: Browse `examples/` directory

## Common Pitfalls

### 1. Forgetting to clone signals

```rust
// ❌ Wrong - write moved into closure
let write = counter.split().1;
Button::new("Click").on_click(move || write.set(0))

// ✅ Correct - clone before moving
let write = counter.split().1;
let write_clone = write.clone();
Button::new("Click").on_click(move || write_clone.set(0))
```

### 2. Tuple vs List confusion

```rust
// Static children - use tuples
Row::new((child1, child2, child3))

// Dynamic children - use list_from
list_from(items.iter().map(|item| create_widget(item)))
```

### 3. Missing .build() in manual construction

```rust
// ❌ Wrong - forgot to build
let row = Row::new((child1, child2));
// row is a Row<_>, not a NodeId!

// ✅ Correct - build returns NodeId
let row_id = Row::new((child1, child2)).build(&mut ctx);
```

### 4. Nested padding confusion

```rust
// ❌ Redundant - both add padding
col!([txt!("Text")], padding: 20.0)  // Already has padding

// ✅ Clear - padding in one place
col!([
    Padding::all(20.0, txt!("Text")),
])
```

## Summary

You've learned:

✅ Basic widget creation with macros
✅ Layout composition (Row, Column, nesting)
✅ Spacing with gap, padding, and Spacer
✅ Practical UI patterns (toolbars, grids, cards)
✅ Reactive state with signals
✅ Forms with validation
✅ Dynamic content with List

**Next:** Build your own app using these patterns! Start simple and gradually add complexity.

Happy widget building! 🎨
