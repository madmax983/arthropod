use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use glam::Vec2;
use style_engine::{BooleanOp, PathCommand, VectorPath, WindingRule};

fn regular_polygon_path(sides: usize, radius: f32) -> VectorPath {
    let mut path = VectorPath::new();
    if sides < 3 {
        return path;
    }

    let two_pi = std::f32::consts::TAU;
    for i in 0..sides {
        let t = i as f32 / sides as f32;
        let angle = t * two_pi;
        let p = Vec2::new(radius * angle.cos(), radius * angle.sin());
        if i == 0 {
            path.commands.push(PathCommand::MoveTo(p));
        } else {
            path.commands.push(PathCommand::LineTo(p));
        }
    }
    path.commands.push(PathCommand::Close);
    path
}

fn concentric_donut_path(
    outer_sides: usize,
    inner_sides: usize,
    outer_r: f32,
    inner_r: f32,
) -> VectorPath {
    let mut path = regular_polygon_path(outer_sides, outer_r);

    let two_pi = std::f32::consts::TAU;
    for i in 0..inner_sides {
        let t = i as f32 / inner_sides as f32;
        let angle = t * two_pi;
        let p = Vec2::new(inner_r * angle.cos(), inner_r * angle.sin());
        if i == 0 {
            path.commands.push(PathCommand::MoveTo(p));
        } else {
            path.commands.push(PathCommand::LineTo(p));
        }
    }
    path.commands.push(PathCommand::Close);
    path.winding_rule = WindingRule::EvenOdd;
    path
}

fn bench_boolean_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_boolean_ops");
    group.sample_size(40);

    for sides in [32usize, 128, 512] {
        let a = regular_polygon_path(sides, 100.0);
        let b = regular_polygon_path(sides, 85.0);
        group.bench_with_input(BenchmarkId::new("union", sides), &sides, |bench, _| {
            bench.iter(|| {
                let out = VectorPath::boolean_op(black_box(&a), black_box(&b), BooleanOp::Union)
                    .expect("union should succeed");
                black_box(out.commands.len());
            });
        });
        group.bench_with_input(BenchmarkId::new("intersect", sides), &sides, |bench, _| {
            bench.iter(|| {
                let out =
                    VectorPath::boolean_op(black_box(&a), black_box(&b), BooleanOp::Intersect)
                        .expect("intersect should succeed");
                black_box(out.commands.len());
            });
        });
        group.bench_with_input(BenchmarkId::new("subtract", sides), &sides, |bench, _| {
            bench.iter(|| {
                let out = VectorPath::boolean_op(black_box(&a), black_box(&b), BooleanOp::Subtract)
                    .expect("subtract should succeed");
                black_box(out.commands.len());
            });
        });
    }
    group.finish();
}

fn bench_contains_point(c: &mut Criterion) {
    let donut = concentric_donut_path(256, 128, 120.0, 50.0);
    let inside = (80.0f32, 0.0f32);
    let hole = (25.0f32, 0.0f32);
    let outside = (180.0f32, 0.0f32);

    let mut group = c.benchmark_group("path_hit_test");
    group.sample_size(100);
    group.bench_function("contains_point_inside", |bench| {
        bench.iter(|| black_box(donut.contains_point(inside.0, inside.1)));
    });
    group.bench_function("contains_point_hole", |bench| {
        bench.iter(|| black_box(donut.contains_point(hole.0, hole.1)));
    });
    group.bench_function("contains_point_outside", |bench| {
        bench.iter(|| black_box(donut.contains_point(outside.0, outside.1)));
    });
    group.finish();
}

fn bench_svg_parse(c: &mut Criterion) {
    let d = "M 20 20 C 20 10 45 10 45 20 S 70 30 70 20 A 20 20 0 0 1 110 20 L 130 40 Q 150 60 130 80 T 90 110 Z";
    c.bench_function("svg_parse_complex_path_d", |bench| {
        bench.iter(|| {
            let path =
                VectorPath::from_svg_path_data(black_box(d)).expect("svg parse should succeed");
            black_box(path.commands.len());
        });
    });
}

criterion_group!(
    benches,
    bench_boolean_ops,
    bench_contains_point,
    bench_svg_parse
);
criterion_main!(benches);
