//! Text shaping benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use text_engine::TextEngine;

fn bench_font_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("font_loading");

    // Create a dummy font byte slice
    let dummy_font_data: Vec<u8> = vec![0; 10_000_000]; // 10MB dummy font data to simulate large font files

    group.bench_function("register_and_load", |b| {
        b.iter(|| {
            // Need a new engine and clear the global registry each time?
            // Since we can't easily clear the global registry, let's just use cosmic_text::fontdb::Database directly
            // to benchmark the cost of cloning vs Arc.
            let mut db = cosmic_text::fontdb::Database::new();
            db.load_font_data(black_box(dummy_font_data.clone()));
        });
    });

    group.bench_function("register_and_load_arc", |b| {
        let arc_data = Arc::new(dummy_font_data.clone());
        b.iter(|| {
            let mut db = cosmic_text::fontdb::Database::new();
            db.load_font_source(cosmic_text::fontdb::Source::Binary(black_box(
                arc_data.clone(),
            )));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_font_loading);
criterion_main!(benches);
