//! Criterion benchmarks for arthropod-ecs performance
//!
//! Run with: cargo bench -p arthropod-ecs
//! View HTML report at: target/criterion/report/index.html

use arthropod_ecs::{FrameworkContext, ReactiveColor, Renderable};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use flux_state::{Runtime, Signal};
use plat_core::Rect;
use render_engine::{Color, NodeContent, Scene, SceneNode, Transform2D};
use std::rc::Rc;

/// Create a test scene with N rectangles
fn create_test_scene(n: usize) -> (Scene, FrameworkContext, Vec<Rc<Runtime>>) {
    let mut scene = Scene::new();
    let mut context = FrameworkContext::new();
    let root = scene.root();
    let mut runtimes = Vec::new();

    for i in 0..n {
        let runtime = Runtime::new();
        let color_signal = Signal::new(runtime.clone(), Color::RED);
        let (read, _write) = color_signal.split();

        let rect_node = SceneNode {
            content: NodeContent::Rect { color: Color::RED },
            transform: Transform2D::identity(),
            bounds: Rect {
                x: (i % 10) as f32 * 100.0,
                y: (i / 10) as f32 * 100.0,
                width: 90.0,
                height: 90.0,
            },
            children: vec![],
            visible: true,
            opacity: 1.0,
        };

        let node_id = scene.add_node(root, rect_node);
        context
            .spawn(node_id)
            .insert(Renderable)
            .insert(ReactiveColor::new(read));

        runtimes.push(runtime);
    }

    (scene, context, runtimes)
}

/// Benchmark ECS update system (reactive signal polling)
fn benchmark_ecs_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("ecs_update");

    for size in [10, 100, 1000, 10_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut scene, mut context, _runtimes) = create_test_scene(size);

            b.iter(|| {
                context.update(black_box(&mut scene));
            });
        });
    }

    group.finish();
}

/// Benchmark ECS render system (instance collection)
fn benchmark_ecs_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("ecs_render");

    for size in [10, 100, 1000, 10_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (scene, mut context, _runtimes) = create_test_scene(size);

            b.iter(|| {
                let instances = context.render(black_box(&scene));
                black_box(instances);
            });
        });
    }

    group.finish();
}

/// Benchmark full frame pipeline (update + render)
fn benchmark_full_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_frame");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut scene, mut context, _runtimes) = create_test_scene(size);

            b.iter(|| {
                context.update(black_box(&mut scene));
                let instances = context.render(black_box(&scene));
                black_box(instances);
            });
        });
    }

    group.finish();
}

/// Benchmark scene node lookups (O(1) HashMap access)
fn benchmark_scene_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_lookups");

    for size in [100, 1000, 10_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (scene, _context, _runtimes) = create_test_scene(size);
            let node_ids: Vec<_> = scene.nodes().map(|(id, _)| id).collect();

            b.iter(|| {
                for &node_id in &node_ids {
                    let _node = scene.get(black_box(node_id));
                    black_box(_node);
                }
            });
        });
    }

    group.finish();
}

/// Benchmark entity spawning
fn benchmark_entity_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_spawn");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut scene = Scene::new();
                    let context = FrameworkContext::new();
                    let root = scene.root();
                    let runtime = Runtime::new();

                    let mut node_ids = Vec::new();
                    for i in 0..size {
                        let rect_node = SceneNode {
                            content: NodeContent::Rect { color: Color::RED },
                            transform: Transform2D::identity(),
                            bounds: Rect {
                                x: (i % 10) as f32 * 100.0,
                                y: (i / 10) as f32 * 100.0,
                                width: 90.0,
                                height: 90.0,
                            },
                            children: vec![],
                            visible: true,
                            opacity: 1.0,
                        };
                        let node_id = scene.add_node(root, rect_node);
                        node_ids.push(node_id);
                    }

                    (context, runtime, node_ids)
                },
                |(mut context, runtime, node_ids)| {
                    for node_id in node_ids {
                        let color_signal = Signal::new(runtime.clone(), Color::RED);
                        let (read, _write) = color_signal.split();

                        context
                            .spawn(black_box(node_id))
                            .insert(Renderable)
                            .insert(ReactiveColor::new(read));
                    }
                    black_box(context);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_ecs_update,
    benchmark_ecs_render,
    benchmark_full_frame,
    benchmark_scene_lookups,
    benchmark_entity_spawn
);
criterion_main!(benches);
