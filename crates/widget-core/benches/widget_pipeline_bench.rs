//! Widget Pipeline Benchmarks
//!
//! Benchmarks the full widget pipeline:
//! - Widget build time (scene graph construction)
//! - Layout computation (flexbox)
//! - Validation overhead
//! - Form aggregation
//!
//! Performance targets (Phase 2 plan):
//! - 1k widgets < 3ms full pipeline
//! - Widget build < 500μs for 1k widgets
//! - Form with 10 fields < 100μs
//!
//! Note: For dynamic content (loops with unknown count), we use scene-level
//! manipulation since the tuple-based API is designed for static composition.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use flux_state::{Runtime, Signal};
use layout_engine::FlexDirection;
use render_engine::NodeContent;
use widget_core::{Button, Container, FlexStyle, Form, Text, TextInput, Widget, WidgetContext};

/// Benchmark building simple text widgets
fn bench_text_widgets(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_widget_build");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut ctx = WidgetContext::new_test();
                for i in 0..size {
                    let text = Text::new(format!("Text {}", i));
                    black_box(text.build(&mut ctx));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark building buttons with interactions
fn bench_button_widgets(c: &mut Criterion) {
    let mut group = c.benchmark_group("button_widget_build");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut ctx = WidgetContext::new_test();
                for i in 0..size {
                    let button = Button::new(format!("Button {}", i)).on_click(|| {});
                    black_box(button.build(&mut ctx));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark building text inputs with validation
fn bench_text_input_widgets(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_input_widget_build");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let runtime = Runtime::new();
                let mut ctx = WidgetContext::new_test();

                for _i in 0..size {
                    let value = Signal::new(runtime.clone(), String::new());
                    let input = TextInput::new(value).validator(|s| {
                        if s.is_empty() {
                            Err("Required".to_string())
                        } else {
                            Ok(())
                        }
                    });
                    black_box(input.build(&mut ctx));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark building containers with children (dynamic content)
fn bench_container_widgets(c: &mut Criterion) {
    let mut group = c.benchmark_group("container_widget_build");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut ctx = WidgetContext::new_test();

                // For dynamic content, use scene-level manipulation
                let container_id = ctx.create_node(ctx.root(), NodeContent::Empty);
                let style = FlexStyle {
                    direction: FlexDirection::Column,
                    ..Default::default()
                };
                ctx.set_layout_style(container_id, style);

                for i in 0..size {
                    let text = Text::new(format!("Item {}", i));
                    let text_id = text.build(&mut ctx);
                    ctx.reparent_to(text_id, container_id);
                }

                black_box(container_id);
            });
        });
    }

    group.finish();
}

/// Benchmark nested container layouts (dynamic content)
fn bench_nested_containers(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_container_build");

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut ctx = WidgetContext::new_test();

                // Create outer column container
                let container_id = ctx.create_node(ctx.root(), NodeContent::Empty);
                let col_style = FlexStyle {
                    direction: FlexDirection::Column,
                    ..Default::default()
                };
                ctx.set_layout_style(container_id, col_style);

                for i in 0..size {
                    // Each row uses tuple-based static composition
                    let row =
                        Container::row((Text::new(format!("Label {}", i)), Button::new("Action")));
                    let row_id = row.build(&mut ctx);
                    ctx.reparent_to(row_id, container_id);
                }

                black_box(container_id);
            });
        });
    }

    group.finish();
}

/// Benchmark form with multiple fields (static composition with known field count)
fn bench_form_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("form_build");

    // Benchmark fixed field counts using tuple-based API
    group.bench_function("5_fields", |b| {
        b.iter(|| {
            let runtime = Runtime::new();
            let mut ctx = WidgetContext::new_test();

            let form = Form::new((
                (
                    "field_0",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_1",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_2",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_3",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_4",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
            ));

            black_box(form.build(&mut ctx));
        });
    });

    group.bench_function("10_fields", |b| {
        b.iter(|| {
            let runtime = Runtime::new();
            let mut ctx = WidgetContext::new_test();

            let form = Form::new((
                (
                    "field_0",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_1",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_2",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_3",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_4",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_5",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_6",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_7",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_8",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
                (
                    "field_9",
                    TextInput::new(Signal::new(runtime.clone(), String::new())),
                ),
            ));

            black_box(form.build(&mut ctx));
        });
    });

    group.finish();
}

/// Benchmark form revalidation
fn bench_form_revalidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("form_revalidation");

    group.bench_function("5_fields", |b| {
        // Setup: build form with fields
        let runtime = Runtime::new();
        let mut ctx = WidgetContext::new_test();

        let form = Form::new((
            (
                "field_0",
                TextInput::new(Signal::new(runtime.clone(), "value_0".to_string())).validator(
                    |s| {
                        if s.len() >= 3 {
                            Ok(())
                        } else {
                            Err("Too short".to_string())
                        }
                    },
                ),
            ),
            (
                "field_1",
                TextInput::new(Signal::new(runtime.clone(), "value_1".to_string())).validator(
                    |s| {
                        if s.len() >= 3 {
                            Ok(())
                        } else {
                            Err("Too short".to_string())
                        }
                    },
                ),
            ),
            (
                "field_2",
                TextInput::new(Signal::new(runtime.clone(), "value_2".to_string())).validator(
                    |s| {
                        if s.len() >= 3 {
                            Ok(())
                        } else {
                            Err("Too short".to_string())
                        }
                    },
                ),
            ),
            (
                "field_3",
                TextInput::new(Signal::new(runtime.clone(), "value_3".to_string())).validator(
                    |s| {
                        if s.len() >= 3 {
                            Ok(())
                        } else {
                            Err("Too short".to_string())
                        }
                    },
                ),
            ),
            (
                "field_4",
                TextInput::new(Signal::new(runtime.clone(), "value_4".to_string())).validator(
                    |s| {
                        if s.len() >= 3 {
                            Ok(())
                        } else {
                            Err("Too short".to_string())
                        }
                    },
                ),
            ),
        ));

        let form_id = form.build(&mut ctx);

        // Benchmark revalidation
        b.iter(|| {
            ctx.revalidate_form(form_id);
            black_box(());
        });
    });

    group.finish();
}

/// Benchmark full widget pipeline (build + layout styles)
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_widget_pipeline");

    for size in [100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let runtime = Runtime::new();
                let mut ctx = WidgetContext::new_test();

                // Create outer container for dynamic content
                let container_id = ctx.create_node(ctx.root(), NodeContent::Empty);
                let col_style = FlexStyle {
                    direction: FlexDirection::Column,
                    gap: 8.0,
                    padding_left: 16.0,
                    padding_right: 16.0,
                    padding_top: 16.0,
                    padding_bottom: 16.0,
                    ..Default::default()
                };
                ctx.set_layout_style(container_id, col_style);

                // Build rows with static tuple composition
                for i in 0..(size / 3) {
                    let row = Container::row((
                        Text::new(format!("Label {}", i)),
                        Button::new("Click").on_click(|| {}),
                    ))
                    .gap(12.0);
                    let row_id = row.build(&mut ctx);
                    ctx.reparent_to(row_id, container_id);
                }

                // Add some inputs
                for i in 0..(size / 3) {
                    let value = Signal::new(runtime.clone(), String::new());
                    let input = TextInput::new(value).placeholder(format!("Input {}", i));
                    let input_id = input.build(&mut ctx);
                    ctx.reparent_to(input_id, container_id);
                }

                black_box(container_id);

                // Note: In real implementation, we'd also run:
                // - Layout computation (layout_engine)
                // - Text shaping (text_engine)
                // - Reactive updates
                // This benchmark focuses on widget build time
            });
        });
    }

    group.finish();
}

/// Benchmark text input editing (cursor movement, character insertion)
fn bench_text_input_editing(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_input_editing");

    group.bench_function("insert_char", |b| {
        let runtime = Runtime::new();
        let mut ctx = WidgetContext::new_test();
        let value = Signal::new(runtime.clone(), "Hello".to_string());
        let input = TextInput::new(value);
        let input_id = input.build(&mut ctx);
        ctx.focus_node(input_id);

        b.iter(|| {
            ctx.send_char('!');
            black_box(());
        });
    });

    group.bench_function("backspace", |b| {
        let runtime = Runtime::new();
        let mut ctx = WidgetContext::new_test();
        let value = Signal::new(runtime.clone(), "Hello World".to_string());
        let input = TextInput::new(value);
        let input_id = input.build(&mut ctx);
        ctx.focus_node(input_id);

        b.iter(|| {
            ctx.send_backspace();
            black_box(());
        });
    });

    group.bench_function("cursor_movement", |b| {
        let runtime = Runtime::new();
        let mut ctx = WidgetContext::new_test();
        let value = Signal::new(runtime.clone(), "Hello World".to_string());
        let input = TextInput::new(value);
        let input_id = input.build(&mut ctx);
        ctx.focus_node(input_id);

        b.iter(|| {
            ctx.send_key_left();
            black_box(());
            ctx.send_key_right();
            black_box(());
        });
    });

    group.finish();
}

/// Benchmark focus navigation (Tab/Shift+Tab)
fn bench_focus_navigation(c: &mut Criterion) {
    let mut group = c.benchmark_group("focus_navigation");

    // Setup context with multiple text inputs
    for count in [5, 10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let runtime = Runtime::new();
            let mut ctx = WidgetContext::new_test();

            // Create multiple text inputs
            for _ in 0..count {
                let value = Signal::new(runtime.clone(), String::new());
                let input = TextInput::new(value);
                let input_id = input.build(&mut ctx);
                // Make sure the first one is focused
                if ctx.focused_node().is_none() {
                    ctx.focus_node(input_id);
                }
            }

            b.iter(|| {
                black_box(ctx.focus_next());
            });
        });
    }

    group.finish();
}

/// Benchmark focus_prev specifically
fn bench_focus_prev(c: &mut Criterion) {
    let mut group = c.benchmark_group("focus_prev");

    let runtime = Runtime::new();
    let mut ctx = WidgetContext::new_test();

    // Create 20 text inputs
    for _ in 0..20 {
        let value = Signal::new(runtime.clone(), String::new());
        let input = TextInput::new(value);
        let input_id = input.build(&mut ctx);
        ctx.focus_node(input_id);
    }

    group.bench_function("20_inputs", |b| {
        b.iter(|| {
            black_box(ctx.focus_prev());
        });
    });

    group.finish();
}

/// Benchmark focus cycling (full round-trip)
fn bench_focus_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("focus_cycle");

    let runtime = Runtime::new();
    let mut ctx = WidgetContext::new_test();
    let mut first_id = None;

    // Create 10 text inputs
    for _ in 0..10 {
        let value = Signal::new(runtime.clone(), String::new());
        let input = TextInput::new(value);
        let input_id = input.build(&mut ctx);
        if first_id.is_none() {
            first_id = Some(input_id);
            ctx.focus_node(input_id);
        }
    }

    group.bench_function("full_cycle_10", |b| {
        b.iter(|| {
            // Cycle through all 10 inputs
            for _ in 0..10 {
                black_box(ctx.focus_next());
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_text_widgets,
    bench_button_widgets,
    bench_text_input_widgets,
    bench_container_widgets,
    bench_nested_containers,
    bench_form_build,
    bench_form_revalidation,
    bench_full_pipeline,
    bench_text_input_editing,
    bench_focus_navigation,
    bench_focus_prev,
    bench_focus_cycle,
);
criterion_main!(benches);
