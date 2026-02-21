# Figma Pipeline Checklist (Phase 1-3) + Benchmarks

Date: 2026-02-10
Source plan: `docs/plans/2026-02-08-figma-rendering-pipeline-design.md`

## Summary

- Phase 1: largely implemented (primitive pipeline, gradients, per-corner radii, strokes, glyph path in primitive shader).
- Phase 2: largely implemented in `style-engine` and render integration.
- Phase 3: functional deliverables are implemented (tessellation pipeline/cache, path rendering integration, boolean ops, hit testing, SVG path parsing, benchmark coverage).
- Phase 4/5: not started in render-engine backend code paths.

## Checklist by planned deliverable

### Phase 1

- `PrimitiveInstance` extended instance type: implemented (`crates/render-engine/src/backend/wgpu/pipelines/primitive_pipeline.rs`).
- `primitive.wgsl` unified shader path for shape + glyph: implemented (`crates/render-engine/src/backend/shaders/primitive.wgsl`).
- Per-corner radii SDF and stroke params: implemented.
- Gradient families (linear/radial/angular/diamond) in primitive pipeline: implemented.
- Drop shadow fast-path instance generation: implemented.
- Gradient text through primitive path (glyph alpha * fill): implemented.
- `RectPipeline`/`GlyphPipeline` removal and primitive pipeline unification: implemented in active backend pipeline path.

### Phase 2

- `style-engine` crate and `VisualStyle` unified model: implemented.
- `VisualStyle` includes `corner_smoothing`, `clips_content`, `fill_geometry`, `stroke_geometry`, `text`: implemented (`crates/style-engine/src/visual.rs`).
- Figma-like paint model and stroke model in public types: implemented.
- Multi-layer stroke paint compositing is implemented (all stroke paint layers render bottom-to-top).

### Phase 3

- Lyon integration: implemented (`crates/render-engine/Cargo.toml`).
- `PathPipeline`: implemented (`crates/render-engine/src/backend/wgpu/pipelines/path_pipeline.rs`).
- `path.wgsl`: implemented (`crates/render-engine/src/backend/shaders/path.wgsl`).
- `TessellationCache` (LRU-style bounded cache): implemented.
- `fill_geometry` routing to path batches: implemented (`crates/render-engine/src/backend/wgpu/mod.rs`).
- `stroke_geometry` routing/tessellation: implemented.
- Per-path paint sampling (including gradients) for tessellated paths: implemented.
- Boolean operations on paths (union/subtract/intersect/exclude): implemented (`crates/style-engine/src/path.rs`).
- Point-in-path hit testing: implemented (`VectorPath::contains_point` in `crates/style-engine/src/path.rs`).
- SVG path parsing (`d` attribute): implemented with command support `M/m`, `L/l`, `H/h`, `V/v`, `Q/q`, `T/t`, `C/c`, `S/s`, `A/a`, `Z/z`.
- Arc flattening quality options: implemented via `SvgParseOptions` (`max_arc_step_degrees`).
- Integration smoke example: implemented (`examples/phase3_path_smoke.rs`).

Phase 3 caveat:
- Boolean operations currently flatten curves to polyline contours before polygon boolean execution. This is robust and practical, but not exact curve-curve boolean math.

## Phase 1-3 verification snapshot

- `cargo check -p render-engine`: pass.
- Targeted tests (path routing/cache/gradient sampling): pass.
- `cargo check --example visual_test_phase2`: pass.

## Benchmark additions in this pass

Added `crates/render-engine/benches/path_pipeline.rs` with:

- `path_tessellation_fill/{32,64,100,256}`:
  - measures fill tessellation cost by path segment count.
- `path_tessellation_cache_hit_fill_100_segments`:
  - measures cache-hit lookup + clone path.
- `path_tessellation_cache_hit_fill_100_segments_keyed`:
  - measures cache-hit lookup when the path hash key is precomputed.
- `frame_like_fill_collection/{hash_each_time,interned_hash}`:
  - frame-like hot-path benchmark to quantify path-hash interning impact in collection loops.
- `path_tessellation_miss_heavy/{fill_miss_512_paths_cache_16,stroke_miss_reused_paths_cache_32}`:
  - miss-path/cold-path benchmark coverage.

Benchmark target registered in `crates/render-engine/Cargo.toml` as:

- `[[bench]] name = "path_pipeline"`.

Added `crates/style-engine/benches/path_ops.rs` with:

- `path_boolean_ops/{union,intersect,subtract}/{32,128,512}`:
  - CPU-side boolean op throughput for representative contour complexity.
- `path_hit_test/{contains_point_inside,contains_point_hole,contains_point_outside}`:
  - point-in-path performance on a donut-like path (EvenOdd).
- `svg_parse_complex_path_d`:
  - end-to-end parsing throughput for mixed command SVG `d` strings (including curves/arcs).

Benchmark target registered in `crates/style-engine/Cargo.toml` as:

- `[[bench]] name = "path_ops"`.

CI guardrails added for style-engine path ops:

- `.github/scripts/check-style-path-bench.sh`
- `path_boolean_ops/intersect/128` median threshold: `<= 50,000 ns`
- `path_hit_test/contains_point_inside` median threshold: `<= 5,000 ns`
- wired into both `.github/workflows/ci.yml` and `.github/workflows/pr-checks.yml`

## Notes on plan alignment

- Current implementation is aligned with the plan's architecture direction (unified primitive pipeline + additive path pipeline).
- Path gradient rendering now uses shader-evaluated gradient atlas sampling in `path.wgsl`:
  - `PathPipeline` provides UV + gradient metadata per vertex and uploads gradient params/atlas for fragment evaluation.
  - This removes the former CPU per-vertex gradient color baking deviation.

## 2026-02-10 Optimization Update (cache hits)

- `TessellationCache` now stores meshes as `Arc<PathMesh>` to avoid cloning mesh vectors on cache hits.
- Cache lookups now support keyed APIs:
  - `get_or_tessellate_fill_with_key(...)`
  - `get_or_tessellate_stroke_with_key(...)`
  - `fill_key(...)`, `stroke_key(...)`, and `stroke_key_from_path_hash(...)`
- `WgpuBackend` path collection computes path hash once and reuses it for fill and stroke-key derivation.
- Added backend path-hash interner for frame-local pointer-stable reuse.
- Added `warm_path_cache(&Scene)` and cache telemetry (`hits`, `misses`, `evictions`).

### Latest measurements

- Non-keyed cache-hit benchmark:
  - `path_tessellation_cache_hit_fill_100_segments`: ~29.9 ns
- Keyed cache-hit benchmark:
  - `path_tessellation_cache_hit_fill_100_segments_keyed`: ~20 ns
- Frame-like collection benchmark:
  - `frame_like_fill_collection/hash_each_time`: ~518 us
  - `frame_like_fill_collection/interned_hash`: ~141 us
  - ~3.7x faster with interning
- Miss-heavy fill benchmark:
  - `path_tessellation_miss_heavy/fill_miss_512_paths_cache_16`: ~2.92 ms

### Style-engine path ops baseline

- `path_boolean_ops/union/32`: ~7.46 us
- `path_boolean_ops/intersect/32`: ~6.81 us
- `path_boolean_ops/subtract/32`: ~9.09 us
- `path_boolean_ops/union/128`: ~24.43 us
- `path_boolean_ops/intersect/128`: ~28.83 us
- `path_boolean_ops/subtract/128`: ~30.51 us
- `path_boolean_ops/union/512`: ~91.80 us
- `path_boolean_ops/intersect/512`: ~79.55 us
- `path_boolean_ops/subtract/512`: ~101.14 us
- `path_hit_test/contains_point_inside`: ~2.49 us
- `path_hit_test/contains_point_hole`: ~2.33 us
- `path_hit_test/contains_point_outside`: ~2.51 us
- `svg_parse_complex_path_d`: ~1.15 us

Interpretation:
- The plan's `<100 ns` cache-hit target is now met on both keyed and non-keyed hot paths.
- Miss-heavy/cold-path still dominates total cost and remains the primary optimization frontier.
