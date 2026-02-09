//! Benchmarks for Phase 1 Primitive Rendering Pipeline
//!
//! Performance requirements:
//! - 1000 solid primitives: < 200μs
//! - Scene node operations: < 10μs
//! - Instance creation: < 100μs for 1000 nodes

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use plat_core::Rect;
use render_engine::{Color, NodeContent, Paint, Scene, SceneNode, VisualStyle};

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
                    style: VisualStyle::new().solid_fill(Color::rgba(0.5, 0.5, 0.5, 1.0).as_vec4()),
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
                    style: VisualStyle::new()
                        .solid_fill(Color::rgba(0.5, 0.5, 0.5, 1.0).as_vec4())
                        .corner_radius(radius),
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

    for i in 0..1000 {
        let node = SceneNode::new(NodeContent::Styled {
            style: VisualStyle::new().solid_fill(Color::RED.as_vec4()),
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
            style: VisualStyle::new().solid_fill(Color::RED.as_vec4()),
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
                style: VisualStyle::new().solid_fill(Color::RED.as_vec4()),
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
                style: VisualStyle::new().solid_fill(Color::RED.as_vec4()),
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

criterion_group!(
    benches,
    bench_create_nodes,
    bench_create_rounded_nodes,
    bench_node_lookup,
    bench_node_mutation,
    bench_visual_style_creation,
    bench_scene_iteration,
    bench_visual_iterator,
);

criterion_main!(benches);
