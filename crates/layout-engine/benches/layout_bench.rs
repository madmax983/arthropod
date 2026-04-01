//! Layout engine benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use layout_engine::{FlexDirection, FlexStyle, LayoutConstraints, LayoutEngine};

fn create_flex_tree(size: usize) -> (LayoutEngine, layout_engine::NodeId) {
    let mut engine = LayoutEngine::new();

    let root = engine.create_node(FlexStyle {
        direction: FlexDirection::Column,
        ..Default::default()
    });

    for _ in 0..size {
        let child = engine.create_node(FlexStyle {
            flex_grow: 1.0,
            ..Default::default()
        });
        engine.add_child(root, child);
    }

    (engine, root)
}

fn bench_flexbox_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("flexbox", size), size, |b, &size| {
            let (mut engine, root) = create_flex_tree(size);
            let constraints = LayoutConstraints {
                max_width: Some(800.0),
                max_height: Some(600.0),
                ..Default::default()
            };

            b.iter(|| {
                engine.compute_layout(black_box(root), black_box(constraints));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_flexbox_layout);
criterion_main!(benches);
