//! Widget Gallery - Showcase of all available widgets
//!
//! This example demonstrates:
//! - All widget types (Container, Text, Button, TextInput, Form)
//! - Reactive state updates
//! - Layout patterns (rows, columns, nesting)
//! - Theming and styling
//!
//! Run with: cargo run --example widget_gallery

use flux_state::{Runtime, Signal};
use glam::Vec4;
use widget_core::{
    Button, Container, FlexDirection, FlexStyle, Form, NodeContent, Text, TextInput, Widget,
    WidgetContext,
};

fn main() {
    println!("=== Arthropod Widget Gallery ===\n");

    let runtime = Runtime::new();
    let mut ctx = WidgetContext::new_test();

    println!("1. Text Widget");
    println!("   - Static text with color and size");
    let text1 = Text::new("Hello, Arthropod!")
        .size(24.0)
        .color(Vec4::new(0.0, 0.47, 0.84, 1.0)); // Blue
    let text1_id = text1.build(&mut ctx);
    println!("   ✓ Built text widget (node: {:?})", text1_id);

    println!("\n2. Reactive Text Widget");
    println!("   - Text that updates when signal changes");
    let counter_text_signal = Signal::new(runtime.clone(), "Count: 0".to_string());
    let (read, write) = counter_text_signal.split();

    let counter_text = Text::reactive(read);
    let _counter_id = counter_text.build(&mut ctx);
    println!("   ✓ Built reactive text (initial value: Count: 0)");

    write.set("Count: 5".to_string());
    println!("   ✓ Updated counter to 5 (text will update reactively)");

    println!("\n3. Button Widget");
    println!("   - Primary button with click handler");
    let clicked_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let clicked_clone = clicked_count.clone();

    let button = Button::new("Click Me!")
        .primary()
        .padding(16.0)
        .on_click(move || {
            clicked_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
    let button_id = button.build(&mut ctx);
    println!("   ✓ Built button widget (node: {:?})", button_id);

    ctx.trigger_click(button_id);
    println!(
        "   ✓ Simulated click (count: {})",
        clicked_count.load(std::sync::atomic::Ordering::SeqCst)
    );

    println!("\n4. TextInput Widget");
    println!("   - Single-line text input with validation");
    let name = Signal::new(runtime.clone(), String::new());

    let input = TextInput::new(name)
        .placeholder("Enter your name...")
        .validator(|s| {
            if s.is_empty() {
                Err("Name is required".to_string())
            } else {
                Ok(())
            }
        });
    let input_id = input.build(&mut ctx);
    println!("   ✓ Built text input (node: {:?})", input_id);
    println!("   ✓ Has placeholder: {}", ctx.has_placeholder(input_id));
    println!(
        "   ✓ Has validation error: {}",
        ctx.has_validation_error(input_id)
    );

    println!("\n5. Container Widget - Row Layout");
    println!("   - Horizontal arrangement of children");
    let row = Container::row((
        Text::new("Item 1"),
        Text::new("Item 2"),
        Text::new("Item 3"),
    ))
    .gap(10.0);
    let row_id = row.build(&mut ctx);
    let row_node = ctx.scene().get_node(row_id).unwrap();
    println!(
        "   ✓ Built row container with {} children",
        row_node.children.len()
    );

    println!("\n6. Container Widget - Column Layout");
    println!("   - Vertical arrangement of children");
    let column = Container::column((Text::new("First"), Text::new("Second"), Text::new("Third")))
        .gap(12.0)
        .padding(16.0);
    let column_id = column.build(&mut ctx);
    let column_node = ctx.scene().get_node(column_id).unwrap();
    println!(
        "   ✓ Built column container with {} children",
        column_node.children.len()
    );

    println!("\n7. Nested Containers");
    println!("   - Complex layouts with nested rows and columns");
    let nested = Container::column((
        Text::new("Header").size(20.0),
        Container::row((
            Button::new("Action 1").secondary(),
            Button::new("Action 2").secondary(),
        ))
        .gap(8.0),
        Text::new("Footer"),
    ));
    let nested_id = nested.build(&mut ctx);
    let nested_node = ctx.scene().get_node(nested_id).unwrap();
    println!("   ✓ Built nested layout (depth: 2 levels)");
    println!("   ✓ Top-level children: {}", nested_node.children.len());

    println!("\n8. Form Widget");
    println!("   - Complete form with multiple fields");
    let username = Signal::new(runtime.clone(), String::new());
    let email = Signal::new(runtime.clone(), String::new());

    let form = Form::new((
        (
            "username",
            TextInput::new(username)
                .placeholder("Username")
                .validator(|s| {
                    if s.len() >= 3 {
                        Ok(())
                    } else {
                        Err("Too short".to_string())
                    }
                }),
        ),
        (
            "email",
            TextInput::new(email).placeholder("Email").validator(|s| {
                if s.contains('@') {
                    Ok(())
                } else {
                    Err("Invalid email".to_string())
                }
            }),
        ),
    ))
    .gap(12.0)
    .on_submit(|data| {
        println!("   📧 Form submitted: {:?}", data);
        Ok(())
    });
    let form_id = form.build(&mut ctx);
    let form_node = ctx.scene().get_node(form_id).unwrap();
    println!("   ✓ Built form with {} fields", form_node.children.len());
    println!("   ✓ Form is valid: {}", ctx.is_form_valid(form_id));

    println!("\n9. Button Styles");
    println!("   - Different button variants");
    let button_default = Button::new("Default");
    let button_primary = Button::new("Primary").primary();
    let button_secondary = Button::new("Secondary").secondary();
    let button_disabled = Button::new("Disabled").disabled(true);

    println!("   ✓ Default button");
    button_default.build(&mut ctx);
    println!("   ✓ Primary button (themed accent color)");
    button_primary.build(&mut ctx);
    println!("   ✓ Secondary button (gray)");
    button_secondary.build(&mut ctx);
    println!("   ✓ Disabled button (no click events)");
    button_disabled.build(&mut ctx);

    println!("\n10. Large Widget Tree");
    println!("    - Performance test with many widgets");
    let start = std::time::Instant::now();

    // For dynamic content (loops), create container manually and reparent children
    let large_id = ctx.create_node(ctx.root(), NodeContent::Empty);
    let column_style = FlexStyle {
        direction: FlexDirection::Column,
        ..Default::default()
    };
    ctx.set_layout_style(large_id, column_style);

    for i in 0..100 {
        let row = Container::row((Text::new(format!("Item {}", i)), Button::new("Click")));
        let row_id = row.build(&mut ctx);
        ctx.reparent_to(row_id, large_id);
    }
    let elapsed = start.elapsed();

    let large_node = ctx.scene().get_node(large_id).unwrap();
    println!("    ✓ Built 100 row containers (200 widgets total)");
    println!("    ✓ Build time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
    println!("    ✓ Children count: {}", large_node.children.len());

    // Scene statistics
    println!("\n=== Scene Statistics ===");
    let total_nodes: usize = ctx.scene().nodes().count();
    println!("Total nodes in scene: {}", total_nodes);
    println!("Root node: {:?}", ctx.scene().root());

    println!("\n=== Widget Gallery Complete ===");
    println!("\nAll widget types demonstrated:");
    println!("  ✓ Text (static and reactive)");
    println!("  ✓ Button (with styles and interactions)");
    println!("  ✓ TextInput (with validation and placeholder)");
    println!("  ✓ Container (row and column layouts)");
    println!("  ✓ Form (field aggregation and submission)");
    println!("  ✓ Nested layouts (complex hierarchies)");
    println!("  ✓ Performance (100+ widgets built efficiently)");
}
