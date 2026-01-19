//! Text shaping benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use text_engine::TextEngine;

fn bench_text_shaping(c: &mut Criterion) {
    let mut group = c.benchmark_group("shaping");

    let texts = vec![
        ("short", "Hello"),
        ("medium", "Hello World, this is a test!"),
        ("long", "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs."),
    ];

    for (name, text) in texts {
        group.bench_with_input(
            BenchmarkId::new("shape", name),
            &text,
            |b, &text| {
                let mut engine = TextEngine::new();

                b.iter(|| {
                    engine.shape_text(black_box(text), black_box(16.0));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_text_shaping);
criterion_main!(benches);
