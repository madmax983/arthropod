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

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use widget_core::{Container, Text, Button, TextInput, Form, Widget, WidgetContext};
use flux_state::{Runtime, Signal};

/// Benchmark building simple text widgets
fn bench_text_widgets(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_widget_build");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut ctx = WidgetContext::new_test();
                    for i in 0..size {
                        let text = Text::new(format!("Text {}", i));
                        black_box(text.build(&mut ctx));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark building buttons with interactions
fn bench_button_widgets(c: &mut Criterion) {
    let mut group = c.benchmark_group("button_widget_build");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut ctx = WidgetContext::new_test();
                    for i in 0..size {
                        let button = Button::new(format!("Button {}", i))
                            .on_click(|| {});
                        black_box(button.build(&mut ctx));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark building text inputs with validation
fn bench_text_input_widgets(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_input_widget_build");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter(|| {
                    let runtime = Runtime::new();
                    let mut ctx = WidgetContext::new_test();

                    for _i in 0..size {
                        let value = Signal::new(runtime.clone(), String::new());
                        let input = TextInput::new(value)
                            .validator(|s| {
                                if s.is_empty() {
                                    Err("Required".to_string())
                                } else {
                                    Ok(())
                                }
                            });
                        black_box(input.build(&mut ctx));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark building containers with children
fn bench_container_widgets(c: &mut Criterion) {
    let mut group = c.benchmark_group("container_widget_build");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut ctx = WidgetContext::new_test();

                    let mut container = Container::column();
                    for i in 0..size {
                        container = container.child(Text::new(format!("Item {}", i)));
                    }

                    black_box(container.build(&mut ctx));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark nested container layouts
fn bench_nested_containers(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_container_build");

    for size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut ctx = WidgetContext::new_test();

                    let mut container = Container::column();
                    for i in 0..size {
                        container = container.child(
                            Container::row()
                                .child(Text::new(format!("Label {}", i)))
                                .child(Button::new("Action"))
                        );
                    }

                    black_box(container.build(&mut ctx));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark form with multiple fields
fn bench_form_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("form_build");

    for field_count in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(field_count),
            field_count,
            |b, &field_count| {
                b.iter(|| {
                    let runtime = Runtime::new();
                    let mut ctx = WidgetContext::new_test();

                    let mut form = Form::new();
                    for i in 0..field_count {
                        let value = Signal::new(runtime.clone(), String::new());
                        form = form.field(
                            format!("field_{}", i),
                            TextInput::new(value)
                                .validator(|s| if s.is_empty() { Err("Required".to_string()) } else { Ok(()) })
                        );
                    }

                    black_box(form.build(&mut ctx));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark form revalidation
fn bench_form_revalidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("form_revalidation");

    for field_count in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(field_count),
            field_count,
            |b, &field_count| {
                // Setup: build form with fields
                let runtime = Runtime::new();
                let mut ctx = WidgetContext::new_test();

                let mut form = Form::new();
                for i in 0..field_count {
                    let value = Signal::new(runtime.clone(), format!("value_{}", i));
                    form = form.field(
                        format!("field_{}", i),
                        TextInput::new(value)
                            .validator(|s| if s.len() >= 3 { Ok(()) } else { Err("Too short".to_string()) })
                    );
                }

                let form_id = form.build(&mut ctx);

                // Benchmark revalidation
                b.iter(|| {
                    black_box(ctx.revalidate_form(form_id));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark full widget pipeline (build + layout styles)
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_widget_pipeline");

    for size in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter(|| {
                    let runtime = Runtime::new();
                    let mut ctx = WidgetContext::new_test();

                    // Build a realistic widget tree
                    let mut container = Container::column().gap(8.0).padding(16.0);

                    for i in 0..(size / 3) {
                        container = container.child(
                            Container::row()
                                .gap(12.0)
                                .child(Text::new(format!("Label {}", i)))
                                .child(Button::new("Click").on_click(|| {}))
                        );
                    }

                    // Add some inputs
                    for i in 0..(size / 3) {
                        let value = Signal::new(runtime.clone(), String::new());
                        container = container.child(
                            TextInput::new(value)
                                .placeholder(&format!("Input {}", i))
                        );
                    }

                    black_box(container.build(&mut ctx));

                    // Note: In real implementation, we'd also run:
                    // - Layout computation (layout_engine)
                    // - Text shaping (text_engine)
                    // - Reactive updates
                    // This benchmark focuses on widget build time
                });
            },
        );
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
            black_box(ctx.send_char('!'));
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
            black_box(ctx.send_backspace());
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
            black_box(ctx.send_key_left());
            black_box(ctx.send_key_right());
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
);
criterion_main!(benches);
