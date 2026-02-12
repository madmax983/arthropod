use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use render_engine::backend::wgpu::effects::{
    backdrop_capture_bounds, blend_multiply, blend_overlay, blend_screen, gaussian_kernel_1d,
};

fn bench_blur_pass_1080p(c: &mut Criterion) {
    c.bench_function("blur_pass_1080p", |b| {
        b.iter(|| {
            let kernel = gaussian_kernel_1d(18.0, 25);
            let mut acc = 0.0f32;
            for _ in 0..(1920 * 1080 / 1024) {
                for w in &kernel {
                    acc += *w;
                }
            }
            black_box(acc);
        });
    });
}

fn bench_background_blur_500_nodes(c: &mut Criterion) {
    c.bench_function("background_blur_500_nodes", |b| {
        b.iter(|| {
            let frame = [0, 0, 1920, 1080];
            let mut total_area = 0u64;
            for i in 0..500u32 {
                let x = (i * 13) % 1800;
                let y = (i * 17) % 1000;
                let bounds = backdrop_capture_bounds([x, y, 120, 80], 12.0, frame);
                total_area += (bounds[2] as u64) * (bounds[3] as u64);
            }
            black_box(total_area);
        });
    });
}

fn bench_blend_composite_1000_layers(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("blend_composite_layers", 1000),
        &1000usize,
        |b, &layers| {
            b.iter(|| {
                let mut dst = [0.1f32, 0.1, 0.1];
                for i in 0..layers {
                    let t = (i as f32) / (layers as f32);
                    let src = [t, 1.0 - t * 0.5, 0.2 + 0.7 * t];
                    dst = match i % 3 {
                        0 => blend_multiply(src, dst),
                        1 => blend_screen(src, dst),
                        _ => blend_overlay(src, dst),
                    };
                }
                black_box(dst);
            });
        },
    );
}

criterion_group!(
    phase4_effects,
    bench_blur_pass_1080p,
    bench_background_blur_500_nodes,
    bench_blend_composite_1000_layers
);
criterion_main!(phase4_effects);
