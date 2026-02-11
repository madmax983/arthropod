//! Benchmarks for Phase 3 tessellated path pipeline.
//!
//! Targets from docs/plans/2026-02-08-figma-rendering-pipeline-design.md:
//! - Tessellation (100 segments): < 500us
//! - Tessellation cache hit: < 100ns

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use glam::Vec2;
use render_engine::backend::wgpu::pipelines::path_pipeline::{TessellationCache, tessellate_fill};
use style_engine::{PathCommand, StrokeAlign, StrokeStyle, VectorPath};
use std::collections::HashMap;

fn circle_like_path(segments: usize, radius: f32) -> VectorPath {
    let mut path = VectorPath::new();
    if segments < 3 {
        return path;
    }

    let two_pi = std::f32::consts::PI * 2.0;
    for i in 0..segments {
        let t = i as f32 / segments as f32;
        let angle = t * two_pi;
        let x = radius * angle.cos();
        let y = radius * angle.sin();

        if i == 0 {
            path.commands.push(PathCommand::MoveTo(Vec2::new(x, y)));
        } else {
            path.commands.push(PathCommand::LineTo(Vec2::new(x, y)));
        }
    }
    path.commands.push(PathCommand::Close);
    path
}

fn bench_tessellate_complex_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_tessellation_fill");
    group.sample_size(100);

    for segments in [32usize, 64, 100, 256] {
        let path = circle_like_path(segments, 100.0);
        group.bench_with_input(BenchmarkId::from_parameter(segments), &segments, |b, _| {
            b.iter(|| {
                let mesh = tessellate_fill(black_box(&path)).expect("tessellation should succeed");
                black_box(mesh);
            });
        });
    }

    group.finish();
}

fn bench_tessellation_cache_hit(c: &mut Criterion) {
    let path = circle_like_path(100, 120.0);
    let key = TessellationCache::fill_key(&path);
    let mut cache = TessellationCache::new(1024);
    cache
        .get_or_tessellate_fill_with_key(key, &path)
        .expect("cache warm-up tessellation should succeed");

    c.bench_function("path_tessellation_cache_hit_fill_100_segments", |b| {
        b.iter(|| {
            let mesh = cache
                .get_or_tessellate_fill(black_box(&path))
                .expect("cache hit should succeed");
            black_box(mesh);
        });
    });

    c.bench_function("path_tessellation_cache_hit_fill_100_segments_keyed", |b| {
        b.iter(|| {
            let mesh = cache
                .get_or_tessellate_fill_with_key(black_box(key), black_box(&path))
                .expect("cache hit should succeed");
            black_box(mesh);
        });
    });
}

#[derive(Clone, Copy)]
struct BenchFingerprint {
    len: usize,
    first: u64,
    last: u64,
}

#[derive(Clone, Copy)]
struct BenchInternerEntry {
    hash: u64,
    fp: BenchFingerprint,
}

#[derive(Default)]
struct BenchPathInterner {
    by_ptr: HashMap<usize, BenchInternerEntry>,
}

impl BenchPathInterner {
    fn command_fp(command: PathCommand) -> u64 {
        match command {
            PathCommand::MoveTo(p) => 0x01 ^ ((p.x.to_bits() as u64) << 1) ^ (p.y.to_bits() as u64).rotate_left(11),
            PathCommand::LineTo(p) => 0x02 ^ ((p.x.to_bits() as u64) << 1) ^ (p.y.to_bits() as u64).rotate_left(11),
            PathCommand::QuadraticTo { control, to } => {
                0x03
                    ^ ((control.x.to_bits() as u64) << 1)
                    ^ (control.y.to_bits() as u64).rotate_left(7)
                    ^ (to.x.to_bits() as u64).rotate_left(17)
                    ^ (to.y.to_bits() as u64).rotate_left(29)
            }
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => {
                0x04
                    ^ ((control1.x.to_bits() as u64) << 1)
                    ^ (control1.y.to_bits() as u64).rotate_left(5)
                    ^ (control2.x.to_bits() as u64).rotate_left(13)
                    ^ (control2.y.to_bits() as u64).rotate_left(19)
                    ^ (to.x.to_bits() as u64).rotate_left(23)
                    ^ (to.y.to_bits() as u64).rotate_left(31)
            }
            PathCommand::Close => 0x05,
        }
    }

    fn fingerprint(path: &VectorPath) -> BenchFingerprint {
        let first = path.commands.first().copied().map(Self::command_fp).unwrap_or(0);
        let last = path.commands.last().copied().map(Self::command_fp).unwrap_or(0);
        BenchFingerprint {
            len: path.commands.len(),
            first,
            last,
        }
    }

    fn hash_for(&mut self, path: &VectorPath) -> u64 {
        let ptr = path as *const VectorPath as usize;
        let fp = Self::fingerprint(path);
        if let Some(entry) = self.by_ptr.get(&ptr) {
            if entry.fp.len == fp.len && entry.fp.first == fp.first && entry.fp.last == fp.last {
                return entry.hash;
            }
        }
        let hash = TessellationCache::fill_key(path);
        self.by_ptr.insert(ptr, BenchInternerEntry { hash, fp });
        hash
    }
}

fn bench_frame_like_collection(c: &mut Criterion) {
    let unique_paths: Vec<VectorPath> = (0..128)
        .map(|i| circle_like_path(24 + (i % 40), 50.0 + (i as f32 * 0.5)))
        .collect();

    // Simulate many nodes reusing a smaller set of path allocations.
    let mut frame_paths: Vec<&VectorPath> = Vec::with_capacity(4096);
    for i in 0..4096 {
        frame_paths.push(&unique_paths[i % unique_paths.len()]);
    }

    let mut cache_hash_each = TessellationCache::new(4096);
    let mut cache_interned = TessellationCache::new(4096);
    let mut interner = BenchPathInterner::default();

    // Warm both caches to isolate key generation + lookup hot path.
    for path in &frame_paths {
        let key_a = TessellationCache::fill_key(path);
        let _ = cache_hash_each.get_or_tessellate_fill_with_key(key_a, path);

        let key_b = interner.hash_for(path);
        let _ = cache_interned.get_or_tessellate_fill_with_key(key_b, path);
    }

    let mut group = c.benchmark_group("frame_like_fill_collection");
    group.sample_size(80);

    group.bench_function("hash_each_time", |b| {
        b.iter(|| {
            let mut touched = 0usize;
            for path in &frame_paths {
                let key = TessellationCache::fill_key(path);
                let mesh = cache_hash_each
                    .get_or_tessellate_fill_with_key(key, path)
                    .expect("cache hit should succeed");
                touched = touched.wrapping_add(mesh.indices.len());
            }
            black_box(touched);
        });
    });

    group.bench_function("interned_hash", |b| {
        b.iter(|| {
            let mut touched = 0usize;
            for path in &frame_paths {
                let key = interner.hash_for(path);
                let mesh = cache_interned
                    .get_or_tessellate_fill_with_key(key, path)
                    .expect("cache hit should succeed");
                touched = touched.wrapping_add(mesh.indices.len());
            }
            black_box(touched);
        });
    });

    group.finish();
}

fn bench_tessellation_miss_heavy(c: &mut Criterion) {
    let paths: Vec<VectorPath> = (0..512)
        .map(|i| circle_like_path(48 + (i % 32), 60.0 + i as f32 * 0.2))
        .collect();
    let keys: Vec<u64> = paths.iter().map(TessellationCache::fill_key).collect();

    let mut group = c.benchmark_group("path_tessellation_miss_heavy");
    group.sample_size(50);

    group.bench_function("fill_miss_512_paths_cache_16", |b| {
        b.iter(|| {
            let mut cache = TessellationCache::new(16);
            let mut acc = 0usize;
            for (path, key) in paths.iter().zip(keys.iter()) {
                let mesh = cache
                    .get_or_tessellate_fill_with_key(*key, path)
                    .expect("miss tessellation should succeed");
                acc = acc.wrapping_add(mesh.indices.len());
            }
            black_box(acc);
        });
    });

    // Reused path geometry but many distinct stroke styles (mostly miss on stroke keys).
    // This reflects "not-hot" style churn while geometry is stable.
    let stroke_styles: Vec<StrokeStyle> = (0..8)
        .map(|i| {
            let width = 1.0 + i as f32;
            StrokeStyle::solid(
                style_engine::Paint::solid(glam::Vec4::new(0.2 + i as f32 * 0.05, 0.6, 0.9, 1.0)),
                width,
                StrokeAlign::Center,
            )
        })
        .collect();

    group.bench_function("stroke_miss_reused_paths_cache_32", |b| {
        b.iter(|| {
            let mut cache = TessellationCache::new(32);
            let mut acc = 0usize;
            for path in &paths {
                let path_hash = TessellationCache::fill_key(path);
                for stroke in &stroke_styles {
                    let key = TessellationCache::stroke_key_from_path_hash(path_hash, stroke);
                    let mesh = cache
                        .get_or_tessellate_stroke_with_key(key, path, stroke)
                        .expect("stroke miss tessellation should succeed");
                    acc = acc.wrapping_add(mesh.indices.len());
                }
            }
            black_box(acc);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tessellate_complex_path,
    bench_tessellation_cache_hit,
    bench_frame_like_collection,
    bench_tessellation_miss_heavy,
);
criterion_main!(benches);
