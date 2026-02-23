//! Criterion benchmarks for arthropod-ecs performance
//!
//! Run with: cargo bench -p arthropod-ecs
//! View HTML report at: target/criterion/report/index.html

use arthropod_ecs::{
    FrameworkContext, ReactiveColor, ReactiveOpacity, ReactiveTransform, Renderable,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use flux_state::{Runtime, Signal};
use plat_core::Rect;
use render_engine::{Color, NodeContent, Scene, SceneNode, Transform2D, VisualStyle};
use std::sync::Arc;

/// Create a test scene with N rectangles (color-only reactive components)
fn create_test_scene(n: usize) -> (FrameworkContext, Vec<Arc<Runtime>>) {
    let mut context = FrameworkContext::new();
    let mut runtimes = Vec::new();

    // Access Scene from World to add nodes
    let root = {
        let scene = context.world().resource::<Scene>();
        scene.root()
    };

    for i in 0..n {
        let runtime = Runtime::new();
        let color_signal = Signal::new(runtime.clone(), Color::RED);
        let (read, _write) = color_signal.split();

        let rect_node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
            },
            transform: Transform2D::identity(),
            bounds: Rect {
                x: (i % 10) as f32 * 100.0,
                y: (i / 10) as f32 * 100.0,
                width: 90.0,
                height: 90.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };

        let node_id = {
            let mut scene = context.world_mut().resource_mut::<Scene>();
            scene.add_node(root, rect_node)
        };

        context
            .spawn(node_id)
            .insert(Renderable)
            .insert(ReactiveColor::new(read));

        runtimes.push(runtime);
    }

    (context, runtimes)
}

/// Create a test scene with all three reactive component types
fn create_full_reactive_scene(n: usize) -> (FrameworkContext, Vec<Arc<Runtime>>) {
    let mut context = FrameworkContext::new();
    let mut runtimes = Vec::new();

    let root = {
        let scene = context.world().resource::<Scene>();
        scene.root()
    };

    for i in 0..n {
        let runtime = Runtime::new();

        let color_signal = Signal::new(runtime.clone(), Color::RED);
        let (color_read, _) = color_signal.split();

        let transform_signal = Signal::new(runtime.clone(), Transform2D::identity());
        let (transform_read, _) = transform_signal.split();

        let opacity_signal = Signal::new(runtime.clone(), 1.0_f32);
        let (opacity_read, _) = opacity_signal.split();

        let rect_node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
            },
            transform: Transform2D::identity(),
            bounds: Rect {
                x: (i % 10) as f32 * 100.0,
                y: (i / 10) as f32 * 100.0,
                width: 90.0,
                height: 90.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };

        let node_id = {
            let mut scene = context.world_mut().resource_mut::<Scene>();
            scene.add_node(root, rect_node)
        };

        context
            .spawn(node_id)
            .insert(Renderable)
            .insert(ReactiveColor::new(color_read))
            .insert(ReactiveTransform::new(transform_read))
            .insert(ReactiveOpacity::new(opacity_read));

        runtimes.push(runtime);
    }

    (context, runtimes)
}

/// Benchmark ECS update system (reactive signal polling)
fn benchmark_ecs_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("ecs_update");

    for size in [10, 100, 1000, 10_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut context, _runtimes) = create_test_scene(size);

            b.iter(|| {
                context.update();
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
            let (mut context, _runtimes) = create_test_scene(size);

            b.iter(|| {
                let instances = context.render();
                black_box(instances);
            });
        });
    }

    group.finish();
}

/// Benchmark full frame pipeline (update + render)
fn benchmark_full_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_frame");

    for size in [10, 100, 1000, 10_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut context, _runtimes) = create_test_scene(size);

            b.iter(|| {
                context.update();
                let instances = context.render();
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
            let (context, _runtimes) = create_test_scene(size);

            // Collect node IDs from the scene
            let node_ids: Vec<_> = {
                let scene = context.world().resource::<Scene>();
                scene.nodes().map(|(id, _)| id).collect()
            };

            b.iter(|| {
                let scene = context.world().resource::<Scene>();
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
                    let mut context = FrameworkContext::new();
                    let runtime = Runtime::new();

                    // Create scene nodes
                    let root = {
                        let scene = context.world().resource::<Scene>();
                        scene.root()
                    };

                    let mut node_ids = Vec::new();
                    for i in 0..size {
                        let rect_node = SceneNode {
                            content: NodeContent::Styled {
                                style: Box::new(
                                    VisualStyle::new().solid_fill(Color::RED.as_vec4()),
                                ),
                            },
                            transform: Transform2D::identity(),
                            bounds: Rect {
                                x: (i % 10) as f32 * 100.0,
                                y: (i / 10) as f32 * 100.0,
                                width: 90.0,
                                height: 90.0,
                            },
                            children: vec![],
                            parent: None,
                            visible: true,
                            opacity: 1.0,
                        };

                        let node_id = {
                            let mut scene = context.world_mut().resource_mut::<Scene>();
                            scene.add_node(root, rect_node)
                        };

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

// --- Phase 0: Per-system breakdown benchmarks ---

/// Benchmark only the reactive color update system (isolated)
fn benchmark_reactive_color_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_system/reactive_color");

    for size in [100, 1000, 10_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut context, _runtimes) = create_test_scene(size);

            b.iter(|| {
                context.update();
            });
        });
    }

    group.finish();
}

/// Benchmark update with all three reactive component types
fn benchmark_all_reactive(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_system/all_reactive");

    for size in [100, 1000, 10_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut context, _runtimes) = create_full_reactive_scene(size);

            b.iter(|| {
                context.update();
            });
        });
    }

    group.finish();
}

/// Benchmark render collection at high entity counts
fn benchmark_render_high_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_system/render_collection");

    for size in [1000, 10_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut context, _runtimes) = create_test_scene(size);

            b.iter(|| {
                let instances = context.render();
                black_box(instances);
            });
        });
    }

    group.finish();
}

/// Benchmark full frame at high entity counts (10K, 50K) for parallelization crossover
fn benchmark_full_frame_high_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_system/full_frame_high");

    for size in [10_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut context, _runtimes) = create_full_reactive_scene(size);

            b.iter(|| {
                context.update();
                let instances = context.render();
                black_box(instances);
            });
        });
    }

    group.finish();
}

/// Benchmark at ultra-high scales to find parallelization crossover (100K+)
fn benchmark_crossover_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("crossover/full_frame");

    // Increase sample size for more stable measurements at high scales
    group.sample_size(50);

    // Test at various scales to find where parallelism becomes beneficial
    for size in [1_000, 5_000, 10_000, 25_000, 50_000, 100_000, 200_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut context, _runtimes) = create_full_reactive_scene(size);

            b.iter(|| {
                context.update();
                let instances = context.render();
                black_box(instances);
            });
        });
    }

    group.finish();
}

/// Benchmark reactive system scaling to find optimal gather-apply threshold
fn benchmark_reactive_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("crossover/reactive");
    group.sample_size(50);

    for size in [
        100, 500, 1_000, 2_500, 5_000, 10_000, 25_000, 50_000, 100_000,
    ]
    .iter()
    {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut context, _runtimes) = create_full_reactive_scene(size);

            b.iter(|| {
                context.update();
            });
        });
    }

    group.finish();
}

/// Benchmark render collection scaling to find optimal parallel threshold
fn benchmark_render_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("crossover/render");
    group.sample_size(50);

    for size in [100, 256, 500, 1_000, 2_500, 5_000, 10_000, 25_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut context, _runtimes) = create_test_scene(size);
            // Run update first to populate render commands
            context.update();

            b.iter(|| {
                let instances = context.render();
                black_box(instances);
            });
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
    benchmark_entity_spawn,
);

criterion_group!(
    parallelization_baselines,
    benchmark_reactive_color_only,
    benchmark_all_reactive,
    benchmark_render_high_count,
    benchmark_full_frame_high_count,
);

criterion_group!(
    crossover_analysis,
    benchmark_crossover_analysis,
    benchmark_reactive_scaling,
    benchmark_render_scaling,
);

criterion_main!(benches, parallelization_baselines, crossover_analysis);
