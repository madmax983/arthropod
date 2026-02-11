//! Benchmarks for Phase 1+2 Primitive Rendering Pipeline
//!
//! Performance requirements:
//! Phase 1:
//! - 1000 solid primitives: < 200μs
//! - Scene node operations: < 10μs
//! Phase 2:
//! - 1000 gradient primitives: < 400μs
//! - 1000 stroked primitives: < 300μs
//! - 1000 shadowed primitives: < 500μs
//! - 1000 gradient text nodes: < 350μs

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use glam::{Vec2, Vec4};
use plat_core::Rect;
use render_engine::{Color, ColorStop, NodeContent, Paint, Scene, SceneNode, StrokeStyle, VisualStyle};
use style_engine::{LinearGradient, StrokeAlign, TextContent};

/// Benchmark creating 1000 scene nodes with solid fills
fn bench_create_nodes(c: &mut Criterion) {
    c.bench_function("create_1000_solid_nodes", |b| {
        b.iter(|| {
            let mut scene = Scene::new();
            let root = scene.root();

            for i in 0..1000 {
                let x = (i % 40) as f32 * 25.0;
                let y = (i / 40) as f32 * 25.0;

                let node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(
                        VisualStyle::new().solid_fill(Color::rgba(0.5, 0.5, 0.5, 1.0).as_vec4()),
                    ),
                });

                let node_id = scene.add_node(root, node);

                // Set bounds (realistic workload)
                if let Some(node) = scene.get_node_mut(node_id) {
                    node.bounds = Rect::new(x, y, 20.0, 20.0);
                }
            }

            black_box(scene);
        });
    });
}

/// Benchmark creating nodes with different corner radii
fn bench_create_rounded_nodes(c: &mut Criterion) {
    c.bench_function("create_1000_rounded_nodes", |b| {
        b.iter(|| {
            let mut scene = Scene::new();
            let root = scene.root();

            for i in 0..1000 {
                let x = (i % 40) as f32 * 25.0;
                let y = (i / 40) as f32 * 25.0;
                let radius = (i % 16) as f32;

                let node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(
                        VisualStyle::new()
                            .solid_fill(Color::rgba(0.5, 0.5, 0.5, 1.0).as_vec4())
                            .corner_radius(radius),
                    ),
                });

                let node_id = scene.add_node(root, node);

                if let Some(node) = scene.get_node_mut(node_id) {
                    node.bounds = Rect::new(x, y, 20.0, 20.0);
                }
            }

            black_box(scene);
        });
    });
}

/// Benchmark scene node lookup operations
fn bench_node_lookup(c: &mut Criterion) {
    // Setup: Create a scene with 1000 nodes
    let mut scene = Scene::new();
    let root = scene.root();
    let mut node_ids = Vec::with_capacity(1000);

    for _ in 0..1000 {
        let node = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
        let id = scene.add_node(root, node);
        node_ids.push(id);
    }

    c.bench_function("lookup_1000_nodes", |b| {
        b.iter(|| {
            for &id in &node_ids {
                black_box(scene.get_node(id));
            }
        });
    });
}

/// Benchmark mutable node access (for reactive updates)
fn bench_node_mutation(c: &mut Criterion) {
    // Setup: Create a scene with 100 nodes
    let mut scene = Scene::new();
    let root = scene.root();
    let mut node_ids = Vec::with_capacity(100);

    for _ in 0..100 {
        let node = SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
        let id = scene.add_node(root, node);
        node_ids.push(id);
    }

    c.bench_function("mutate_100_node_colors", |b| {
        b.iter(|| {
            for &id in &node_ids {
                if let Some(node) = scene.get_node_mut(id) {
                    if let NodeContent::Styled { ref mut style } = node.content {
                        if !style.fills.is_empty() {
                            style.fills[0] = Paint::Solid(Color::BLUE.as_vec4());
                        }
                    }
                }
            }
        });
    });
}

/// Benchmark creating VisualStyle instances
fn bench_visual_style_creation(c: &mut Criterion) {
    c.bench_function("create_1000_visual_styles", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let style = VisualStyle::new()
                    .solid_fill(Color::rgba(0.5, 0.5, 0.5, 1.0).as_vec4())
                    .corner_radius(8.0);
                black_box(style);
            }
        });
    });
}

/// Benchmark scene iteration (for rendering pipeline)
fn bench_scene_iteration(c: &mut Criterion) {
    // Create scenes of different sizes
    for node_count in [100, 500, 1000, 5000].iter() {
        let mut scene = Scene::new();
        let root = scene.root();

        for i in 0..*node_count {
            let x = (i % 100) as f32 * 10.0;
            let y = (i / 100) as f32 * 10.0;

            let node = SceneNode::new(NodeContent::Styled {
                style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
            });
            let id = scene.add_node(root, node);

            if let Some(node) = scene.get_node_mut(id) {
                node.bounds = Rect::new(x, y, 8.0, 8.0);
            }
        }

        c.bench_with_input(
            BenchmarkId::new("iterate_nodes", node_count),
            node_count,
            |b, _| {
                b.iter(|| {
                    let count = scene.nodes().count();
                    black_box(count);
                });
            },
        );
    }
}

/// Benchmark visual iterator (used by render system)
fn bench_visual_iterator(c: &mut Criterion) {
    for node_count in [100, 500, 1000, 5000].iter() {
        let mut scene = Scene::new();
        let root = scene.root();

        for i in 0..*node_count {
            let x = (i % 100) as f32 * 10.0;
            let y = (i / 100) as f32 * 10.0;

            let node = SceneNode::new(NodeContent::Styled {
                style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
            });
            let id = scene.add_node(root, node);

            if let Some(node) = scene.get_node_mut(id) {
                node.bounds = Rect::new(x, y, 8.0, 8.0);
                node.visible = true;
            }
        }

        c.bench_with_input(
            BenchmarkId::new("iterate_visuals", node_count),
            node_count,
            |b, _| {
                b.iter(|| {
                    let count = scene.iter_visuals().count();
                    black_box(count);
                });
            },
        );
    }
}

/// Benchmark creating nodes with linear gradients (Phase 2)
fn bench_create_gradient_nodes(c: &mut Criterion) {
    c.bench_function("create_1000_gradient_nodes", |b| {
        b.iter(|| {
            let mut scene = Scene::new();
            let root = scene.root();

            for i in 0..1000 {
                let x = (i % 40) as f32 * 25.0;
                let y = (i / 40) as f32 * 25.0;

                // Real gradient rendering
                let node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(
                        VisualStyle::new()
                            .fill(Paint::Linear(LinearGradient {
                                start: Vec2::new(0.0, 0.5),
                                end: Vec2::new(1.0, 0.5),
                                stops: vec![
                                    ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
                                    ColorStop::new(1.0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
                                ],
                            }))
                            .corner_radius(8.0),
                    ),
                });

                let node_id = scene.add_node(root, node);

                if let Some(node) = scene.get_node_mut(node_id) {
                    node.bounds = Rect::new(x, y, 20.0, 20.0);
                }
            }

            black_box(scene);
        });
    });
}

/// Benchmark creating nodes with strokes (Phase 2)
fn bench_create_stroked_nodes(c: &mut Criterion) {
    c.bench_function("create_1000_stroked_nodes", |b| {
        b.iter(|| {
            let mut scene = Scene::new();
            let root = scene.root();

            for i in 0..1000 {
                let x = (i % 40) as f32 * 25.0;
                let y = (i / 40) as f32 * 25.0;

                // Real stroke rendering
                let node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(
                        VisualStyle::new()
                            .solid_fill(Vec4::new(0.95, 0.95, 0.95, 1.0))
                            .stroke(StrokeStyle::solid(
                                Paint::Solid(Vec4::new(0.2, 0.4, 0.8, 1.0)),
                                3.0,
                                StrokeAlign::Center,
                            ))
                            .corner_radius(4.0),
                    ),
                });

                let node_id = scene.add_node(root, node);

                if let Some(node) = scene.get_node_mut(node_id) {
                    node.bounds = Rect::new(x, y, 20.0, 20.0);
                }
            }

            black_box(scene);
        });
    });
}

/// Benchmark creating nodes with drop shadows (Phase 2)
fn bench_create_shadowed_nodes(c: &mut Criterion) {
    c.bench_function("create_1000_shadowed_nodes", |b| {
        b.iter(|| {
            let mut scene = Scene::new();
            let root = scene.root();

            for i in 0..1000 {
                let x = (i % 40) as f32 * 25.0;
                let y = (i / 40) as f32 * 25.0;

                // Real drop shadow rendering
                let node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(
                        VisualStyle::new()
                            .solid_fill(Vec4::new(1.0, 1.0, 1.0, 1.0))
                            .drop_shadow(
                                Vec2::new(4.0, 4.0),
                                8.0,
                                Vec4::new(0.0, 0.0, 0.0, 0.4),
                            )
                            .corner_radius(8.0),
                    ),
                });

                let node_id = scene.add_node(root, node);

                if let Some(node) = scene.get_node_mut(node_id) {
                    node.bounds = Rect::new(x, y, 20.0, 20.0);
                }
            }

            black_box(scene);
        });
    });
}

/// Benchmark complex VisualStyle creation (gradient + stroke + shadow)
fn bench_complex_visual_style(c: &mut Criterion) {
    c.bench_function("create_1000_complex_styles", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                // Real complex style with all Phase 2 features
                let style = VisualStyle::new()
                    .fill(Paint::Linear(LinearGradient {
                        start: Vec2::new(0.0, 0.0),
                        end: Vec2::new(1.0, 1.0),
                        stops: vec![
                            ColorStop::new(0.0, Vec4::new(0.3, 0.5, 1.0, 1.0)),
                            ColorStop::new(1.0, Vec4::new(0.6, 0.2, 0.9, 1.0)),
                        ],
                    }))
                    .stroke(StrokeStyle::solid(
                        Paint::Solid(Vec4::new(1.0, 1.0, 1.0, 0.8)),
                        2.0,
                        StrokeAlign::Inside,
                    ))
                    .drop_shadow(
                        Vec2::new(4.0, 4.0),
                        12.0,
                        Vec4::new(0.0, 0.0, 0.0, 0.4),
                    )
                    .corner_radius(12.0)
                    .opacity(0.95);

                black_box(style);
            }
        });
    });
}

/// Benchmark creating nodes with gradient text (Phase 2)
fn bench_create_gradient_text_nodes(c: &mut Criterion) {
    c.bench_function("create_1000_gradient_text_nodes", |b| {
        b.iter(|| {
            let mut scene = Scene::new();
            let root = scene.root();

            for i in 0..1000 {
                let x = (i % 40) as f32 * 25.0;
                let y = (i / 40) as f32 * 25.0;

                // Gradient text with linear gradient (the Phase 2 showstopper!)
                let node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(
                        VisualStyle::new()
                            .fill(Paint::Linear(LinearGradient {
                                start: Vec2::new(0.0, 0.5),
                                end: Vec2::new(1.0, 0.5),
                                stops: vec![
                                    ColorStop::new(0.0, Vec4::new(1.0, 0.2, 0.2, 1.0)), // Red
                                    ColorStop::new(1.0, Vec4::new(0.2, 0.4, 1.0, 1.0)), // Blue
                                ],
                            }))
                            .text(TextContent::new("TEXT", 16.0)),
                    ),
                });

                let node_id = scene.add_node(root, node);

                if let Some(node) = scene.get_node_mut(node_id) {
                    node.bounds = Rect::new(x, y, 50.0, 20.0);
                }
            }

            black_box(scene);
        });
    });
}

criterion_group!(
    benches,
    bench_create_nodes,
    bench_create_rounded_nodes,
    bench_node_lookup,
    bench_node_mutation,
    bench_visual_style_creation,
    bench_scene_iteration,
    bench_visual_iterator,
    // Phase 2 benchmarks
    bench_create_gradient_nodes,
    bench_create_stroked_nodes,
    bench_create_shadowed_nodes,
    bench_complex_visual_style,
    bench_create_gradient_text_nodes,
);

criterion_main!(benches);
