//! Framework Benchmarks
//!
//! Benchmarks for core framework operations:
//! - EventDispatcher performance
//! - auto_layout computation
//!
//! Performance targets:
//! - Event dispatch < 1μs per event
//! - Layout computation for 100 widgets < 500μs

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use flux_state::{Runtime, Signal};
use indexmap::IndexMap;
use layout_engine::{FlexDirection, FlexStyle};
use plat_core::{ElementState, Event, Key, KeyboardInput, Modifiers, Point, WindowEvent};
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode};
use std::collections::HashMap;
use widget_core::WidgetContext;

use arthropod::event_dispatcher::EventDispatcher;
use arthropod::layout::auto_layout;

// =========================================================================
// Helper functions
// =========================================================================

fn create_keyboard_event(key: Key) -> Event {
    Event::Window {
        window_id: Default::default(),
        event: WindowEvent::KeyboardInput(KeyboardInput {
            key,
            state: ElementState::Pressed,
            modifiers: Modifiers::default(),
            repeat: false,
        }),
    }
}

fn create_cursor_event(x: f64, y: f64) -> Event {
    Event::Window {
        window_id: Default::default(),
        event: WindowEvent::CursorMoved {
            position: Point { x, y },
        },
    }
}

fn create_text_input_node(ctx: &mut WidgetContext) -> NodeId {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime, String::new());
    let (read, write) = signal.split();
    let node_id = ctx.create_node(
        ctx.root(),
        NodeContent::Rect {
            color: Color::WHITE,
        },
    );
    ctx.add_text_input_state(node_id, read, write, false, None);
    node_id
}

fn create_scene_with_nodes(count: usize) -> (Scene, Vec<NodeId>, HashMap<NodeId, FlexStyle>) {
    let mut scene = Scene::new();
    let root = scene.root();
    let mut node_ids = Vec::new();
    let mut styles = HashMap::new();

    // Root style
    styles.insert(
        root,
        FlexStyle {
            direction: FlexDirection::Column,
            width: Some(800.0),
            height: Some(600.0),
            gap: 8.0,
            padding_left: 16.0,
            padding_right: 16.0,
            padding_top: 16.0,
            padding_bottom: 16.0,
            ..Default::default()
        },
    );

    for _ in 0..count {
        let node = SceneNode::new(NodeContent::Rect { color: Color::RED });
        let node_id = scene.add_node(root, node);
        node_ids.push(node_id);

        styles.insert(
            node_id,
            FlexStyle {
                width: Some(200.0),
                height: Some(40.0),
                ..Default::default()
            },
        );
    }

    (scene, node_ids, styles)
}

// =========================================================================
// EventDispatcher Benchmarks
// =========================================================================

/// Benchmark dispatching keyboard events
fn bench_dispatch_keyboard(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_dispatch_keyboard");

    group.bench_function("character_key", |b| {
        let mut ctx = WidgetContext::new_test();
        let node_id = create_text_input_node(&mut ctx);
        ctx.focus_node(node_id);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();
        let event = create_keyboard_event(Key::A);

        b.iter(|| {
            black_box(dispatcher.dispatch(&event, &mut ctx, &scene));
        });
    });

    group.bench_function("special_key_tab", |b| {
        let mut ctx = WidgetContext::new_test();
        let _node1 = create_text_input_node(&mut ctx);
        let _node2 = create_text_input_node(&mut ctx);
        ctx.focus_next();

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();
        let event = create_keyboard_event(Key::Tab);

        b.iter(|| {
            black_box(dispatcher.dispatch(&event, &mut ctx, &scene));
        });
    });

    group.bench_function("backspace", |b| {
        let mut ctx = WidgetContext::new_test();
        let runtime = Runtime::new();
        let signal = Signal::new(runtime, "Hello World".to_string());
        let (read, write) = signal.split();
        let node_id = ctx.create_node(
            ctx.root(),
            NodeContent::Rect {
                color: Color::WHITE,
            },
        );
        ctx.add_text_input_state(node_id, read, write, false, None);
        ctx.focus_node(node_id);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();
        let event = create_keyboard_event(Key::Backspace);

        b.iter(|| {
            black_box(dispatcher.dispatch(&event, &mut ctx, &scene));
        });
    });

    group.finish();
}

/// Benchmark dispatching mouse events
fn bench_dispatch_mouse(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_dispatch_mouse");

    group.bench_function("cursor_moved", |b| {
        let mut ctx = WidgetContext::new_test();
        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let scene = Scene::new();

        let mut x = 0.0;
        b.iter(|| {
            x += 1.0;
            let event = create_cursor_event(x, 100.0);
            black_box(dispatcher.dispatch(&event, &mut ctx, &scene));
        });
    });

    group.finish();
}

/// Benchmark event dispatch with form submission
fn bench_dispatch_form_submit(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_dispatch_form");

    group.bench_function("enter_submit", |b| {
        let mut ctx = WidgetContext::new_test();
        let form_node = NodeId(999);
        ctx.add_form_state(form_node, IndexMap::new(), None);

        let mut dispatcher = EventDispatcher::new(HashMap::new(), Some(form_node));
        let scene = Scene::new();
        let event = create_keyboard_event(Key::Enter);

        b.iter(|| {
            black_box(dispatcher.dispatch(&event, &mut ctx, &scene));
        });
    });

    group.finish();
}

// =========================================================================
// auto_layout Benchmarks
// =========================================================================

/// Benchmark auto_layout with varying widget counts
fn bench_auto_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("auto_layout");

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let (mut scene, _node_ids, styles) = create_scene_with_nodes(count);
            let root = scene.root();

            b.iter(|| {
                auto_layout(&mut scene, root, 800.0, 600.0, &styles);
                black_box(&scene);
            });
        });
    }

    group.finish();
}

/// Benchmark layout with nested containers
fn bench_auto_layout_nested(c: &mut Criterion) {
    let mut group = c.benchmark_group("auto_layout_nested");

    for depth in [2, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            let mut scene = Scene::new();
            let mut styles = HashMap::new();
            let root = scene.root();

            styles.insert(
                root,
                FlexStyle {
                    direction: FlexDirection::Column,
                    width: Some(800.0),
                    height: Some(600.0),
                    padding_left: 10.0,
                    padding_top: 10.0,
                    ..Default::default()
                },
            );

            // Create nested containers
            let mut current_parent = root;
            for _ in 0..depth {
                let container = SceneNode::new(NodeContent::Empty);
                let container_id = scene.add_node(current_parent, container);

                styles.insert(
                    container_id,
                    FlexStyle {
                        direction: FlexDirection::Column,
                        width: Some(200.0),
                        height: Some(100.0),
                        padding_left: 5.0,
                        padding_top: 5.0,
                        ..Default::default()
                    },
                );

                // Add a few children at each level
                for _ in 0..3 {
                    let child = SceneNode::new(NodeContent::Rect { color: Color::RED });
                    let child_id = scene.add_node(container_id, child);

                    styles.insert(
                        child_id,
                        FlexStyle {
                            width: Some(50.0),
                            height: Some(20.0),
                            ..Default::default()
                        },
                    );
                }

                current_parent = container_id;
            }

            b.iter(|| {
                auto_layout(&mut scene, root, 800.0, 600.0, &styles);
                black_box(&scene);
            });
        });
    }

    group.finish();
}

/// Benchmark layout with row containers
fn bench_auto_layout_row(c: &mut Criterion) {
    let mut group = c.benchmark_group("auto_layout_row");

    for count in [5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut scene = Scene::new();
            let mut styles = HashMap::new();
            let root = scene.root();

            styles.insert(
                root,
                FlexStyle {
                    direction: FlexDirection::Row,
                    width: Some(800.0),
                    height: Some(100.0),
                    gap: 10.0,
                    ..Default::default()
                },
            );

            for _ in 0..count {
                let child = SceneNode::new(NodeContent::Rect { color: Color::BLUE });
                let child_id = scene.add_node(root, child);

                styles.insert(
                    child_id,
                    FlexStyle {
                        width: Some(50.0),
                        height: Some(50.0),
                        ..Default::default()
                    },
                );
            }

            b.iter(|| {
                auto_layout(&mut scene, root, 800.0, 600.0, &styles);
                black_box(&scene);
            });
        });
    }

    group.finish();
}

/// Benchmark combined event + layout cycle (simulating frame update)
fn bench_frame_update_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_update_cycle");

    group.bench_function("10_widgets", |b| {
        let (mut scene, _node_ids, styles) = create_scene_with_nodes(10);
        let root = scene.root();

        let mut ctx = WidgetContext::new_test();
        for _ in 0..5 {
            create_text_input_node(&mut ctx);
        }
        ctx.focus_next();

        let mut dispatcher = EventDispatcher::new(HashMap::new(), None);
        let events = vec![
            create_keyboard_event(Key::H),
            create_keyboard_event(Key::E),
            create_keyboard_event(Key::L),
            create_keyboard_event(Key::L),
            create_keyboard_event(Key::O),
        ];

        b.iter(|| {
            // Process events
            for event in &events {
                dispatcher.dispatch(event, &mut ctx, &scene);
            }

            // Run layout
            auto_layout(&mut scene, root, 800.0, 600.0, &styles);

            black_box(&scene);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_dispatch_keyboard,
    bench_dispatch_mouse,
    bench_dispatch_form_submit,
    bench_auto_layout,
    bench_auto_layout_nested,
    bench_auto_layout_row,
    bench_frame_update_cycle,
);
criterion_main!(benches);
