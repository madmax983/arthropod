//! Scene Graph Benchmarks
//!
//! Benchmarks for scene graph operations including hit testing.
//!
//! Performance targets:
//! - hit_test on 1k nodes < 100μs
//! - hit_test on 10k nodes < 1ms

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use plat_core::Rect;
use render_engine::{Color, NodeContent, Scene, SceneNode};

/// Create a scene with many non-overlapping nodes
fn create_scene_with_nodes(count: usize) -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    let grid_size = (count as f32).sqrt().ceil() as usize;
    let node_size = 10.0;
    let spacing = 15.0;

    for i in 0..count {
        let row = i / grid_size;
        let col = i % grid_size;

        let mut node = SceneNode::new(NodeContent::Rect { color: Color::RED });
        node.bounds = Rect::new(
            col as f32 * spacing,
            row as f32 * spacing,
            node_size,
            node_size,
        );
        scene.add_node(root, node);
    }

    scene
}

/// Create a scene with overlapping nodes (worst case for hit testing)
fn create_scene_with_overlapping_nodes(count: usize) -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    for i in 0..count {
        let mut node = SceneNode::new(NodeContent::Rect { color: Color::RED });
        // All nodes overlap at center with slight offset
        node.bounds = Rect::new(
            100.0 + (i as f32 * 0.1),
            100.0 + (i as f32 * 0.1),
            50.0,
            50.0,
        );
        scene.add_node(root, node);
    }

    scene
}

/// Benchmark hit_test with varying scene sizes
fn bench_hit_test_varying_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_hit_test");

    for size in [100, 500, 1000, 5000, 10000].iter() {
        let scene = create_scene_with_nodes(*size);

        // Test hit on node in middle of scene
        let grid_size = (*size as f32).sqrt().ceil() as usize;
        let mid = grid_size / 2;
        let test_x = mid as f32 * 15.0 + 5.0;
        let test_y = mid as f32 * 15.0 + 5.0;

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(scene.hit_test(test_x, test_y));
            });
        });
    }

    group.finish();
}

/// Benchmark hit_test miss (no node at position)
fn bench_hit_test_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_hit_test_miss");

    for size in [100, 1000, 10000].iter() {
        let scene = create_scene_with_nodes(*size);

        // Test position where no node exists
        let test_x = 99999.0;
        let test_y = 99999.0;

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(scene.hit_test(test_x, test_y));
            });
        });
    }

    group.finish();
}

/// Benchmark hit_test with overlapping nodes (worst case)
fn bench_hit_test_overlapping(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_hit_test_overlapping");

    for size in [100, 500, 1000].iter() {
        let scene = create_scene_with_overlapping_nodes(*size);

        // Test in the overlap region
        let test_x = 125.0;
        let test_y = 125.0;

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(scene.hit_test(test_x, test_y));
            });
        });
    }

    group.finish();
}

/// Benchmark node lookup by ID
fn bench_node_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_node_lookup");

    for size in [100, 1000, 10000].iter() {
        let scene = create_scene_with_nodes(*size);

        // Get a node ID from the middle
        let nodes: Vec<_> = scene.nodes().map(|(id, _)| id).collect();
        let target_id = nodes[nodes.len() / 2];

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(scene.get_node(target_id));
            });
        });
    }

    group.finish();
}

/// Benchmark adding nodes to scene
fn bench_add_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_add_node");

    group.bench_function("single_add", |b| {
        let mut scene = Scene::new();
        let root = scene.root();

        b.iter(|| {
            let node = SceneNode::new(NodeContent::Rect { color: Color::RED });
            black_box(scene.add_node(root, node));
        });
    });

    group.bench_function("batch_100", |b| {
        b.iter(|| {
            let mut scene = Scene::new();
            let root = scene.root();

            for _ in 0..100 {
                let node = SceneNode::new(NodeContent::Rect { color: Color::RED });
                scene.add_node(root, node);
            }

            black_box(&scene);
        });
    });

    group.finish();
}

/// Benchmark marking nodes dirty
fn bench_mark_dirty(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_mark_dirty");

    let mut scene = create_scene_with_nodes(1000);
    let nodes: Vec<_> = scene.nodes().map(|(id, _)| id).collect();

    group.bench_function("mark_single", |b| {
        let target = nodes[500];
        b.iter(|| {
            scene.mark_dirty(target);
        });
    });

    group.bench_function("take_dirty", |b| {
        // First mark some as dirty
        for i in 0..100 {
            scene.mark_dirty(nodes[i]);
        }

        b.iter(|| {
            black_box(scene.take_dirty());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_hit_test_varying_sizes,
    bench_hit_test_miss,
    bench_hit_test_overlapping,
    bench_node_lookup,
    bench_add_node,
    bench_mark_dirty,
);
criterion_main!(benches);
