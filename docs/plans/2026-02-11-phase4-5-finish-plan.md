# Phase 4 + 5 Finish-Line Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete Phase 4 and Phase 5 to production-ready behavior (not scaffolding), including real multipass effects, browser runtime loop/input, fallback behavior, visual correctness, and benchmark/CI gates.

**Architecture:** Keep the existing `render-engine`/`plat-core` structure and incrementally replace placeholder paths with deterministic effect-plan execution and web runtime behavior. Implement in strict TDD order so every new pass, planner rule, and browser behavior is backed by tests and measurable gates. Preserve native behavior while adding wasm/web paths behind explicit feature and target cfgs.

**Tech Stack:** Rust 2024, wgpu 28, WGSL, bevy_ecs resource integration, wasm32-unknown-unknown, web-sys/wasm-bindgen, Trunk, Criterion.

**Required skills:** `@test-driven-development` `@verification-before-completion` `@rust-router` `@rust-perf-engineer`

---

### Task 1: Replace Effect Classification with Deterministic Effect Plan

**Files:**
- Modify: `crates/render-engine/src/backend/wgpu/effects.rs`
- Modify: `crates/render-engine/src/backend/wgpu/mod.rs`
- Test: `crates/render-engine/tests/phase4_effect_plan_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_effect_plan_orders_layer_blur_pipeline_steps() {
    let scene = make_scene_with_layer_blur();
    let plan = plan_effect_passes(&scene, 1280, 720);
    let kinds: Vec<_> = plan.iter().map(|p| p.kind).collect();
    assert_eq!(
        kinds,
        vec![
            EffectPassKind::OffscreenLayer,
            EffectPassKind::BlurHorizontal,
            EffectPassKind::BlurVertical,
            EffectPassKind::BlendComposite
        ]
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p render-engine --test phase4_effect_plan_tests -- --nocapture`  
Expected: FAIL with missing `plan_effect_passes` and/or wrong ordering.

**Step 3: Write minimal implementation**

```rust
pub struct EffectPass {
    pub node_id: crate::NodeId,
    pub kind: EffectPassKind,
    pub target: Option<RenderTargetHandle>,
    pub bounds_px: [u32; 4],
    pub blend_mode: style_engine::BlendMode,
}

pub fn plan_effect_passes(scene: &Scene, width: u32, height: u32) -> Vec<EffectPass> {
    // deterministic walk over z-ordered visuals and pass expansion rules
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p render-engine --test phase4_effect_plan_tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/render-engine/src/backend/wgpu/effects.rs crates/render-engine/src/backend/wgpu/mod.rs crates/render-engine/tests/phase4_effect_plan_tests.rs
git commit -m "feat(render-engine): add deterministic phase4 effect pass planner"
```

### Task 2: Complete Render Target Pool with Budget + Real Reclamation

**Files:**
- Create: `crates/render-engine/src/backend/wgpu/render_target_pool.rs`
- Modify: `crates/render-engine/src/backend/wgpu/context.rs`
- Modify: `crates/render-engine/src/backend/wgpu/mod.rs`
- Test: `crates/render-engine/tests/phase4_render_target_pool_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_pool_enforces_soft_budget_and_evicts_oldest_free_targets() {
    let mut pool = RenderTargetPool::new(1024);
    // acquire/release multiple targets over budget
    // assert free bytes <= budget after end_frame()
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p render-engine --test phase4_render_target_pool_tests -- --nocapture`  
Expected: FAIL with missing budget eviction behavior.

**Step 3: Write minimal implementation**

```rust
pub fn end_frame(&mut self, ctx: &mut WgpuContext) {
    // return in-use to free
    // evict LRU free targets until <= soft budget
    // call ctx.remove_render_target(handle) for evicted handles
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p render-engine --test phase4_render_target_pool_tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/render-engine/src/backend/wgpu/render_target_pool.rs crates/render-engine/src/backend/wgpu/context.rs crates/render-engine/src/backend/wgpu/mod.rs crates/render-engine/tests/phase4_render_target_pool_tests.rs
git commit -m "feat(render-engine): enforce render target pool memory budget and eviction"
```

### Task 3: Execute Layer Blur Passes (A->B->A) with Radius Tiering

**Files:**
- Modify: `crates/render-engine/src/backend/wgpu/mod.rs`
- Modify: `crates/render-engine/src/backend/wgpu/pipelines/blur_pipeline.rs`
- Test: `crates/render-engine/tests/phase4_blur_execution_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_select_blur_tier_uses_downsample_path_for_large_radius() {
    assert_eq!(select_blur_tier(8.0), BlurTier::FullRes);
    assert_eq!(select_blur_tier(36.0), BlurTier::HalfRes);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p render-engine --test phase4_blur_execution_tests -- --nocapture`  
Expected: FAIL with missing `select_blur_tier`/execution path.

**Step 3: Write minimal implementation**

```rust
enum BlurTier { FullRes, HalfRes }
fn select_blur_tier(radius: f32) -> BlurTier { if radius > 24.0 { BlurTier::HalfRes } else { BlurTier::FullRes } }
```

Then wire actual blur pass encoding in `render()` effect execution.

**Step 4: Run test to verify it passes**

Run: `cargo test -p render-engine --test phase4_blur_execution_tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/render-engine/src/backend/wgpu/mod.rs crates/render-engine/src/backend/wgpu/pipelines/blur_pipeline.rs crates/render-engine/tests/phase4_blur_execution_tests.rs
git commit -m "feat(render-engine): execute layer blur passes with radius tiering"
```

### Task 4: Implement Background Blur Capture + Masked Composite

**Files:**
- Modify: `crates/render-engine/src/backend/wgpu/mod.rs`
- Modify: `crates/render-engine/src/backend/wgpu/effects.rs`
- Test: `crates/render-engine/tests/phase4_background_blur_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_background_blur_inflates_bounds_by_radius_and_clips_to_frame() {
    let b = backdrop_capture_bounds([10, 10, 100, 100], 12.0, [0, 0, 120, 120]);
    assert_eq!(b, [0, 0, 120, 120]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p render-engine --test phase4_background_blur_tests -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

```rust
fn backdrop_capture_bounds(node: [u32;4], radius: f32, frame: [u32;4]) -> [u32;4] {
    // inflate by ceil(radius*2), clamp to frame
}
```

Then wire pass order:
1. capture backdrop region  
2. blur horizontal/vertical  
3. mask by shape  
4. render node on top

**Step 4: Run test to verify it passes**

Run: `cargo test -p render-engine --test phase4_background_blur_tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/render-engine/src/backend/wgpu/mod.rs crates/render-engine/src/backend/wgpu/effects.rs crates/render-engine/tests/phase4_background_blur_tests.rs
git commit -m "feat(render-engine): add background blur capture and masked compositing"
```

### Task 5: Implement Inner Shadow Pass with Shape-Interior Masking

**Files:**
- Modify: `crates/render-engine/src/backend/wgpu/mod.rs`
- Modify: `crates/render-engine/src/backend/shaders/blur.wgsl`
- Test: `crates/render-engine/tests/phase4_inner_shadow_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_inner_shadow_alpha_nonzero_only_inside_mask() {
    assert_eq!(inner_shadow_alpha(0.0, 1.0), 0.0);
    assert!(inner_shadow_alpha(1.0, 0.8) > 0.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p render-engine --test phase4_inner_shadow_tests -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

```rust
fn inner_shadow_alpha(mask: f32, blurred_offset_mask: f32) -> f32 {
    ((blurred_offset_mask - (1.0 - mask)).clamp(0.0, 1.0)) * mask
}
```

Then connect mask/offscreen/subtract/composite pass flow.

**Step 4: Run test to verify it passes**

Run: `cargo test -p render-engine --test phase4_inner_shadow_tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/render-engine/src/backend/wgpu/mod.rs crates/render-engine/src/backend/shaders/blur.wgsl crates/render-engine/tests/phase4_inner_shadow_tests.rs
git commit -m "feat(render-engine): implement inner shadow masked blur pass"
```

### Task 6: Integrate Stencil Push/Pop into Real Render Passes

**Files:**
- Modify: `crates/render-engine/src/backend/wgpu/pipelines/stencil_pipeline.rs`
- Modify: `crates/render-engine/src/backend/wgpu/mod.rs`
- Test: `crates/render-engine/tests/phase4_stencil_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_nested_clip_pass_sequence_is_balanced() {
    let seq = plan_clip_sequence_for_nested_clips();
    assert_eq!(seq, vec![StencilPush, StencilPush, StencilPop, StencilPop]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p render-engine --test phase4_stencil_tests -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

```rust
// apply stencil reference increments/decrements and compare Equal(depth)
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p render-engine --test phase4_stencil_tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/render-engine/src/backend/wgpu/pipelines/stencil_pipeline.rs crates/render-engine/src/backend/wgpu/mod.rs crates/render-engine/tests/phase4_stencil_tests.rs
git commit -m "feat(render-engine): enforce nested stencil push/pop clipping"
```

### Task 7: Expand Blend Pipeline to Full Figma Blend Mode Set

**Files:**
- Modify: `crates/render-engine/src/backend/shaders/blend.wgsl`
- Modify: `crates/render-engine/src/backend/wgpu/pipelines/blend_pipeline.rs`
- Modify: `crates/render-engine/src/backend/wgpu/effects.rs`
- Test: `crates/render-engine/tests/phase4_blend_modes_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_blend_mode_screen_matches_reference() {
    let src = [0.2, 0.5, 0.8];
    let dst = [0.9, 0.4, 0.25];
    assert_eq!(blend_screen(src, dst), [0.92, 0.7, 0.85]);
}
```

Add similar tests for overlay, darken, lighten, difference, exclusion.

**Step 2: Run test to verify it fails**

Run: `cargo test -p render-engine --test phase4_blend_modes_tests -- --nocapture`  
Expected: FAIL due missing modes/reference funcs.

**Step 3: Write minimal implementation**

```rust
pub fn blend_screen(...) -> [f32;3] { ... }
pub fn blend_overlay(...) -> [f32;3] { ... }
// etc
```

Then mirror mode IDs in WGSL shader switch.

**Step 4: Run test to verify it passes**

Run: `cargo test -p render-engine --test phase4_blend_modes_tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/render-engine/src/backend/shaders/blend.wgsl crates/render-engine/src/backend/wgpu/pipelines/blend_pipeline.rs crates/render-engine/src/backend/wgpu/effects.rs crates/render-engine/tests/phase4_blend_modes_tests.rs
git commit -m "feat(render-engine): add complete blend mode compositing support"
```

### Task 8: Add Phase 4 Visual Tests + Benchmarks + Exit Gates

**Files:**
- Create: `examples/visual_test_phase4_blur.rs`
- Create: `examples/visual_test_phase4_blend.rs`
- Create: `examples/visual_test_phase4_clipping.rs`
- Create: `crates/render-engine/benches/phase4_effects.rs`
- Modify: `crates/render-engine/Cargo.toml`

**Step 1: Write the failing benchmark/test harness**

Add empty benchmark group names and assertions for p95 thresholds.

**Step 2: Run to verify it fails**

Run: `cargo bench -p render-engine phase4_effects -- --noplot`  
Expected: FAIL or empty benchmark group.

**Step 3: Write minimal implementation**

Implement benches:
- `blur_pass_1080p`
- `background_blur_500_nodes`
- `blend_composite_1000_layers`

Add threshold assertions in benchmark post-check helper.

**Step 4: Run to verify it passes**

Run: `cargo bench -p render-engine phase4_effects -- --noplot`  
Expected: PASS with reported timings.

**Step 5: Commit**

```bash
git add examples/visual_test_phase4_blur.rs examples/visual_test_phase4_blend.rs examples/visual_test_phase4_clipping.rs crates/render-engine/benches/phase4_effects.rs crates/render-engine/Cargo.toml
git commit -m "test(render-engine): add phase4 visual tests and performance gates"
```

### Task 9: Replace Web Stub Loop with requestAnimationFrame Runtime

**Files:**
- Modify: `crates/plat-core/src/platform/web.rs`
- Test: `crates/plat-core/src/platform/web.rs` (`#[cfg(test)]` pure logic tests)

**Step 1: Write the failing test**

```rust
#[test]
fn test_redraw_scheduler_requests_next_frame_once() {
    let mut sched = RedrawScheduler::default();
    assert!(sched.request());
    assert!(!sched.request()); // coalesced until frame consumed
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p plat-core redraw_scheduler -- --nocapture`  
Expected: FAIL with missing scheduler.

**Step 3: Write minimal implementation**

Add scheduler + RAF callback path and consume-on-frame semantics.

**Step 4: Run test to verify it passes**

Run: `cargo test -p plat-core redraw_scheduler -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/plat-core/src/platform/web.rs
git commit -m "feat(plat-core): add requestAnimationFrame-driven web event loop"
```

### Task 10: Web Input Mapping (Pointer/Wheel/Keyboard/Resize)

**Files:**
- Modify: `crates/plat-core/src/platform/web.rs`
- Test: `crates/plat-core/tests/web_event_mapping_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_wheel_delta_mode_line_is_normalized_to_pixels() {
    let e = WheelInput { delta_y: 3.0, mode: WheelMode::Line };
    assert_eq!(normalize_wheel(e), 48.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p plat-core --test web_event_mapping_tests -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Add pure mapping helpers:
- pointer down/move/up  
- wheel delta normalization  
- key down/up mapping  
- resize + DPR event emission

Then attach DOM listeners using those helpers.

**Step 4: Run test to verify it passes**

Run: `cargo test -p plat-core --test web_event_mapping_tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/plat-core/src/platform/web.rs crates/plat-core/tests/web_event_mapping_tests.rs
git commit -m "feat(plat-core): map browser input and resize events to platform events"
```

### Task 11: Web Font/Asset Loader and Cache

**Files:**
- Modify: `crates/text-engine/src/lib.rs`
- Create: `crates/text-engine/src/web_loader.rs`
- Test: `crates/text-engine/tests/web_font_loader_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_font_cache_returns_cached_bytes_for_same_url() {
    let mut c = FontCache::default();
    c.insert("https://x/font.ttf", vec![1,2,3]);
    assert_eq!(c.get("https://x/font.ttf").unwrap(), &[1,2,3]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p text-engine --test web_font_loader_tests -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Create:

```rust
pub enum FontSource { Bytes(Vec<u8>), Url(String) }
```

Add async web fetch loader and in-memory cache keyed by URL/version.

**Step 4: Run test to verify it passes**

Run: `cargo test -p text-engine --test web_font_loader_tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/text-engine/src/lib.rs crates/text-engine/src/web_loader.rs crates/text-engine/tests/web_font_loader_tests.rs
git commit -m "feat(text-engine): add web font fetch and cache support"
```

### Task 12: WebGPU Primary + WebGL2 Fallback Probe

**Files:**
- Modify: `crates/render-engine/src/backend/wgpu/context.rs`
- Test: `crates/render-engine/tests/web_backend_probe_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_backend_probe_prefers_webgpu_then_falls_back_to_gl() {
    let caps = ProbeCaps { webgpu_available: false, webgl2_available: true };
    assert_eq!(select_web_backend(caps), WebBackend::WebGl2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p render-engine --test web_backend_probe_tests -- --nocapture`  
Expected: FAIL.

**Step 3: Write minimal implementation**

```rust
enum WebBackend { WebGpu, WebGl2 }
fn select_web_backend(caps: ProbeCaps) -> WebBackend { ... }
```

Wire the selected backend into context creation for wasm.

**Step 4: Run test to verify it passes**

Run: `cargo test -p render-engine --test web_backend_probe_tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/render-engine/src/backend/wgpu/context.rs crates/render-engine/tests/web_backend_probe_tests.rs
git commit -m "feat(render-engine): add webgpu/webgl2 runtime backend probe"
```

### Task 13: Make `widget_gallery_web` Render Real Content + Browser Smoke Check

**Files:**
- Modify: `examples/widget_gallery_web.rs`
- Modify: `index.html`
- Modify: `Trunk.toml`
- Create: `scripts/smoke/web_gallery_smoke.ps1`

**Step 1: Write the failing smoke script**

`scripts/smoke/web_gallery_smoke.ps1` should fail if build artifacts are missing or page title/canvas is absent.

**Step 2: Run to verify it fails**

Run: `powershell -ExecutionPolicy Bypass -File scripts/smoke/web_gallery_smoke.ps1`  
Expected: FAIL before real rendering loop is connected.

**Step 3: Write minimal implementation**

Use real app startup path in `widget_gallery_web`:
- create window/canvas through `plat-core` web backend  
- initialize render backend  
- schedule redraw on RAF  
- submit at least one visible primitive/text frame

**Step 4: Run to verify it passes**

Run:
- `trunk build --example widget_gallery_web --features web`
- `powershell -ExecutionPolicy Bypass -File scripts/smoke/web_gallery_smoke.ps1`

Expected: PASS.

**Step 5: Commit**

```bash
git add examples/widget_gallery_web.rs index.html Trunk.toml scripts/smoke/web_gallery_smoke.ps1
git commit -m "feat(examples): render live widget gallery in browser with smoke check"
```

### Task 14: Final Verification Matrix + CI Gate Update

**Files:**
- Modify: `.github/workflows/ci.yml` (or existing CI workflow path)
- Modify: `docs/plans/2026-02-11-phase4-5-implementation-log.md`

**Step 1: Write failing CI gate locally**

Add required jobs:
- wasm check  
- phase4 tests  
- phase4 benchmark smoke run  
- trunk build smoke

**Step 2: Run local equivalent to verify gaps**

Run:
- `cargo test -p render-engine --test phase4_effects_tests -- --nocapture`
- `cargo check -p plat-core --target wasm32-unknown-unknown --features web`
- `cargo check -p render-engine --target wasm32-unknown-unknown --features web`
- `trunk build --example widget_gallery_web --features web`

Expected: identify any remaining failures.

**Step 3: Write minimal implementation**

Update CI workflow and fix command/env details until all commands run cleanly.

**Step 4: Run full verification**

Run:
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `cargo bench -p render-engine phase4_effects -- --noplot`
- `trunk build --example widget_gallery_web --features web`

Expected: PASS.

**Step 5: Commit**

```bash
git add .github/workflows/ci.yml docs/plans/2026-02-11-phase4-5-implementation-log.md
git commit -m "ci: enforce phase4/5 completion gates for native and web"
```

---

## Completion Gates (must all be true)

1. Phase 4: layer blur, background blur, inner shadow, clip stack, and blend compositing execute real render passes and visually match expected output.
2. Phase 4: benchmark thresholds from plan are met or regressions are documented and accepted.
3. Phase 5: browser loop runs continuously via RAF and maps pointer/wheel/keyboard/resize.
4. Phase 5: `widget_gallery_web` renders visible content, not console-only.
5. wasm build + trunk build + native checks pass in CI.
6. Implementation log updated with command evidence for every gate.

