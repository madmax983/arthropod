# Phase 4 + 5 Implementation Log

Date: 2026-02-11
Scope: Implement Phase 4 (multi-pass effects) and Phase 5 (WASM/web target) from `docs/plans/2026-02-08-figma-rendering-pipeline-design.md`.

## Work Items

- [x] P4.1 Add render-target infrastructure and pooling with tests
- [x] P4.2 Add blur, blend, stencil, and effect-planning modules with tests
- [x] P4.3 Integrate effect planning into `WgpuBackend` render flow
- [x] P4.4 Add Phase 4 shader assets and wire pipeline registry
- [x] P5.1 Add web platform backend in `plat-core` with wasm cfg routing
- [x] P5.2 Add workspace/crate feature gates for `native` and `web`
- [x] P5.3 Add web entrypoint example for widget gallery
- [x] P5.4 Run verification (tests/checks, including wasm target checks)

## Execution Notes

### 2026-02-11

- Started implementation pass.
- Audited existing code:
  - `render-engine` already has Phase 1-3 primitives/path pipeline.
  - `style-engine` already exposes effects and blend modes.
  - `plat-core` currently supports Windows/macOS + stub fallback; no wasm web backend yet.
- Decided on incremental TDD approach:
  - Write failing tests per subsystem.
  - Implement minimum viable production code to satisfy tests.
  - Integrate modules and then run full verification commands.
- Added failing Phase 4 test suite: `crates/render-engine/tests/phase4_effects_tests.rs`
  - Covers: render-target pool reuse, blur kernel normalization, blend multiply reference math, effect pass classification, clip/stencil classification.
- RED verification run:
  - Command: `cargo test -p render-engine phase4_effects_tests -- --nocapture`
  - Result: FAIL (expected)
  - Failure: unresolved import `render_engine::backend::wgpu::effects` (module not implemented yet).
- Implemented Phase 4 effect infrastructure module:
  - Added `crates/render-engine/src/backend/wgpu/effects.rs`
  - Added exports in `crates/render-engine/src/backend/wgpu/mod.rs`
  - Implemented:
    - `EffectPassKind`
    - `RenderTargetKey`
    - `RenderTargetPool` (acquire/release/end_frame)
    - `gaussian_kernel_1d`
    - `blend_multiply`
    - `classify_effect_passes`
- GREEN verification run:
  - Command: `cargo test -p render-engine --test phase4_effects_tests -- --nocapture`
  - Result: PASS (5/5)
- Added Phase 4 pipeline modules and assets:
  - `crates/render-engine/src/backend/wgpu/pipelines/blur_pipeline.rs`
  - `crates/render-engine/src/backend/wgpu/pipelines/blend_pipeline.rs`
  - `crates/render-engine/src/backend/wgpu/pipelines/stencil_pipeline.rs`
  - `crates/render-engine/src/backend/shaders/blur.wgsl`
  - `crates/render-engine/src/backend/shaders/blend.wgsl`
  - Wired into `crates/render-engine/src/backend/wgpu/pipelines/mod.rs`
- Added Phase 4 context/backend integration:
  - Offscreen target allocation API in `crates/render-engine/src/backend/wgpu/context.rs`
  - Effect planning + pool reservation path in `crates/render-engine/src/backend/wgpu/mod.rs`
  - Added unit coverage for scene effect classification.
- Verification run:
  - Command: `cargo test -p render-engine --lib -- --nocapture`
  - Result: PASS (88/88)
- Started Phase 5 RED step:
  - Command: `cargo check -p plat-core --target wasm32-unknown-unknown --features web`
  - Result: FAIL (expected)
  - Failure: package `plat-core` does not contain feature `web`.
- Implemented Phase 5 web feature gating and wasm dependencies:
  - Updated workspace/root features (`native`, `web`) in `Cargo.toml`.
  - Added `web` feature in `crates/plat-core/Cargo.toml`.
  - Added `web` feature in `crates/render-engine/Cargo.toml`.
  - Added wasm-specific dependencies for `plat-core` (`wasm-bindgen`, `web-sys`) and `render-engine` (`getrandom` wasm backend).
- Added wasm target config:
  - Created `.cargo/config.toml` with wasm rustflags for `getrandom_backend="wasm_js"`.
- Implemented web platform backend:
  - Added `crates/plat-core/src/platform/web.rs`.
  - Wired cfg routing in `crates/plat-core/src/platform/mod.rs`.
- Added web gallery entrypoint:
  - Created `examples/widget_gallery_web.rs` with `#[wasm_bindgen(start)]` startup path.
- Continued wasm verification:
  - Command: `cargo check -p plat-core --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `cargo check -p render-engine --target wasm32-unknown-unknown --features web`
  - Result: PASS.
- Encountered root wasm example check failures:
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: FAIL.
  - Cause: non-wasm-safe dependencies still compiled for wasm target (`arthropod`, `renderdoc` path, terminal-only dev deps).
- Fixed root wasm dependency gating:
  - Updated root `Cargo.toml`:
    - moved `arthropod` to non-wasm target dependency block.
    - kept `renderdoc` only on non-wasm target dependency block.
    - moved `ratatui` and `crossterm` to non-wasm target dev-dependency block.
    - added wasm-only root deps needed by example (`wasm-bindgen`, `web-sys` with `console`).
- Encountered wasm example binary entrypoint issue:
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: FAIL.
  - Cause: wasm example binary still requires a `main` symbol.
- Fixed wasm entrypoint:
  - Added `#[cfg(target_arch = "wasm32")] fn main() {}` shim to `examples/widget_gallery_web.rs`.
- Final verification pass:
  - Command: `cargo test -p render-engine --test phase4_effects_tests -- --nocapture`
  - Result: PASS (5/5).
  - Command: `cargo test -p render-engine --lib -- --nocapture`
  - Result: PASS (88/88).
  - Command: `cargo check -p plat-core --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `cargo check -p render-engine --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `cargo check --example widget_gallery`
  - Result: PASS (native sanity check after target-specific gating).
- Formatting and post-format recheck:
  - Command: `cargo fmt --all`
  - Result: PASS.
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
- Final diff cleanup and safety revalidation:
  - Reverted unrelated formatting-only files touched by `cargo fmt --all` so the change set stays scoped to Phase 4/5 work.
  - Command: `cargo test -p render-engine --test phase4_effects_tests -- --nocapture`
  - Result: PASS (5/5).
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
- Browser runtime follow-up (`.wasm` direct execution on Windows):
  - Reproduced the user-facing issue context: running `target\\wasm32-unknown-unknown\\debug\\examples\\widget_gallery_web.wasm` directly is not a valid Win32 process.
  - Added Trunk browser harness files:
    - `index.html` (root web target with `#arthropod-canvas` + Trunk rust asset)
    - `Trunk.toml` (target + dist defaults for web build)
  - Updated `examples/widget_gallery_web.rs` docs with browser run command.
  - Command: `trunk build --example widget_gallery_web --features web`
  - Result: PASS.
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.

### 2026-02-11 (finish-plan execution, batch 1: tasks 1-3)

- Task 1 RED:
  - Added `crates/render-engine/tests/phase4_effect_plan_tests.rs`.
  - Command: `cargo test -p render-engine --test phase4_effect_plan_tests -- --nocapture`
  - Result: FAIL (expected): missing `EffectPlanNode` and `plan_effect_passes`.
- Task 1 implementation:
  - Added deterministic planning API in `crates/render-engine/src/backend/wgpu/effects.rs`:
    - `EffectPass`
    - `EffectPlanNode` + `EffectPlanNode::from_scene`
    - `plan_effect_passes(...)`
    - frame-clamped bounds helper
  - Ensured offscreen effect chains append `BlendComposite` for layer/background/inner-shadow compositing.
- Task 1 GREEN:
  - Command: `cargo test -p render-engine --test phase4_effect_plan_tests -- --nocapture`
  - Result: PASS (1/1).

- Task 2 RED:
  - Added `crates/render-engine/tests/phase4_render_target_pool_tests.rs`.
  - Command: `cargo test -p render-engine --test phase4_render_target_pool_tests -- --nocapture`
  - Result: FAIL (expected): missing `end_frame_with` and `free_bytes`.
- Task 2 implementation:
  - Created `crates/render-engine/src/backend/wgpu/render_target_pool.rs`.
  - Moved/expanded pool internals:
    - explicit `free_order` tracking for FIFO eviction
    - `free_bytes()`
    - `end_frame_with(...)` with budget-based eviction callback
  - Re-exported pool types from `effects.rs` for compatibility.
  - Wired module in `crates/render-engine/src/backend/wgpu/mod.rs`.
  - Integrated real reclamation callback:
    - `WgpuBackend::render` now calls `end_frame_with` and evicts context targets via `context.remove_render_target(...)`.
- Task 2 GREEN:
  - Command: `cargo test -p render-engine --test phase4_render_target_pool_tests -- --nocapture`
  - Result: PASS (1/1).

- Task 3 RED:
  - Added `crates/render-engine/tests/phase4_blur_execution_tests.rs`.
  - Command: `cargo test -p render-engine --test phase4_blur_execution_tests -- --nocapture`
  - Result: FAIL (expected): missing `BlurTier` and `select_blur_tier`.
- Task 3 implementation:
  - Added `BlurTier` + `select_blur_tier(radius)` in `crates/render-engine/src/backend/wgpu/pipelines/blur_pipeline.rs`.
  - Added render-path hook in `crates/render-engine/src/backend/wgpu/mod.rs`:
    - computes `max_scene_blur_radius(scene)`
    - selects tier via `select_blur_tier(...)` during effect planning.
- Task 3 GREEN:
  - Command: `cargo test -p render-engine --test phase4_blur_execution_tests -- --nocapture`
  - Result: PASS (1/1).

- Batch sanity re-run:
  - Command: `cargo test -p render-engine --test phase4_effect_plan_tests -- --nocapture`
  - Result: PASS.
  - Command: `cargo test -p render-engine --test phase4_render_target_pool_tests -- --nocapture`
  - Result: PASS.
  - Command: `cargo test -p render-engine --lib -- --nocapture`
  - Result: PASS (90/90).
  - Command: `cargo check -p render-engine --target wasm32-unknown-unknown --features web`
  - Result: PASS.

### 2026-02-11 (finish-plan execution, batch 2: tasks 4-6)

- Task 4 RED:
  - Added `crates/render-engine/tests/phase4_background_blur_tests.rs`.
  - Command: `cargo test -p render-engine --test phase4_background_blur_tests -- --nocapture`
  - Result: FAIL (expected): missing `backdrop_capture_bounds`.
- Task 4 implementation:
  - Added `backdrop_capture_bounds(node_bounds, radius, frame_bounds)` in `crates/render-engine/src/backend/wgpu/effects.rs`.
  - Added internal unit coverage for bounds inflate+clamp behavior.
  - Added render-path hook in `crates/render-engine/src/backend/wgpu/mod.rs`:
    - `collect_background_capture_bounds(scene, frame_width, frame_height)`
    - called during render effect-planning path.
- Task 4 GREEN:
  - Command: `cargo test -p render-engine --test phase4_background_blur_tests -- --nocapture`
  - Result: PASS (1/1).

- Task 5 RED:
  - Added `crates/render-engine/tests/phase4_inner_shadow_tests.rs`.
  - Command: `cargo test -p render-engine --test phase4_inner_shadow_tests -- --nocapture`
  - Result: FAIL (expected): missing `inner_shadow_alpha`.
- Task 5 implementation:
  - Added CPU helper `inner_shadow_alpha(mask, blurred_offset_mask)` in `crates/render-engine/src/backend/wgpu/effects.rs`.
  - Added WGSL helper of the same formula in `crates/render-engine/src/backend/shaders/blur.wgsl`.
  - Added internal unit coverage for mask behavior.
- Task 5 GREEN:
  - Command: `cargo test -p render-engine --test phase4_inner_shadow_tests -- --nocapture`
  - Result: PASS (1/1).

- Task 6 RED:
  - Added `crates/render-engine/tests/phase4_stencil_tests.rs`.
  - Command: `cargo test -p render-engine --test phase4_stencil_tests -- --nocapture`
  - Result: FAIL (expected): missing `plan_clip_sequence_for_nested_clips`.
- Task 6 implementation:
  - Added `plan_clip_sequence_for_nested_clips()` in `crates/render-engine/src/backend/wgpu/pipelines/stencil_pipeline.rs`.
  - Added render hook in `crates/render-engine/src/backend/wgpu/mod.rs`:
    - when stencil effects are present, computes planned clip sequence.
- Task 6 GREEN:
  - Command: `cargo test -p render-engine --test phase4_stencil_tests -- --nocapture`
  - Result: PASS (1/1).

- Batch sanity re-run:
  - Command: `cargo test -p render-engine --test phase4_background_blur_tests -- --nocapture`
  - Result: PASS.
  - Command: `cargo test -p render-engine --test phase4_inner_shadow_tests -- --nocapture`
  - Result: PASS.
  - Command: `cargo test -p render-engine --test phase4_stencil_tests -- --nocapture`
  - Result: PASS.
  - Command: `cargo test -p render-engine --lib -- --nocapture`
  - Result: PASS (92/92).
  - Command: `cargo check -p render-engine --target wasm32-unknown-unknown --features web`
  - Result: PASS.

### 2026-02-11 (finish-plan execution, batch 3: tasks 7-9)

- Task 7 RED:
  - Added `crates/render-engine/tests/phase4_blend_modes_tests.rs`.
  - Command: `cargo test -p render-engine --test phase4_blend_modes_tests -- --nocapture`
  - Result: FAIL (expected): missing blend helpers (`blend_screen`, `blend_overlay`, `blend_darken`, `blend_lighten`, `blend_difference`, `blend_exclusion`).
- Task 7 implementation:
  - Added CPU blend references in `crates/render-engine/src/backend/wgpu/effects.rs`:
    - `blend_screen`
    - `blend_overlay`
    - `blend_darken`
    - `blend_lighten`
    - `blend_difference`
    - `blend_exclusion`
  - Expanded shader mode coverage in `crates/render-engine/src/backend/shaders/blend.wgsl`:
    - explicit branches for darken/lighten/difference/exclusion/color burn/color dodge/linear burn/linear dodge
    - hard-light and soft-light fallbacks
    - explicit handling comments for HSL non-separable modes + passthrough fallback.
  - Added pipeline-side helper + unit test in `crates/render-engine/src/backend/wgpu/pipelines/blend_pipeline.rs`:
    - `has_explicit_shader_branch(mode)`
- Task 7 GREEN:
  - Command: `cargo test -p render-engine --test phase4_blend_modes_tests -- --nocapture`
  - Result: PASS (6/6).

- Task 8 RED:
  - Command: `cargo bench -p render-engine phase4_effects -- --noplot`
  - Result: FAIL (expected in current state): no dedicated benchmark target was wired yet (`--noplot` passed to default lib bench harness).
- Task 8 implementation:
  - Added visual smoke examples:
    - `examples/visual_test_phase4_blur.rs`
    - `examples/visual_test_phase4_blend.rs`
    - `examples/visual_test_phase4_clipping.rs`
  - Added criterion benchmark target:
    - `crates/render-engine/benches/phase4_effects.rs`
    - benchmarks: `blur_pass_1080p`, `background_blur_500_nodes`, `blend_composite_layers/1000`
  - Registered bench in `crates/render-engine/Cargo.toml` (`[[bench]] name = "phase4_effects"`).
  - Registered examples in root `Cargo.toml`:
    - `visual_test_phase4_blur`
    - `visual_test_phase4_blend`
    - `visual_test_phase4_clipping`
- Task 8 GREEN:
  - Command: `cargo bench -p render-engine --bench phase4_effects -- --noplot`
  - Result: PASS.
  - Command: `cargo check --example visual_test_phase4_blur`
  - Result: PASS.
  - Command: `cargo check --example visual_test_phase4_blend`
  - Result: PASS.
  - Command: `cargo check --example visual_test_phase4_clipping`
  - Result: PASS.

- Task 9 RED:
  - Added `crates/plat-core/src/platform/web_runtime.rs` with failing scheduler test references.
  - Command: `cargo test -p plat-core redraw_scheduler -- --nocapture`
  - Result: FAIL (expected): missing `RedrawScheduler`.
- Task 9 implementation:
  - Added `RedrawScheduler` with coalescing behavior in `crates/plat-core/src/platform/web_runtime.rs`.
  - Added scheduler tests:
    - `test_redraw_scheduler_requests_next_frame_once`
    - `test_redraw_scheduler_consume_resets_pending_state`
  - Exposed runtime helper module in `crates/plat-core/src/platform/mod.rs`.
  - Replaced one-shot web `run()` path with RAF loop in `crates/plat-core/src/platform/web.rs`:
    - lifecycle resume dispatch
    - coalesced redraw scheduling via `RedrawScheduler`
    - per-frame `RedrawRequested` event + `on_redraw` callback
    - control-flow exit handling
- Task 9 GREEN:
  - Command: `cargo test -p plat-core redraw_scheduler -- --nocapture`
  - Result: PASS (2 scheduler tests).
  - Command: `cargo check -p plat-core --target wasm32-unknown-unknown --features web`
  - Result: PASS.

- Batch sanity re-run:
  - Command: `cargo test -p render-engine --test phase4_blend_modes_tests -- --nocapture`
  - Result: PASS.
  - Command: `cargo bench -p render-engine --bench phase4_effects -- --noplot`
  - Result: PASS.
  - Command: `cargo test -p plat-core redraw_scheduler -- --nocapture`
  - Result: PASS.
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.

### 2026-02-11 (finish-plan execution, batch 4: task 10)

- Task 10 RED:
  - Added `crates/plat-core/tests/web_event_mapping_tests.rs`.
  - Command: `cargo test -p plat-core --test web_event_mapping_tests -- --nocapture`
  - Result: FAIL (expected): unresolved imports for missing mapping helpers (`map_pointer_*`, `map_key_input`, `normalize_wheel`, `map_resize_events`, `wheel_event_from_input`).
- Task 10 implementation:
  - Expanded pure web mapping helpers in `crates/plat-core/src/platform/web_runtime.rs`:
    - wheel normalization + wheel event mapping
    - pointer down/up/move mapping (with and without modifiers)
    - keyboard code/key mapping (with and without modifiers)
    - resize + DPR mapping to `Resized` and `ScaleFactorChanged`
  - Re-exported web mapping helpers from `crates/plat-core/src/lib.rs` for integration test access.
  - Updated `crates/plat-core/src/platform/web.rs` to attach DOM listeners using shared mapping helpers:
    - pointer: `mousedown`, `mouseup`, `mousemove`, `mouseenter`, `mouseleave`
    - wheel: `wheel` (with prevent_default + normalization)
    - keyboard: `keydown`, `keyup`
    - resize: `resize` (canvas physical size update + resize events)
  - Added required `web-sys` features in `crates/plat-core/Cargo.toml`:
    - `KeyboardEvent`, `MouseEvent`, `WheelEvent`, `Event`, `EventTarget`
- Task 10 GREEN:
  - Command: `cargo test -p plat-core --test web_event_mapping_tests -- --nocapture`
  - Result: PASS (5/5).
  - Command: `cargo check -p plat-core`
  - Result: PASS.
  - Command: `cargo check -p plat-core --target wasm32-unknown-unknown --features web`
  - Result: PASS.

### 2026-02-11 (finish-plan execution, batch 4: task 11)

- Task 11 RED:
  - Added `crates/text-engine/tests/web_font_loader_tests.rs`.
  - Command: `cargo test -p text-engine --test web_font_loader_tests -- --nocapture`
  - Result: FAIL (expected): unresolved imports for missing web font loader API (`FontCache`, `FontSource`, `load_font_source`, `load_font_url`) and missing test runtime crate (`pollster`).
- Task 11 implementation:
  - Added `crates/text-engine/src/web_loader.rs`:
    - `FontSource` (`Bytes`, `Url`)
    - `FontCache` with URL and URL+version key support
    - `versioned_cache_key`
    - `load_font_source` and `load_font_url`
    - wasm fetch path + native unsupported fallback
    - `WebFontLoadError`
  - Exported web loader API from `crates/text-engine/src/lib.rs`.
  - Updated `crates/text-engine/Cargo.toml`:
    - Added `thiserror`
    - Added wasm target deps: `js-sys`, `wasm-bindgen`, `wasm-bindgen-futures`, `web-sys` (`Window`, `Response`)
    - Added dev-dependency `pollster` for async tests.
- Task 11 GREEN:
  - Command: `cargo test -p text-engine --test web_font_loader_tests -- --nocapture`
  - Result: PASS (4/4).
  - Command: `cargo check -p text-engine --target wasm32-unknown-unknown`
  - Result: PASS.

### 2026-02-11 (finish-plan execution, batch 4: task 12)

- Task 12 RED:
  - Added `crates/render-engine/tests/web_backend_probe_tests.rs`.
  - Command: `cargo test -p render-engine --test web_backend_probe_tests -- --nocapture`
  - Result: FAIL (expected): missing backend probe API (`ProbeCaps`, `WebBackend`, `select_web_backend`).
- Task 12 implementation:
  - Added backend probe types/functions in `crates/render-engine/src/backend/wgpu/context.rs`:
    - `WebBackend`
    - `ProbeCaps`
    - `select_web_backend(...)`
    - wasm runtime probe helper for WebGPU/WebGL2 availability
  - Wired selected backend into wasm instance creation:
    - `WebGpu` -> `wgpu::Backends::BROWSER_WEBGPU`
    - `WebGl2` -> `wgpu::Backends::GL`
  - Added backend-specific wasm limits:
    - WebGPU path uses `downlevel_defaults`
    - WebGL2 path uses `downlevel_webgl2_defaults`
  - Updated wasm dependencies in `crates/render-engine/Cargo.toml`:
    - `js-sys`, `wasm-bindgen`, `web-sys` (`Window`, `Document`, `HtmlCanvasElement`, `Navigator`)
- Task 12 GREEN:
  - Command: `cargo test -p render-engine --test web_backend_probe_tests -- --nocapture`
  - Result: PASS (1/1).
  - Command: `cargo check -p render-engine --target wasm32-unknown-unknown --features web`
  - Result: PASS.

### 2026-02-11 (finish-plan execution, batch 5: task 13)

- Task 13 RED:
  - Added smoke script `scripts/smoke/web_gallery_smoke.ps1`.
  - Command: `powershell -ExecutionPolicy Bypass -File scripts/smoke/web_gallery_smoke.ps1`
  - Result: FAIL (expected): built HTML title did not match live gallery requirement.
- Task 13 implementation:
  - Implemented real browser-rendered app in `examples/widget_gallery_web.rs`:
    - `plat_core::run::<WebGalleryApp>()` startup path
    - creates `Window` via `EventLoop`
    - initializes `WgpuBackend`
    - builds a visible scene (panel, title/subtitle text, color swatches)
    - handles resize/scale events and redraw renders
  - Updated `plat-core` web window handle support in `crates/plat-core/src/platform/web.rs`:
    - `HasWindowHandle` now returns `WebCanvasWindowHandle`
    - `HasDisplayHandle` now returns `DisplayHandle::web()`
  - Relaxed backend constructor bounds for web compatibility:
    - `crates/render-engine/src/backend/wgpu/context.rs`
    - `crates/render-engine/src/backend/wgpu/mod.rs`
    - removed unconditional `Sync` bound on window handle type
  - Updated browser shell assets:
    - `index.html` title now `Arthropod Widget Gallery Live`
    - `Trunk.toml` set `filehash = false` for stable smoke artifact checks
- Task 13 GREEN:
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `trunk build --example widget_gallery_web --features web`
  - Result: PASS.
  - Command: `powershell -ExecutionPolicy Bypass -File scripts/smoke/web_gallery_smoke.ps1`
  - Result: PASS.

### 2026-02-11 (finish-plan execution, batch 5: task 14)

- Task 14 implementation:
  - Updated `.github/workflows/ci.yml` with `phase45-gates` job:
    - wasm target checks for `plat-core`, `render-engine`, and `widget_gallery_web`
    - explicit Phase 4 test suite execution
    - Phase 4 benchmark smoke (`phase4_effects`)
    - Trunk build smoke for web gallery output
  - Addressed strict clippy gate blockers discovered during full verification:
    - `crates/style-engine/src/path.rs`
      - collapsed nested conditionals
      - replaced needless range loop with iterator+enumerate
      - explicitly allowed `too_many_arguments` for arc polyline helper
    - `crates/render-engine/src/backend/wgpu/effects.rs`
      - replaced `% 2 == 0` with `.is_multiple_of(2)`
    - `crates/render-engine/src/backend/wgpu/pipelines/path_pipeline.rs`
      - collapsed nested conditional in path interning lookup
    - `crates/render-engine/src/backend/wgpu/pipelines/primitive_pipeline.rs`
      - collapsed nested conditional in drop-shadow emission
      - simplified optional pipeline access pattern
    - `crates/render-engine/src/backend/wgpu/render_target_pool.rs`
      - collapsed nested conditional in target reuse path
    - `crates/render-engine/src/backend/wgpu/mod.rs`
      - collapsed nested conditionals in path interning and stroke routing
      - collapsed nested mesh-result/emptiness checks
- Task 14 local gate check (step 2):
  - Command: `cargo test -p render-engine --test phase4_effects_tests -- --nocapture`
  - Result: PASS (5/5).
  - Command: `cargo check -p plat-core --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `cargo check -p render-engine --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `trunk build --example widget_gallery_web --features web`
  - Result: PASS.
- Task 14 full verification (step 4):
  - Command: `cargo fmt --all --check`
  - Result: FAIL initially (format drift) then PASS after `cargo fmt --all`.
  - Command: `cargo clippy --all-targets --all-features -- -D warnings`
  - Result: FAIL initially (strict lint violations) then PASS after fixes.
  - Command: `cargo test --all`
  - Result: PASS.
  - Command: `cargo bench -p render-engine phase4_effects -- --noplot`
  - Result: FAIL (lib bench harness receives unsupported `--noplot`).
  - Command: `cargo bench -p render-engine --bench phase4_effects -- --noplot`
  - Result: PASS.
  - Command: `trunk build --example widget_gallery_web --features web`
  - Result: PASS.
  - Command: `powershell -ExecutionPolicy Bypass -File scripts/smoke/web_gallery_smoke.ps1`
  - Result: PASS.

### 2026-02-11 (post-plan follow-up: wasm panic diagnostics hardening)

- Issue:
  - Browser reported `RuntimeError: unreachable` with stack ending in `Result::expect` during `widget_gallery_web` startup.
- Fix:
  - Updated `examples/widget_gallery_web.rs`:
    - replaced direct `expect(...)` startup path with explicit error logging before panic for:
      - window creation
      - WGPU backend initialization
    - enabled `console_error_panic_hook::set_once()` in wasm start function.
    - added console logging for `plat_core::run` errors.
  - Updated root `Cargo.toml` wasm deps:
    - added `console_error_panic_hook = "0.1"`.
- Verification:
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `trunk build --example widget_gallery_web --features web`
  - Result: PASS.

### 2026-02-11 (post-plan follow-up: wasm no-thread condvar panic fix)

- Issue:
  - Browser panic:
    - `condvar wait not supported`
    - `cannot recursively acquire mutex`
  - Root cause: wasm path still used `pollster::block_on` during backend init (`request_adapter`/`request_device`), which relies on thread/condvar primitives not supported in wasm no-threads runtime.
- Fix:
  - `crates/render-engine/src/backend/wgpu/context.rs`
    - introduced shared async initializer (`new_impl`)
    - kept native `new(...)` wrapper (driven by `pollster` on native)
    - added wasm `new_async(...)` entrypoint
    - removed direct `pollster::block_on` usage from core initialization flow
  - `crates/render-engine/src/backend/wgpu/mod.rs`
    - split backend construction into:
      - native `new(...)`
      - wasm `new_async(...)`
      - shared `from_context(...)` pipeline setup
  - `examples/widget_gallery_web.rs`
    - switched to non-blocking async backend initialization via `wasm_bindgen_futures::spawn_local`
    - app now waits until backend init completes, then renders
    - no startup `expect` panic path for backend init
  - `Cargo.toml`
    - added wasm dependency: `wasm-bindgen-futures`
- Verification:
  - Command: `cargo check -p render-engine --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `cargo check --example single_rect_test`
  - Result: PASS.
  - Command: `trunk build --example widget_gallery_web --features web`
  - Result: PASS.

### 2026-02-11 (post-plan follow-up: web event queue re-entrancy borrow fix)

- Issue:
  - Browser panic:
    - `RefCell already borrowed` in `crates/plat-core/src/platform/web.rs`
  - Root cause: event dispatch could re-enter queue drain paths during callbacks and hit nested mutable `RefCell` borrows in the pending event queue path.
- Fix:
  - Updated `crates/plat-core/src/platform/web.rs`:
    - added `is_draining_events` guard (`Rc<Cell<bool>>`) to prevent nested drain re-entry
    - changed queue push/pop operations to scoped borrows so mutable queue borrows are released before callback dispatch
    - threaded drain guard through all event listener dispatch paths and RAF drain points
    - preserved deferred event behavior when `app.try_borrow_mut()` is unavailable by pushing event back to queue front
- Verification:
  - Command: `cargo check -p plat-core --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `trunk build --example widget_gallery_web --features web`
  - Result: PASS.
  - Command: `powershell -ExecutionPolicy Bypass -File scripts/smoke/web_gallery_smoke.ps1`
  - Result: PASS.
  - Command: `cargo fmt --all --check`
  - Result: FAIL initially (format drift), then PASS after `cargo fmt --all`.
  - Command: `cargo test -p plat-core --test web_event_mapping_tests -- --nocapture`
  - Result: PASS (5/5).

### 2026-02-11 (post-plan follow-up: wasm font/shader/pipeline validation fixes)

- Issues:
  - Browser panic:
    - `cosmic-text ... no default font found` while shaping text in wasm.
  - WebGPU shader validation:
    - integral vertex output `flags: u32` missing `@interpolate(flat)` in primitive shader.
  - WebGPU pipeline validation:
    - blur uniform struct consumed 128 bytes in WGSL but bind-group `minBindingSize` was 124.
- Fix:
  - Updated `crates/text-engine/src/lib.rs`:
    - added `font_system_has_faces(...)` guard
    - `shape_text(...)` and `shape_text_parallel(...)` now return empty shaped output when no fonts are available (prevents wasm panic)
    - added regression test `test_shape_text_without_available_fonts_returns_empty`
  - Updated `crates/render-engine/src/backend/shaders/primitive.wgsl`:
    - annotated `flags` varying with `@interpolate(flat)`.
  - Updated `crates/render-engine/src/backend/wgpu/pipelines/blur_pipeline.rs`:
    - added `_tail_pad` so `BlurParams` ABI matches WGSL uniform size (128 bytes)
    - added regression test `test_blur_params_uniform_size_matches_wgsl`.
- Verification:
  - Command: `cargo test -p text-engine --lib -- --nocapture`
  - Result: PASS (7/7).
  - Command: `cargo test -p render-engine blur_params_uniform_size_matches_wgsl -- --nocapture`
  - Result: PASS.
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `trunk build --example widget_gallery_web --features web`
  - Result: PASS.
  - Command: `powershell -ExecutionPolicy Bypass -File scripts/smoke/web_gallery_smoke.ps1`
  - Result: PASS.
  - Command: `cargo fmt --all --check`
  - Result: PASS.

### 2026-02-11 (post-plan follow-up: primitive WGSL non-uniform texture sampling fix)

- Issue:
  - WebGPU shader validation failed:
    - `'textureSample' must only be called from uniform control flow`
    - triggered by glyph-texture sample in branch guarded by per-fragment `flags`.
  - Downstream warnings (`Invalid ShaderModule`, `Invalid RenderPipeline`, invalid command buffer submit) were secondary failures caused by primitive shader compilation failure.
- Fix:
  - Updated `crates/render-engine/src/backend/shaders/primitive.wgsl`:
    - replaced gradient atlas sample with `textureSampleLevel(..., 0.0)`
    - replaced glyph atlas sample with `textureSampleLevel(..., 0.0)`
  - This removes implicit-derivative requirements that enforce uniform control flow for `textureSample`.
- Verification:
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `trunk build --example widget_gallery_web --features web`
  - Result: PASS.
  - Command: `powershell -ExecutionPolicy Bypass -File scripts/smoke/web_gallery_smoke.ps1`
  - Result: PASS.

### 2026-02-11 (post-plan follow-up: wasm text rendering via bundled runtime font)

- Goal:
  - Restore visible text in `widget_gallery_web` on wasm after no-system-font safety guard.
- RED:
  - Added tests in `crates/text-engine/src/lib.rs` expecting runtime font registration to unblock shaping:
    - `test_register_font_bytes_unblocks_shaping_on_empty_db`
    - `test_shape_text_parallel_uses_runtime_registered_font`
  - Command: `cargo test -p text-engine --lib -- --nocapture`
  - Result: FAIL (expected): missing `TextEngine::register_font_bytes`.
- Implementation:
  - Bundled a local fallback font from already-vendored dependency assets:
    - added `assets/fonts/Inter-Regular.ttf`
    - added `assets/fonts/Inter-LICENSE`
  - Updated `crates/text-engine/src/lib.rs`:
    - added global runtime font registry (`OnceLock<Mutex<Vec<Vec<u8>>>>`)
    - added font propagation helpers:
      - `register_global_font_bytes`
      - `apply_global_fonts`
    - thread-local shaping pool now tracks applied global font count
    - `TextEngine` now tracks `applied_global_fonts`
    - added public `TextEngine::register_font_bytes(bytes) -> usize`
    - `shape_text` and `shape_text_parallel` now sync global runtime fonts before shaping
  - Updated `crates/render-engine/src/backend/text/text_renderer.rs`:
    - added `TextRenderer::register_font_bytes(bytes) -> usize`
  - Updated `crates/render-engine/src/backend/wgpu/mod.rs`:
    - added `WgpuBackend::register_font_bytes(bytes) -> usize` (in inherent impl)
  - Updated `examples/widget_gallery_web.rs`:
    - embedded bundled font bytes with `include_bytes!("../assets/fonts/Inter-Regular.ttf")`
    - registers font immediately after backend init
    - logs loaded face count.
- GREEN + verification:
  - Command: `cargo test -p text-engine --lib -- --nocapture`
  - Result: PASS (9/9).
  - Command: `cargo check -p render-engine`
  - Result: PASS.
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `trunk build --example widget_gallery_web --features web`
  - Result: PASS.
  - Command: `powershell -ExecutionPolicy Bypass -File scripts/smoke/web_gallery_smoke.ps1`
  - Result: PASS.
  - Command: `cargo fmt --all --check`
  - Result: FAIL initially (format drift), then PASS after `cargo fmt --all`.

### 2026-02-11 (post-plan follow-up: Playwright-style visual regression harness for Phase 4)

- Goal:
  - Add real screenshot-based visual regression tests for Phase 4 blur/blend/clipping.
- Implementation:
  - Added wasm visual fixture app:
    - `examples/phase4_visual_web.rs`
    - renders deterministic scenes for `blur`, `blend`, and `clipping` selected by query param (`?case=`).
    - emits readiness markers in DOM:
      - `body[data-arthropod-ready="1"]`
      - `body[data-arthropod-case="<case>"]`
  - Added root example registration and wasm web-sys features in `Cargo.toml`:
    - `[[example]] phase4_visual_web`
    - wasm `web-sys` features expanded for location/document access.
  - Added Playwright harness files:
    - `package.json` (scripts `visual:test` / `visual:update`)
    - `playwright.config.mjs` (webServer uses `trunk serve --example phase4_visual_web --features web`)
    - `tests/visual/phase4-visual.spec.mjs` (canvas snapshots for blur/blend/clipping)
  - Added docs:
    - `docs/testing/phase4-visual-regression.md`
  - Updated `.gitignore` for Node/Playwright outputs.
- Rust-side verification:
  - Command: `cargo check --example phase4_visual_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `cargo check --example phase4_visual_web`
  - Result: PASS.
  - Command: `trunk build --example phase4_visual_web --features web`
  - Result: PASS.
  - Command: `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web`
  - Result: PASS.
  - Command: `powershell -ExecutionPolicy Bypass -File scripts/smoke/web_gallery_smoke.ps1`
  - Result: PASS.
  - Command: `cargo fmt --all --check`
  - Result: PASS.
- Playwright execution status in this environment:
  - Command: `npm install`
  - Result: FAIL (`EACCES` fetching `@playwright/test` from npm registry in sandboxed environment).
  - Command: `npm run visual:test`
  - Result: FAIL (`playwright` command unavailable because dependencies were not installable in this sandbox).

### 2026-02-11 (desktop pivot: Phase 4 visual examples render native windows, not console-only)

- Goal:
  - Ensure `visual_test_phase4_blur`, `visual_test_phase4_blend`, and `visual_test_phase4_clipping` are real desktop rendering examples using the normal native pipeline.
- Implementation:
  - Added shared scene builders in `examples/phase4_visual_scenes.rs`:
    - `build_phase4_blur_scene()`
    - `build_phase4_blend_scene()`
    - `build_phase4_clipping_scene()`
  - Reworked desktop examples into full `plat_core::Application` implementations:
    - `examples/visual_test_phase4_blur.rs`
    - `examples/visual_test_phase4_blend.rs`
    - `examples/visual_test_phase4_clipping.rs`
  - Each example now:
    - creates a real native window
    - initializes `WgpuBackend`
    - handles resize/close events
    - renders the Phase 4 scene on redraw via `backend.render(&scene)`
- Verification:
  - Command: `cargo check --example visual_test_phase4_blur --example visual_test_phase4_blend --example visual_test_phase4_clipping`
  - Result: PASS.
  - Command: `cargo test -p render-engine --test phase4_effect_plan_tests --test phase4_render_target_pool_tests --test phase4_blur_execution_tests --test phase4_background_blur_tests --test phase4_inner_shadow_tests --test phase4_stencil_tests --test phase4_blend_modes_tests --test phase4_effects_tests -- --nocapture`
  - Result: PASS (all listed Phase 4 effect/planner tests).
  - Command: `cargo fmt --all` then `cargo fmt --all --check`
  - Result: PASS (`examples/phase4_visual_scenes.rs` auto-formatted to satisfy rustfmt).
  - Command: `cargo run --example visual_test_phase4_blur` (20s timeout)
  - Result: STARTED successfully (windowed app banner + backend init log observed; command timed out intentionally because event loop keeps running).
  - Command: `cargo run --example visual_test_phase4_blend` (12s timeout)
  - Result: STARTED successfully (windowed app banner + backend init log observed; command timed out intentionally because event loop keeps running).
  - Command: `cargo run --example visual_test_phase4_clipping` (12s timeout)
  - Result: STARTED successfully (windowed app banner + backend init log observed; command timed out intentionally because event loop keeps running).

### 2026-02-11 (desktop visual regression harness: native screenshot + golden diff)

- Goal:
  - Add deterministic native desktop visual regression tests for Phase 4 blur/blend/clipping (no web dependency).
- TDD checkpoints:
  - RED:
    - Added new `WgpuContext` tests expecting missing helpers:
      - `test_aligned_bytes_per_row_rounds_up_to_256_bytes`
      - `test_unpack_readback_pixels_removes_padding`
      - `test_unpack_readback_pixels_swizzles_bgra_to_rgba`
    - Command: `cargo test -p render-engine context::tests::test_aligned_bytes_per_row_rounds_up_to_256_bytes -- --nocapture`
    - Result: FAIL (expected, missing `aligned_bytes_per_row` / `unpack_readback_pixels`).
  - GREEN:
    - Implemented:
      - `aligned_bytes_per_row(...)`
      - `unpack_readback_pixels(...)` (padding strip + BGRA->RGBA swizzle)
      - `WgpuContext::with_offscreen_render_pass(...)` for native RGBA capture.
    - Added `WgpuBackend` capture path:
      - `render_scene_to_rgba(...)`
      - extracted shared frame prep into `prepare_phase4_effect_state(...)` and `collect_frame_batches(...)`.
  - RED (golden test):
    - Added `tests/phase4_desktop_visual_regression.rs`.
    - Command: `cargo test --test phase4_desktop_visual_regression -- --nocapture`
    - Result: FAIL (expected, missing `tests/visual/golden/phase4/*.png`).
  - GREEN:
    - Added native capture example:
      - `examples/capture_phase4_visuals.rs` (writes blur/blend/clipping PNGs)
    - Generated and committed goldens:
      - `tests/visual/golden/phase4/blur.png`
      - `tests/visual/golden/phase4/blend.png`
      - `tests/visual/golden/phase4/clipping.png`
    - Re-ran regression test: PASS.

- Additional changes:
  - Added tolerant image diff utility:
    - `compare_images_with_tolerance(...)` in `crates/arthropod-test/src/visual_test.rs`
    - with unit tests for tolerance behavior.
  - Added docs:
    - `docs/testing/phase4-desktop-visual-regression.md`
  - Added CI gate:
    - `.github/workflows/ci.yml` job `phase4-desktop-visual-regression` on Windows.
  - Added ignore for generated artifacts:
    - `.gitignore`: `tests/visual/artifacts/`
  - Added example registration:
    - `Cargo.toml`: `[[example]] capture_phase4_visuals`

- Verification evidence:
  - `cargo fmt --all --check` -> PASS
  - `cargo check -p render-engine` -> PASS
  - `cargo check --example capture_phase4_visuals` -> PASS
  - `cargo test -p arthropod-test visual_test -- --nocapture` -> PASS
  - `cargo run --example capture_phase4_visuals` -> PASS (writes all three goldens)
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS

### 2026-02-11 (desktop visual regression CI hardening: failure artifacts + threshold overrides)

- Goal:
  - Make CI fail only on substantial visual changes and attach failed render outputs for fast PR review.
- Changes:
  - `tests/phase4_desktop_visual_regression.rs`
    - added env-configurable thresholds:
      - `ARTHROPOD_VISUAL_CHANNEL_TOLERANCE` (default `2`)
      - `ARTHROPOD_VISUAL_MAX_DIFF_RATIO` (default `0.02`)
  - `.github/workflows/ci.yml`
    - `phase4-desktop-visual-regression` job now runs with:
      - `ARTHROPOD_VISUAL_CHANNEL_TOLERANCE=2`
      - `ARTHROPOD_VISUAL_MAX_DIFF_RATIO=0.04`
    - added `actions/upload-artifact@v4` step on `failure()`:
      - artifact name: `phase4-desktop-visual-artifacts`
      - path: `tests/visual/artifacts/phase4`
  - `docs/testing/phase4-desktop-visual-regression.md`
    - documented env overrides and CI artifact behavior.
- Verification evidence:
  - `cargo fmt --all --check` -> PASS
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS
  - `ARTHROPOD_VISUAL_MAX_DIFF_RATIO=0.04 cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS

### 2026-02-11 (parity gap close: `VisualStyle.opacity` now participates in rendering)

- Goal:
  - Align runtime rendering with model semantics so node alpha is `node.opacity * style.opacity`.
- TDD:
  - RED tests added in `crates/render-engine/src/backend/wgpu/mod.rs`:
    - `test_style_opacity_multiplies_node_opacity_for_primitive_instances`
    - `test_style_opacity_multiplies_node_opacity_for_path_batches`
  - RED verification:
    - `cargo test -p render-engine style_opacity_multiplies_node_opacity -- --nocapture`
    - Result: FAIL (expected; got 0.5 and 0.8 instead of 0.2 and 0.4).
  - GREEN implementation:
    - `create_node_instances(...)` now passes combined opacity.
    - `collect_instances_impl(...)` computes `effective_opacity = node.opacity * style.opacity`.
    - Applies `effective_opacity` to:
      - primitive instance creation
      - path batch opacity
      - text fill opacity path
  - GREEN verification:
    - `cargo test -p render-engine style_opacity_multiplies_node_opacity -- --nocapture` -> PASS
    - `cargo check -p render-engine` -> PASS
    - `cargo fmt --all` applied after rustfmt diffs.

### 2026-02-11 (parity gap close: `clips_content` now clips child primitive bounds in collection path)

- Goal:
  - Apply parent `clips_content` to child primitive bounds during instance collection.
- TDD:
  - RED tests added in `crates/render-engine/src/backend/wgpu/mod.rs`:
    - `test_clips_content_partially_clips_child_primitive_bounds`
    - `test_clips_content_skips_child_fully_outside_clip_bounds`
  - RED verification:
    - `cargo test -p render-engine clips_content_ -- --nocapture`
    - Result: FAIL (expected; child width un-clipped and outside child still emitted).
  - GREEN implementation:
    - Added ancestor clip intersection helpers:
      - `rect_intersection(...)`
      - `ancestor_clip_bounds(...)`
      - `clipped_bounds_for_node(...)`
    - `collect_instances_impl(...)` now:
      - computes per-node clipped bounds from ancestor clip containers
      - skips nodes fully outside clip
      - uses clipped bounds for primitive instance size/position and path batch offsets/sizes
  - GREEN verification:
    - `cargo test -p render-engine clips_content_ -- --nocapture` -> PASS
    - `cargo run --example capture_phase4_visuals` -> PASS (regenerated goldens after clipping change)
    - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS

### 2026-02-11 (model parity: Figma `individualStrokeWeights` alias)

- Goal:
  - Improve import parity for Figma stroke weight naming.
- Implementation:
  - Added serde aliases on `StrokeStyle.side_weights`:
    - `individualStrokeWeights`
    - `individual_weights`
  - Added unit test:
    - `test_deserialize_figma_individual_stroke_weights_alias`
- Verification:
  - `cargo test -p style-engine deserialize_figma_individual_stroke_weights_alias -- --nocapture` -> PASS
  - `cargo fmt --all --check` -> PASS

### 2026-02-11 (desktop multipass blur fix: WGSL uniform layout + visual parity)

- Issue:
  - Desktop/native Phase 4 blur path produced invalid pipeline at runtime and blank/transparent output.
  - Uncaptured wgpu validation error from `Blur Shader`:
    - uniform `array<f32, 25>` in WGSL had invalid stride in uniform address space.
- Root cause:
  - WGSL uniform layout rules require 16-byte-aligned array stride; scalar float arrays are not valid for this block shape.
- Implementation:
  - Updated `crates/render-engine/src/backend/wgpu/pipelines/blur_pipeline.rs`:
    - `BlurParams` now uses packed weight storage: `packed_weights: [[f32; 4]; 7]`.
    - Added explicit padding field `_pad2: [u32; 2]` so Rust ABI offsets match WGSL uniform layout.
    - Updated `from_radius(...)` packing logic.
    - Added/updated tests:
      - `test_blur_params_uniform_size_matches_wgsl`
      - `test_blur_params_pack_weights`
  - Updated `crates/render-engine/src/backend/shaders/blur.wgsl`:
    - replaced `weights: array<f32, 25>` with `packed_weights: array<vec4<f32>, 7>`.
    - added `blur_weight(index)` helper for packed lookup.
  - Removed temporary debug print from `render_scene_to_rgba(...)` in `crates/render-engine/src/backend/wgpu/mod.rs`.
  - Gated native-only readback helpers in `crates/render-engine/src/backend/wgpu/context.rs` with `#[cfg(not(target_arch = "wasm32"))]` to keep wasm checks warning-free.
  - Regenerated desktop golden images with fixed blur output:
    - `tests/visual/golden/phase4/blur.png`
    - `tests/visual/golden/phase4/blend.png`
    - `tests/visual/golden/phase4/clipping.png`
- Verification:
  - `cargo test -p render-engine test_blur_params_uniform_size_matches_wgsl -- --nocapture` -> PASS
  - `cargo run --example capture_phase4_visuals -- --out .tmp/phase4-debug` -> PASS (no blur pipeline validation errors)
  - `cargo run --example capture_phase4_visuals` -> PASS (goldens updated)
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS
  - `cargo test -p render-engine --test phase4_effect_plan_tests --test phase4_render_target_pool_tests --test phase4_blur_execution_tests --test phase4_background_blur_tests --test phase4_inner_shadow_tests --test phase4_stencil_tests --test phase4_blend_modes_tests --test phase4_effects_tests -- --nocapture` -> PASS
  - `cargo test -p plat-core --test web_event_mapping_tests -- --nocapture` -> PASS
  - `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web` -> PASS
  - `cargo fmt --all --check` -> PASS

### 2026-02-11 (parity continuation: `isMask`/`maskType` + non-placeholder `Paint::Image`)

- Goal:
  - Close remaining mapping gaps from the Phase 4/5 parity table:
    - `isMask` / `maskType`
    - `fills[].type: IMAGE`
- Implementation:
  - `style-engine` mask model:
    - Added `MaskType` enum in `crates/style-engine/src/visual.rs`:
      - `Alpha`, `Vector`, `Luminance`
    - Extended `VisualStyle` with:
      - `is_mask: bool` (`serde` alias `isMask`)
      - `mask_type: MaskType` (`serde` alias `maskType`)
    - Added builder methods:
      - `VisualStyle::is_mask(...)`
      - `VisualStyle::mask_type(...)`
    - Exported `MaskType` in `crates/style-engine/src/lib.rs`.
  - `render-engine` mask clipping behavior:
    - Added ancestor sibling-mask resolution in `crates/render-engine/src/backend/wgpu/mod.rs`:
      - `ancestor_mask_bounds(...)`
      - integrated into `clipped_bounds_for_node(...)`
    - Behavior:
      - last preceding mask sibling per ancestor level clips subsequent sibling bounds.
      - combined with existing `clips_content` clipping.
  - `render-engine` image fill behavior:
    - Added CPU image registry + sampling module:
      - `crates/render-engine/src/backend/wgpu/image_store.rs`
      - APIs:
        - `register_image_rgba8(...)`
        - `unregister_image(...)`
        - `sample_image_fill(...)`
      - scale modes handled: `Fill`, `Fit`, `Crop`, `Tile`
      - optional 3x3 UV transform support.
    - Exposed backend registration API in `WgpuBackend`:
      - `register_image_rgba8(...)`
      - `unregister_image(...)`
    - Updated path sampling in `crates/render-engine/src/backend/wgpu/pipelines/path_pipeline.rs`:
      - `Paint::Image` now samples registered image data instead of magenta fallback.
    - Updated instance collection in `crates/render-engine/src/backend/wgpu/mod.rs`:
      - image-filled rectangle nodes are routed through path batches (no primitive magenta fallback).
- Tests added:
  - `crates/style-engine/src/visual.rs`:
    - `test_builder_mask_fields`
    - `test_deserialize_figma_mask_aliases`
  - `crates/render-engine/src/backend/wgpu/mod.rs`:
    - `test_image_fill_rect_is_routed_to_path_batches`
    - `test_mask_node_clips_subsequent_sibling_bounds`
    - `test_mask_node_does_not_clip_preceding_sibling`
  - `crates/render-engine/src/backend/wgpu/image_store.rs`:
    - `test_register_rejects_invalid_byte_len`
    - `test_sample_image_fill_reads_registered_pixel`
    - `test_sample_image_fill_tile_wraps_uv`
  - `crates/render-engine/src/backend/wgpu/pipelines/path_pipeline.rs`:
    - `test_sample_paint_at_uv_image_fill_reads_registered_image`
- Verification:
  - `cargo test -p style-engine --lib -- --nocapture` -> PASS
  - `cargo test -p render-engine --lib -- --nocapture` -> PASS
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS
  - `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web` -> PASS
  - `cargo fmt --all --check` -> PASS

### 2026-02-11 (visual parity extension: desktop regression now covers `mask` and `image`)

- Goal:
  - Add screenshot-level regression coverage for newly added mask/image rendering behavior.
- Implementation:
  - Extended shared scene fixtures in `examples/phase4_visual_scenes.rs`:
    - `build_phase4_mask_scene()`
    - `build_phase4_image_scene()`
    - added `PHASE4_IMAGE_TEST_ID` fixture ID.
  - Updated desktop capture harness `examples/capture_phase4_visuals.rs`:
    - registers deterministic test image bytes via `WgpuBackend::register_image_rgba8(...)`.
    - captures additional cases:
      - `mask`
      - `image`
  - Updated desktop regression test `tests/phase4_desktop_visual_regression.rs`:
    - registers same deterministic image asset.
    - compares `mask` and `image` outputs against committed goldens.
  - Updated docs `docs/testing/phase4-desktop-visual-regression.md` with new coverage/golden list.
  - Generated new goldens:
    - `tests/visual/golden/phase4/mask.png`
    - `tests/visual/golden/phase4/image.png`
- Verification:
  - `cargo run --example capture_phase4_visuals` -> PASS
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS
  - `cargo test -p render-engine --lib -- --nocapture` -> PASS
  - `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web` -> PASS
  - `cargo fmt --all --check` -> PASS

### 2026-02-11 (visual tuning follow-up: blur compositing write mode + clipping palette)

- User feedback:
  - blend/blur output looked off and clipping sample had a distracting yellow rectangle.
- Implementation:
  - Updated `crates/render-engine/src/backend/wgpu/pipelines/blur_pipeline.rs`:
    - extracted `blur_color_target_state(...)`.
    - switched blur pass color target to replace writes (`blend: None`) to avoid double alpha blending in fullscreen blur compositing.
    - added unit test `test_blur_pipeline_uses_replace_target_state`.
  - Updated `examples/phase4_visual_scenes.rs`:
    - changed clipping scene accent (`escape_rect`) fill from saturated yellow to cyan-blue to remove random yellow outlier.
  - Regenerated desktop phase4 goldens after scene/pipeline updates:
    - `tests/visual/golden/phase4/blur.png`
    - `tests/visual/golden/phase4/blend.png`
    - `tests/visual/golden/phase4/clipping.png`
    - `tests/visual/golden/phase4/mask.png`
    - `tests/visual/golden/phase4/image.png`
- Verification:
  - `cargo fmt --all` -> PASS
  - `cargo test -p render-engine test_blur_pipeline_uses_replace_target_state --lib -- --nocapture` -> PASS
  - `cargo run --example capture_phase4_visuals` -> PASS
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS

### 2026-02-11 (root-cause fix: multipass UV orientation + blend alpha compositing + regression hardening)

- User feedback:
  - blend and blur visuals were still incorrect after initial tuning.
- Root cause investigation (systematic):
  - Added failing regression guard in `tests/phase4_desktop_visual_regression.rs`:
    - `phase4_blend_modes_render_distinct_outputs`
    - verified first two blend columns were pixel-identical before fix (RED).
  - Traced multipass execution path in `crates/render-engine/src/backend/wgpu/mod.rs` and confirmed per-node blend modes were being passed correctly.
  - Identified shader-space mismatch:
    - fullscreen triangle UVs in `blur.wgsl` and `blend.wgsl` sampled with incorrect Y orientation relative to render-target reads/writes.
    - this caused multipass source/destination sampling to come from wrong rows, making blend columns collapse and blur composition look wrong.
- Implementation:
  - `crates/render-engine/src/backend/shaders/blur.wgsl`
    - corrected fullscreen UV mapping in `vs_main`:
      - `out.uv = vec2<f32>(p.x * 0.5 + 0.5, 1.0 - (p.y * 0.5 + 0.5));`
  - `crates/render-engine/src/backend/shaders/blend.wgsl`
    - corrected fullscreen UV mapping with same Y flip.
    - retained alpha-correct source-over blend compositing:
      - unpremultiply sampled RGBA
      - apply blend function in straight-color space
      - recomposite with Porter-Duff source-over into premultiplied output.
  - `crates/render-engine/src/backend/wgpu/effects.rs`
    - added CPU reference compositing helper:
      - `composite_blend_over(mode, src, dst)`
    - added internal `apply_blend_mode(...)` helper.
  - `crates/render-engine/tests/phase4_blend_modes_tests.rs`
    - added RED/GREEN coverage:
      - `test_composite_blend_over_preserves_destination_when_source_alpha_zero`
      - `test_composite_blend_over_uses_source_alpha_for_blend_mix`
  - `tests/phase4_desktop_visual_regression.rs`
    - added blend distinctness visual test:
      - `phase4_blend_modes_render_distinct_outputs`
      - asserts blend columns differ and Screen is brighter than Multiply at overlap sample.
  - `examples/phase4_visual_scenes.rs`
    - clipping accent kept as cyan-blue (removed yellow outlier from previous follow-up).
- Regenerated goldens after final fix:
  - `tests/visual/golden/phase4/blur.png`
  - `tests/visual/golden/phase4/blend.png`
  - `tests/visual/golden/phase4/clipping.png`
  - `tests/visual/golden/phase4/mask.png`
  - `tests/visual/golden/phase4/image.png`
- Verification:
  - `cargo test -p render-engine --test phase4_blend_modes_tests -- --nocapture` -> PASS
  - `cargo test --test phase4_desktop_visual_regression phase4_blend_modes_render_distinct_outputs -- --nocapture` -> PASS
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS
  - `cargo test -p render-engine --lib -- --nocapture` -> PASS
  - `cargo run --example capture_phase4_visuals` -> PASS
  - `cargo fmt --all --check` -> PASS

### 2026-02-11 (mask visual clarity improvement + regression stability)

- User feedback:
  - mask golden did not clearly demonstrate clipping behavior.
- Implementation:
  - Updated `examples/phase4_visual_scenes.rs` `build_phase4_mask_scene()` to make mask behavior visually explicit:
    - added backdrop panel behind mask demo area.
    - changed mask node to a large rounded shape (`corner_radius(140.0)`) centered in panel.
    - enlarged post-mask gradient layer so clipping boundary is obvious.
    - added additional oversized post-mask siblings (`masked_bar`, `masked_chip`) so all are clipped by the same mask.
    - retained pre-mask blue sibling as unmasked control element.
  - Regenerated goldens with updated mask fixture:
    - `tests/visual/golden/phase4/mask.png` (plus full phase4 set via capture example).
  - Improved regression determinism in `tests/phase4_desktop_visual_regression.rs`:
    - added process-local mutex (`OnceLock<Mutex<()>>`) and lock guard in both tests.
    - prevents parallel test execution races when both tests initialize/render through native WGPU paths concurrently.
- Verification:
  - `cargo run --example capture_phase4_visuals` -> PASS
  - `cargo test --test phase4_desktop_visual_regression phase4_desktop_visual_regression_matches_golden -- --nocapture` -> PASS
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS (both tests together)

### 2026-02-11 (image visual fidelity fix: path image sampling density)

- User feedback:
  - image golden did not look like an image (looked like flat/gradient regions instead of sampled content).
- Root cause:
  - `Paint::Image` in `crates/render-engine/src/backend/wgpu/pipelines/path_pipeline.rs` was sampled only at tessellated mesh vertices.
  - rectangle/path fill meshes have sparse vertices (often corners), so GPU interpolation blurred/collapsed image detail.
- Implementation:
  - Added adaptive image-batch subdivision in `path_pipeline.rs` during `PathPipeline::prepare(...)`:
    - `image_subdivision_steps(...)`
    - `append_subdivided_image_triangle(...)`
    - `append_batch_geometry(...)`
    - `push_batch_vertex(...)`
    - `sample_batch_color(...)`
  - Behavior:
    - non-image paints keep existing geometry path (no behavior change).
    - image paints subdivide triangles before upload so interior texels are sampled across the surface.
    - subdivision is capped by:
      - `IMAGE_SUBDIVISION_TARGET_PIXELS`
      - `IMAGE_SUBDIVISION_MAX`
      - `IMAGE_SUBDIVISION_TRIANGLE_BUDGET`
  - Added regression tests in `path_pipeline.rs`:
    - `test_append_batch_geometry_subdivides_image_batches_and_samples_interior`
    - updated `test_sample_paint_at_uv_image_fill_reads_registered_image` to cleanup image registration after test.
  - Regenerated desktop phase4 goldens:
    - `tests/visual/golden/phase4/image.png` now shows clear marker sprite across fill/fit/tile cases.
- Verification:
  - `cargo test -p render-engine path_pipeline -- --nocapture` -> PASS (9/9)
  - `cargo run --example capture_phase4_visuals` -> PASS
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS (2/2)

### 2026-02-12 (success-criteria closeout: Figma JSON -> render regression test)

- Goal:
  - Close success criterion #2 from `docs/plans/2026-02-08-figma-rendering-pipeline-design.md` with an automated end-to-end test:
    - Figma-style JSON fixture -> Arthropod scene/style mapping -> desktop render -> golden comparison.
- Implementation:
  - Added Figma-style fixture:
    - `tests/fixtures/figma/figma_import_scene.json`
    - includes scene metadata, nodes, style properties, and image asset metadata.
  - Added desktop integration regression:
    - `tests/figma_json_render_regression.rs`
    - parses fixture JSON into local DTOs.
    - maps Figma-style properties to engine types:
      - paints: solid / linear gradient / image
      - stroke: `strokeWeight`, `strokeAlign`, `individualStrokeWeights`
      - effects: drop/inner shadow, layer/background blur
      - corner radius, opacity, blend mode, clips content, mask fields
    - builds `Scene` with `NodeContent::Styled`.
    - registers deterministic fixture images via `WgpuBackend::register_image_rgba8(...)`.
    - renders offscreen with `render_scene_to_rgba(...)`.
    - compares against golden with tolerance.
    - supports golden refresh via `ARTHROPOD_UPDATE_GOLDENS=1`.
  - Added committed golden:
    - `tests/visual/golden/figma/figma_import_scene.png`
  - Added root test-only serde derive dependency in `Cargo.toml`:
    - `[dev-dependencies] serde = { version = "1.0", features = ["derive"] }`
- Verification:
  - `ARTHROPOD_UPDATE_GOLDENS=1 cargo test --test figma_json_render_regression -- --nocapture` -> PASS
  - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS
  - `cargo clippy --test figma_json_render_regression -- -D warnings` -> PASS

### 2026-02-12 (cross-browser wasm visual smoke wiring)

- Goal:
  - Add and validate browser-project coverage for wasm visual smoke checks.
- Implementation:
  - Updated Playwright project config in `playwright.config.mjs`:
    - added `chromium` and `firefox` projects.
    - added browser launch options to request WebGPU support.
  - Updated npm scripts in `package.json`:
    - `visual:test:chromium`
    - `visual:test:firefox`
  - Hardened web fixture startup signaling in `examples/phase4_visual_web.rs`:
    - added error marker path:
      - `data-arthropod-ready="0"`
      - `data-arthropod-error="<message>"`
    - startup failures now publish explicit DOM marker instead of timing out silently.
  - Updated Playwright test logic in `tests/visual/phase4-visual.spec.mjs`:
    - waits for ready marker presence (`0` or `1`).
    - skips only when startup error explicitly indicates WebGPU unavailable.
    - fails fast for all other startup errors.
  - Updated usage docs in `docs/testing/phase4-visual-regression.md` with:
    - multi-browser setup commands.
    - browser-specific run commands.
    - readiness/error marker behavior.
- Verification:
  - `cargo check --example phase4_visual_web --target wasm32-unknown-unknown --features web` -> PASS
  - `npm run visual:test:chromium` -> PASS (3 skipped due WebGPU unavailable in this runtime)
  - `npm run visual:test:firefox` -> PASS (3 skipped due WebGPU unavailable in this runtime)

### 2026-02-12 (continuation verification on current head)

- Goal:
  - Re-verify the key desktop + web Phase 4/5 gates after latest commits.
- Verification:
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS (2/2)
  - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (1/1)
  - `cargo test -p plat-core --test web_event_mapping_tests -- --nocapture` -> PASS (5/5)
  - `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web` -> PASS
  - `npm run visual:test` -> PASS (6 skipped due WebGPU unavailable in this runtime)

### 2026-02-13 (phase4 image golden sharpness improvement)

- User feedback:
  - `tests/visual/golden/phase4/image.png` looked noticeably blurry.
- Root cause:
  - `Paint::Image` path rendering still shades by interpolating vertex colors.
  - Existing image subdivision density (target 12px, max 32, budget 4096) was too coarse for scaled image fills, producing soft edges.
- Implementation:
  - Updated image subdivision tuning in `crates/render-engine/src/backend/wgpu/pipelines/path_pipeline.rs`:
    - `IMAGE_SUBDIVISION_TARGET_PIXELS`: `12.0` -> `4.0`
    - `IMAGE_SUBDIVISION_MAX`: `32` -> `128`
    - `IMAGE_SUBDIVISION_TRIANGLE_BUDGET`: `4096` -> `16_384`
  - Added regression unit test:
    - `test_image_subdivision_steps_dense_for_scaled_images` (expects dense tessellation for 300x170 image fills).
  - Increased deterministic phase4 image fixture resolution in both harnesses:
    - `examples/capture_phase4_visuals.rs`
    - `tests/phase4_desktop_visual_regression.rs`
    - fixture size: `64x48` -> `192x144`
  - Regenerated desktop phase4 goldens:
    - `tests/visual/golden/phase4/image.png`
- Verification:
  - `cargo test -p render-engine path_pipeline -- --nocapture` -> PASS
  - `cargo fmt --all` -> PASS
  - `cargo run --example capture_phase4_visuals` -> PASS
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS

### 2026-02-21 (parity continuation: multi-layer stroke paints + image `STRETCH` scale mode)

- Goal:
  - Continue closing remaining Figma parity deltas after Phase 4/5 closeout:
    - render all stroke paint layers (not just top paint)
    - support REST `IMAGE.scaleMode = STRETCH`
- Implementation:
  - Multi-layer stroke paint compositing:
    - Updated `crates/render-engine/src/backend/wgpu/pipelines/primitive_builder.rs`:
      - stroke emission now iterates all `StrokeStyle.paints` bottom-to-top.
      - applies to both pipeline and no-pipeline paths.
    - Updated `crates/render-engine/src/backend/wgpu/instance_collector.rs`:
      - path stroke routing now emits one `PathBatch` per stroke paint layer.
      - applies to scene collection and `collect_style_batches_for_bounds(...)`.
    - Added regression tests:
      - `test_create_primitive_instances_emits_all_stroke_paints_in_order`
      - `test_collect_instances_adds_batch_per_stroke_paint_layer`
  - Image stretch scale mode:
    - Added `ImageScaleMode::Stretch` in `crates/style-engine/src/paint.rs`.
    - Added stretch handling in `crates/render-engine/src/backend/wgpu/image_store.rs`.
    - Added stretch hash-key branch in
      `crates/render-engine/src/backend/wgpu/pipelines/path_pipeline.rs`.
    - Extended Figma JSON mapping in `tests/figma_json_render_regression.rs`:
      - `FigmaImageScaleMode::Stretch` -> `ImageScaleMode::Stretch`.
    - Added regression tests:
      - `test_sample_image_fill_stretch_preserves_full_image_range`
      - `figma_image_scale_mode_stretch_maps_to_style_engine_stretch`
  - Browser parity wiring:
    - Added Playwright WebKit project in `playwright.config.mjs`.
    - Added npm script `visual:test:webkit` in `package.json`.
    - Updated `docs/testing/phase4-visual-regression.md` for WebKit/Safari.
  - Restored wasm example gate for web target checks:
    - Updated root `Cargo.toml` dev-dependency gating:
      - moved `flux-devtools` from global `[dev-dependencies]` to
        `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`.
      - avoids pulling `crossterm` into `wasm32` example checks.
- Verification:
  - `cargo test -p render-engine --lib -- --nocapture` -> PASS
  - `cargo test -p style-engine --lib -- --nocapture` -> PASS
  - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (after golden refresh)
  - `cargo test --test figma_json_render_regression figma_image_scale_mode_stretch_maps_to_style_engine_stretch -- --nocapture` -> PASS
  - `cargo check --example widget_gallery_web --target wasm32-unknown-unknown --features web` -> PASS
  - `cargo bench -p render-engine --bench phase4_effects -- --noplot` -> PASS
  - `cargo clippy -p render-engine --lib -- -D warnings` -> PASS
  - `cargo clippy -p style-engine --lib -- -D warnings` -> PASS
  - `cargo fmt --all --check` -> PASS
  - `npm install` -> PASS
  - `npx playwright install webkit` -> PASS
  - `npm run visual:test:chromium` -> PASS (3 skipped: WebGPU unavailable)
  - `npm run visual:test:firefox` -> PASS (3 skipped: WebGPU unavailable)
  - `npm run visual:test:webkit` -> PASS (3 skipped: WebGPU unavailable)
  - `npm run visual:test` -> PASS (9 skipped: WebGPU unavailable across projects)

### 2026-02-21 (parity continuation: multi-layer fill routing in path/image collection)

- Goal:
  - Close additional Figma fill-stack parity gaps in batch collection:
    - ensure `fill_geometry` emits one path batch per fill paint layer (not just first fill)
    - ensure rectangle styles containing any `Paint::Image` route the full fill stack through path batches (avoids primitive magenta fallback for mixed fill stacks)
- Implementation:
  - Updated `crates/render-engine/src/backend/wgpu/instance_collector.rs`:
    - replaced single-fill resolver with `resolve_path_fill_paints(...)`.
    - `collect_instances_impl(...)` now:
      - emits one `PathBatch` per fill layer for vector fill geometry.
      - routes any image-containing rectangle fill stack through path batches.
    - `collect_style_batches_for_bounds(...)` mirrors the same behavior for multipass/offscreen paths.
  - Added regression tests in `crates/render-engine/src/backend/wgpu/tests.rs`:
    - `test_collect_instances_adds_batch_per_fill_paint_layer_for_path_geometry`
    - `test_image_fill_rect_with_multiple_fill_layers_routes_all_fills_to_path_batches`
- Verification:
  - RED:
    - `cargo test -p render-engine test_collect_instances_adds_batch_per_fill_paint_layer_for_path_geometry -- --nocapture` -> FAIL (expected: only first fill batch emitted)
    - `cargo test -p render-engine test_image_fill_rect_with_multiple_fill_layers_routes_all_fills_to_path_batches -- --nocapture` -> FAIL (expected: primitive fallback path engaged)
  - GREEN:
    - `cargo test -p render-engine test_collect_instances_adds_batch_per_fill_paint_layer_for_path_geometry -- --nocapture` -> PASS
    - `cargo test -p render-engine test_image_fill_rect_with_multiple_fill_layers_routes_all_fills_to_path_batches -- --nocapture` -> PASS
  - Regression safety:
    - `cargo test -p render-engine --lib -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS
    - `cargo clippy -p render-engine --lib -- -D warnings` -> PASS
    - `cargo fmt --all` -> PASS

### 2026-02-21 (parity continuation: Figma JSON mapping for corner smoothing + stroke metadata)

- Goal:
  - Expand Figma JSON parity mapping coverage in the regression harness for properties present in the master mapping table:
    - `cornerSmoothing` -> `VisualStyle.corner_smoothing`
    - `strokeCap` -> `StrokeStyle.cap`
    - `strokeJoin` -> `StrokeStyle.join`
    - `strokeDashes` -> `StrokeStyle.dash_pattern`
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - Added `FigmaStyle` fields:
      - `corner_smoothing`
      - `stroke_cap`
      - `stroke_join`
      - `stroke_dashes`
    - Added enums and mappings:
      - `FigmaStrokeCap` -> `StrokeCap`
      - `FigmaStrokeJoin` -> `StrokeJoin`
    - Applied mapping in `FigmaStyle::into_visual_style(...)`.
  - Added regression tests:
    - `figma_style_corner_smoothing_maps_to_visual_style_corner_smoothing`
    - `figma_style_stroke_cap_join_and_dashes_map_to_stroke_style`
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_style_corner_smoothing_maps_to_visual_style_corner_smoothing -- --nocapture` -> FAIL
    - `cargo test --test figma_json_render_regression figma_style_stroke_cap_join_and_dashes_map_to_stroke_style -- --nocapture` -> FAIL
  - GREEN:
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS
  - Additional gates:
    - `cargo clippy -p render-engine --lib -- -D warnings` -> PASS
    - `cargo fmt --all` -> PASS
    - `cargo clippy --test figma_json_render_regression -- -D warnings` -> FAIL due pre-existing unrelated warning in `crates/arthropod/src/app/core.rs:116` (`unused_mut`)

### 2026-02-21 (parity continuation: Figma JSON gradient variant mapping coverage)

- Goal:
  - Extend the Figma JSON parity harness to cover additional fill paint gradient variants from the mapping table:
    - `GRADIENT_RADIAL`
    - `GRADIENT_ANGULAR`
    - `GRADIENT_DIAMOND`
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - Added `FigmaPaint` variants:
      - `GradientRadial { center, radius, stops }`
      - `GradientAngular { center, angle, stops }`
      - `GradientDiamond { center, scale, stops }`
    - Added conversion mapping in `FigmaPaint::into_paint(...)` to:
      - `Paint::Radial(RadialGradient { ... })`
      - `Paint::Angular(AngularGradient { ... })`
      - `Paint::Diamond(DiamondGradient { ... })`
    - Added regression test:
      - `figma_gradient_variants_map_to_style_engine_gradient_paints`
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_gradient_variants_map_to_style_engine_gradient_paints -- --nocapture`
      - FAIL (`unknown variant GRADIENT_RADIAL`)
  - GREEN:
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS
  - Hygiene:
    - `cargo fmt --all` -> PASS
  - Parity status check:
    - `npm run visual:test` -> PASS (9 skipped; WebGPU unavailable in this runtime)

### 2026-02-21 (parity continuation: path gradients now evaluated in shader)

- Goal:
  - Close the remaining documented Phase 3 deviation where `PathPipeline` pre-baked gradient colors per vertex on CPU.
- Implementation:
  - Updated `crates/render-engine/src/backend/wgpu/pipelines/path_pipeline.rs`:
    - Extended `PathGpuVertex` payload to include:
      - normalized UV
      - flat paint metadata (`fill_type`, `gradient_index`)
    - Added gradient atlas/params state to `PathPipeline` (matching primitive pipeline strategy):
      - `GradientAtlas`
      - gradient params storage buffer + bind group
    - `prepare(...)` now:
      - registers path gradient stops in atlas
      - stores per-batch gradient params
      - uploads gradient resources before draw
    - `append_batch_geometry(...)` now:
      - emits neutral base color for gradient paints (opacity in alpha)
      - keeps CPU sampling path for `Paint::Image` (with subdivision)
  - Updated `crates/render-engine/src/backend/shaders/path.wgsl`:
    - Added gradient bind group + storage buffer definitions.
    - Added per-fragment gradient sampling logic for linear/radial/angular/diamond.
    - Fragment shader now evaluates gradient color from UV + params when `fill_type != 0`.
  - Added regression test:
    - `test_append_batch_geometry_defers_linear_gradient_to_shader_path`
- Verification:
  - RED:
    - `cargo test -p render-engine test_append_batch_geometry_defers_linear_gradient_to_shader_path -- --nocapture` -> FAIL
  - GREEN:
    - `cargo test -p render-engine test_append_batch_geometry_defers_linear_gradient_to_shader_path -- --nocapture` -> PASS
  - Regression safety:
    - `cargo test -p render-engine --lib -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS
    - `cargo clippy -p render-engine --lib -- -D warnings` -> PASS
    - `cargo fmt --all` -> PASS

### 2026-02-21 (parity continuation: rounded corners preserved for image-filled rect routing)

- Goal:
  - Preserve `cornerRadius`/`rectangleCornerRadii` behavior when rectangle nodes with image fills are routed through path batches.
- Implementation:
  - Added `rounded_rect_path_for_size(...)` in `crates/render-engine/src/backend/wgpu/clipping.rs`:
    - builds a rounded rectangle `VectorPath` with quadratic corner segments.
    - normalizes corner radii against width/height constraints.
  - Updated image-fill routing in `crates/render-engine/src/backend/wgpu/instance_collector.rs`:
    - replaced sharp `rect_path_for_size(...)` usage with `rounded_rect_path_for_size(...)` in:
      - scene collection path
      - `collect_style_batches_for_bounds(...)` multipass path
  - Added regression test:
    - `test_image_fill_rect_with_corner_radius_uses_rounded_path_geometry`
- Verification:
  - RED:
    - `cargo test -p render-engine test_image_fill_rect_with_corner_radius_uses_rounded_path_geometry -- --nocapture` -> FAIL
  - GREEN:
    - `cargo test -p render-engine test_image_fill_rect_with_corner_radius_uses_rounded_path_geometry -- --nocapture` -> PASS
  - Regression safety:
    - `cargo test -p render-engine --lib -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS
    - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS
    - `cargo clippy -p render-engine --lib -- -D warnings` -> PASS
    - `cargo fmt --all` -> PASS
  - Parity status check:
    - `npm run visual:test` -> PASS (9 skipped; WebGPU unavailable in this runtime)

### 2026-02-24 (parity continuation: Figma fixture hierarchy + mask/clip scope)

- Goal:
  - Close hierarchy parity gap in the JSON fixture importer by mapping `parentId` relationships into the scene tree instead of flattening all nodes under scene root.
  - Add nested fixture coverage so mask/clip scope behavior is exercised through hierarchical import.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - extended `FigmaNodeFixture` with optional `id` and `parent_id`.
    - added hierarchy regression test:
      - `figma_node_parent_id_maps_to_scene_hierarchy`
    - replaced flat `build_scene(...)` insertion with parent-aware construction:
      - stable fixture ID resolution
      - parent-first multi-pass insertion
      - deterministic root fallback for dangling `parentId` references
  - Updated fixture `tests/fixtures/figma/figma_import_scene.json`:
    - added explicit `id` across nodes and `parentId` links for nested structure.
    - moved content nodes under a container frame with `clipsContent`.
    - added nested mask scope container (`id: 70`) with mask/target children and an overlapping sibling outside mask scope to validate hierarchy-scoped masking behavior in rendered output.
  - Regenerated figma golden:
    - `tests/visual/golden/figma/figma_import_scene.png`
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_node_parent_id_maps_to_scene_hierarchy -- --nocapture` -> FAIL (`expected one top-level imported node under scene root`, got 4)
  - GREEN:
    - `cargo test --test figma_json_render_regression figma_node_parent_id_maps_to_scene_hierarchy -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> FAIL initially on expected golden drift (`diff_ratio=0.0554`)
    - `ARTHROPOD_UPDATE_GOLDENS=1 cargo test --test figma_json_render_regression figma_json_render_regression_matches_golden -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS
  - Regression safety:
    - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS
    - `cargo fmt --all` -> PASS

### 2026-02-24 (parity continuation: CI gate includes figma visual regression)

- Goal:
  - Prevent regressions in hierarchy-aware Figma import parity by enforcing `figma_json_render_regression` in CI alongside desktop Phase 4 visual regression.
- Implementation:
  - Updated `.github/workflows/ci.yml` job `phase4-desktop-visual-regression`:
    - expanded job comment to include figma golden checks.
    - added step:
      - `cargo test --test figma_json_render_regression -- --nocapture`
    - added artifact upload on failure for figma outputs:
      - `tests/visual/artifacts/figma`
- Verification:
  - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS
  - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS

### 2026-02-24 (parity continuation: enforce desktop visual parity on PRs)

- Goal:
  - Shift visual-parity failures left by running the Windows desktop visual regression gate for pull requests, not only trunk/manual runs.
- Implementation:
  - Updated `.github/workflows/ci.yml`:
    - job `phase4-desktop-visual-regression` now runs when `github.event_name == 'pull_request'` in addition to existing push/manual paths.
  - This keeps both visual gates PR-blocking:
    - `phase4_desktop_visual_regression`
    - `figma_json_render_regression`
- Verification:
  - Local command checks remain green:
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS
    - `cargo test --test phase4_desktop_visual_regression -- --nocapture` -> PASS

### 2026-02-24 (parity continuation: dedicated headed Chromium WebGPU gate)

- Goal:
  - Add a browser parity gate that validates real WebGPU adapter availability in headed Chromium, rather than relying on headless skip behavior.
- Implementation:
  - Added adapter gate test:
    - `tests/visual/webgpu-adapter.spec.mjs`
    - asserts `navigator.gpu.requestAdapter()` resolves to a non-null adapter under headed Chromium.
  - Added npm scripts in `package.json`:
    - `visual:test:chromium:headed:adapter`
    - `visual:test:webgpu:headed` (adapter gate + headed phase4 snapshots)
  - Added dedicated workflow:
    - `.github/workflows/webgpu-headed-visual.yml`
    - triggers: `workflow_dispatch` + nightly schedule
    - runs on `windows-latest`
    - installs Trunk + Playwright Chromium
    - executes headed adapter gate then headed phase4 visual regression
    - uploads visual artifacts on failure
  - Updated docs:
    - `docs/testing/phase4-visual-regression.md` with new commands and workflow reference.
- Verification:
  - `npm run visual:test:chromium:headed:adapter` -> PASS
  - `npm run visual:test:webgpu:headed` -> PASS

### 2026-02-27 (parity continuation: browser fixture parity with desktop scenes + mask/image coverage)

- Goal:
  - Remove browser/desktop visual fixture drift by sharing phase4 scene definitions.
  - Extend browser visual parity coverage from 3 cases to 5 cases (`blur`, `blend`, `clipping`, `mask`, `image`).
- Implementation:
  - Updated `examples/phase4_visual_web.rs`:
    - switched scene construction to shared `examples/phase4_visual_scenes.rs` builders.
    - extended query cases with `mask` and `image`.
    - added deterministic image fixture registration for image-case rendering (`PHASE4_IMAGE_TEST_ID`).
    - hardened startup/render failure signaling:
      - sanitized `data-arthropod-error` marker values
      - render failures now set ready/error markers instead of only logging.
  - Updated Playwright browser visual test:
    - `tests/visual/phase4-visual.spec.mjs` now includes `mask` and `image`.
    - added per-test WebGPU adapter preflight (`navigator.gpu.requestAdapter()`) so unsupported runtimes skip cleanly before startup waits.
  - Updated Chromium headed snapshots:
    - regenerated `phase4-clipping-chromium-win32.png` (scene parity alignment)
    - added `phase4-mask-chromium-win32.png`
    - added `phase4-image-chromium-win32.png`
  - Updated docs:
    - `docs/testing/phase4-visual-regression.md` includes mask/image case coverage and adapter preflight behavior.
- Verification:
  - `cargo fmt --all` -> PASS
  - `cargo check --example phase4_visual_web --target wasm32-unknown-unknown --features web` -> PASS
  - `npm run visual:update:chromium:headed` -> PASS (5/5 snapshots)
  - `npm run visual:test:webgpu:headed` -> PASS (adapter gate + 5/5 chromium headed snapshots)
  - `npm run visual:test` -> PASS (15 skipped in current runtime due missing WebGPU adapters in headless/browser matrix)

### 2026-02-27 (parity continuation: enforce headed WebGPU browser gate on PRs)

- Goal:
  - Shift headed browser WebGPU visual parity failures left by running the dedicated headed Chromium gate on pull requests, not only manual/nightly paths.
- Implementation:
  - Updated `.github/workflows/webgpu-headed-visual.yml` triggers:
    - added `pull_request` for branches `trunk` and `main`
    - retained existing `workflow_dispatch` + nightly schedule
  - Updated docs:
    - `docs/testing/phase4-visual-regression.md` workflow note now reflects PR trigger coverage.
- Verification:
  - `npm run visual:test:webgpu:headed` -> PASS

### 2026-02-27 (parity continuation: Figma `strokeMiterAngle` / `strokeMiterLimit` mapping)

- Goal:
  - Close an additional Figma stroke metadata gap in the JSON parity harness by mapping miter settings into `StrokeStyle.miter_limit`.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs` `FigmaStyle`:
    - added `stroke_miter_angle` (`strokeMiterAngle`)
    - added `stroke_miter_limit` (`strokeMiterLimit`)
  - Updated style mapping in `FigmaStyle::into_visual_style(...)`:
    - if `strokeMiterLimit` is provided and valid (`> 0`, finite), it maps directly to `StrokeStyle.miter_limit`
    - otherwise, `strokeMiterAngle` is converted to miter limit using `1 / sin(angle/2)` (angle in degrees)
    - otherwise defaults to existing stroke default miter limit
  - Added regression tests:
    - `figma_style_stroke_miter_angle_maps_to_miter_limit`
    - `figma_style_explicit_stroke_miter_limit_overrides_angle_mapping`
- Verification:
  - `cargo fmt --all` -> PASS
  - `cargo test --test figma_json_render_regression figma_style_stroke_miter_angle_maps_to_miter_limit -- --nocapture` -> PASS
  - `cargo test --test figma_json_render_regression figma_style_explicit_stroke_miter_limit_overrides_angle_mapping -- --nocapture` -> PASS
  - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (8/8)

### 2026-02-27 (parity continuation: Figma `dashOffset` mapping)

- Goal:
  - Close an additional stroke metadata parity gap by mapping Figma `dashOffset` into `StrokeStyle.dash_offset`.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - Added `FigmaStyle.dash_offset` with `strokeDashOffset` alias support.
    - Applied mapping in `FigmaStyle::into_visual_style(...)`:
      - finite `dashOffset` values map to `StrokeStyle.dash_offset`.
      - non-finite values fall back to default stroke dash offset.
    - Added regression test:
      - `figma_style_dash_offset_maps_to_stroke_style_dash_offset`
  - Updated mapping reference in `docs/plans/2026-02-08-figma-rendering-pipeline-design.md`:
    - added `dashOffset -> StrokeStyle.dash_offset`.
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_style_dash_offset_maps_to_stroke_style_dash_offset -- --nocapture` -> FAIL (expected, unmapped)
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_style_dash_offset_maps_to_stroke_style_dash_offset -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (9/9)

### 2026-02-27 (parity continuation: Figma `fillGeometry` / `strokeGeometry` mapping)

- Goal:
  - Close vector geometry mapping gaps by mapping Figma `fillGeometry` and `strokeGeometry` into `VisualStyle.fill_geometry` / `VisualStyle.stroke_geometry`.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - extended `FigmaStyle` with:
      - `fill_geometry`
      - `stroke_geometry`
    - added geometry parser types:
      - `FigmaPathGeometry` (untagged: SVG path string or object form)
      - `FigmaPathGeometryObject { path, winding_rule }`
      - `FigmaWindingRule` (`NONZERO` / `EVENODD`)
    - added conversion path:
      - parse SVG path strings via `VectorPath::from_svg_path_data`
      - apply optional winding-rule override when provided
      - drop invalid path entries (non-panicking mapper behavior)
    - mapped parsed paths in `FigmaStyle::into_visual_style(...)` to:
      - `style.fill_geometry(...)`
      - `style.stroke_geometry(...)`
  - Added regression tests:
    - `figma_style_fill_geometry_maps_svg_paths_with_winding_rule`
    - `figma_style_stroke_geometry_maps_svg_path_strings`
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_style_fill_geometry_maps_svg_paths_with_winding_rule -- --nocapture` -> FAIL (expected, unmapped)
    - `cargo test --test figma_json_render_regression figma_style_stroke_geometry_maps_svg_path_strings -- --nocapture` -> FAIL (expected, unmapped)
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_style_fill_geometry_maps_svg_paths_with_winding_rule -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression figma_style_stroke_geometry_maps_svg_path_strings -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (11/11)

### 2026-02-27 (parity continuation: fixture-level vector geometry coverage)

- Goal:
  - Ensure vector-geometry parity is exercised by the real fixture/golden path, not only isolated style-mapping tests.
- Implementation:
  - Updated fixture `tests/fixtures/figma/figma_import_scene.json`:
    - added node `80` with `fillGeometry` (object form + `windingRule: EVENODD`) to exercise filled vector path import.
    - added node `81` with `strokeGeometry` (string form) to exercise stroked vector path import.
  - Added fixture sanity regression in `tests/figma_json_render_regression.rs`:
    - `figma_fixture_includes_vector_geometry_render_cases`
    - validates the fixture maps at least one node with non-empty `fill_geometry` and one with non-empty `stroke_geometry`.
  - Regenerated figma golden to include the new fixture geometry output:
    - `tests/visual/golden/figma/figma_import_scene.png`
- Verification:
  - `cargo fmt --all` -> PASS
  - `cargo test --test figma_json_render_regression figma_fixture_includes_vector_geometry_render_cases -- --nocapture` -> PASS
  - `cargo test --test figma_json_render_regression figma_json_render_regression_matches_golden -- --nocapture` -> FAIL (expected golden drift, `diff_ratio=0.0259`)
  - `ARTHROPOD_UPDATE_GOLDENS=1 cargo test --test figma_json_render_regression figma_json_render_regression_matches_golden -- --nocapture` -> PASS
  - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (12/12)

### 2026-02-27 (parity continuation: `fillGeometry.pathData` alias support)

- Goal:
  - Improve fixture/import compatibility for geometry object variants that encode path data as `pathData` instead of `path`.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - `FigmaPathGeometryObject.path` now accepts `pathData` alias.
    - added regression test:
      - `figma_style_fill_geometry_path_data_alias_maps`
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_style_fill_geometry_path_data_alias_maps -- --nocapture` -> FAIL (`untagged enum FigmaPathGeometry` deserialize mismatch)
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_style_fill_geometry_path_data_alias_maps -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (13/13)

### 2026-02-27 (parity continuation: `windingRule` alias `EVEN_ODD`)

- Goal:
  - Improve geometry import compatibility for winding-rule payloads that encode even-odd as `EVEN_ODD`.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - `FigmaWindingRule::EvenOdd` now accepts alias `EVEN_ODD` in addition to `EVENODD`.
    - added regression test:
      - `figma_style_fill_geometry_even_odd_winding_alias_maps`
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_style_fill_geometry_even_odd_winding_alias_maps -- --nocapture` -> FAIL (deserialize mismatch)
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_style_fill_geometry_even_odd_winding_alias_maps -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (14/14)

### 2026-02-27 (parity continuation: image paint `imageTransform` alias mapping)

- Goal:
  - Improve Figma image-paint compatibility for REST/plugin payloads that emit
    transform data under `imageTransform` using a 2x3 affine matrix.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - `FigmaPaint::Image.transform` now accepts alias `imageTransform`.
    - introduced `FigmaImageTransform` (untagged) to accept:
      - flat `[f32; 9]` matrices,
      - 2x3 row matrices (`[[a,b,tx],[c,d,ty]]`), normalized to homogeneous 3x3,
      - 3x3 row matrices.
    - added regression test:
      - `figma_image_paint_image_transform_alias_maps_to_affine_matrix`
  - Updated mapping reference in `docs/plans/2026-02-08-figma-rendering-pipeline-design.md`:
    - added row:
      - `fills[].imageTransform` -> `Paint::Image.transform: Option<[f32; 9]>`
- Verification:
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_image_paint_image_transform_alias_maps_to_affine_matrix -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (15/15)

### 2026-02-27 (parity continuation: image paint `imageRef`/`imageHash` mapping)

- Goal:
  - Improve Figma image-paint import compatibility for REST/plugin payloads that
    identify images with string references (`imageRef`/`imageHash`) instead of numeric `imageId`.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - `FigmaPaint::Image.image_id` now accepts aliases:
      - `imageId`
      - `imageRef`
      - `imageHash`
    - added `FigmaImageId` (untagged) to parse numeric IDs and string references.
    - string references now map to stable `ImageId` values via deterministic hashing.
    - added regression test:
      - `figma_image_paint_image_ref_alias_maps_to_stable_image_id`
  - Updated mapping reference in `docs/plans/2026-02-08-figma-rendering-pipeline-design.md`:
    - added row:
      - `fills[].imageRef` / `fills[].imageHash` -> `Paint::Image.image_id` (stable `ImageId`)
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_image_paint_image_ref_alias_maps_to_stable_image_id -- --nocapture` -> FAIL (`missing field image_id`)
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_image_paint_image_ref_alias_maps_to_stable_image_id -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (16/16)

### 2026-02-27 (parity continuation: image paint `scalingFactor` mapping)

- Goal:
  - Close the remaining tile-image import gap where Figma `scalingFactor` was
    parsed but effectively ignored in parity mapping.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - extended `FigmaPaint::Image` with `scaling_factor` (`scalingFactor` alias).
    - added helper `figma_image_transform(...)` to synthesize a transform when:
      - paint mode is `TILE`,
      - `scalingFactor` is finite and > 0,
      - and no explicit transform was supplied.
    - synthesized matrix uses inverse UV scaling:
      - `scalingFactor = 2.0` -> `[0.5, 0, 0, 0, 0.5, 0, 0, 0, 1]`
    - explicit `imageTransform` remains authoritative when present.
    - added regression test:
      - `figma_image_paint_tile_scaling_factor_maps_to_transform`
  - Updated mapping reference in `docs/plans/2026-02-08-figma-rendering-pipeline-design.md`:
    - added row:
      - `fills[].scalingFactor` -> `Paint::Image.transform` (tile UV scale)
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_image_paint_tile_scaling_factor_maps_to_transform -- --nocapture` -> FAIL (`left: None`, expected synthesized transform)
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_image_paint_tile_scaling_factor_maps_to_transform -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (17/17)

### 2026-02-27 (parity continuation: geometry `windingRule` alias `NONE`)

- Goal:
  - Improve vector-geometry import compatibility for payloads that emit
    `windingRule: "NONE"` on path objects.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - extended `FigmaWindingRule` with alias variant `NONE`.
    - mapped `FigmaWindingRule::None` to `WindingRule::NonZero` as a safe fallback.
    - added regression test:
      - `figma_style_stroke_geometry_none_winding_alias_maps`
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_style_stroke_geometry_none_winding_alias_maps -- --nocapture` -> FAIL (`untagged enum FigmaPathGeometry` deserialize mismatch)
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_style_stroke_geometry_none_winding_alias_maps -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (18/18)

### 2026-02-27 (parity continuation: stroke cap arrow variants)

- Goal:
  - Improve compatibility with Figma vector payloads that use non-basic stroke cap variants
    (for example `LINE_ARROW`) that were previously rejected during deserialization.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - extended `FigmaStrokeCap` to accept:
      - `LINE_ARROW`
      - `TRIANGLE_ARROW`
      - `DIAMOND_FILLED`
      - `CIRCLE_FILLED`
    - mapped these variants to supported runtime caps:
      - `CIRCLE_FILLED` -> `StrokeCap::Round`
      - `LINE_ARROW` / `TRIANGLE_ARROW` / `DIAMOND_FILLED` -> `StrokeCap::Butt`
    - added regression test:
      - `figma_style_stroke_cap_line_arrow_maps_to_supported_fallback`
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_style_stroke_cap_line_arrow_maps_to_supported_fallback -- --nocapture` -> FAIL (`unknown variant LINE_ARROW`)
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_style_stroke_cap_line_arrow_maps_to_supported_fallback -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (19/19)

### 2026-02-27 (parity continuation: unsupported paint types are ignored)

- Goal:
  - Prevent Figma import deserialization failures when encountering paint types
    not yet supported by the runtime (for example `VIDEO` / `PATTERN`).
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - added `FigmaPaint::Unsupported` (`#[serde(other)]`) fallback variant.
    - changed `FigmaPaint::into_paint` to return `Option<Paint>`.
    - updated fill/stroke mapping paths to `filter_map(FigmaPaint::into_paint)` so unsupported paints are skipped.
    - added regression test:
      - `figma_style_unsupported_paint_types_are_ignored`
  - Updated mapping reference in `docs/plans/2026-02-08-figma-rendering-pipeline-design.md`:
    - added row:
      - `fills[].type: VIDEO / PATTERN` -> ignored (unsupported paint fallback)
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_style_unsupported_paint_types_are_ignored -- --nocapture` -> FAIL (`unknown variant VIDEO`)
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_style_unsupported_paint_types_are_ignored -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (20/20)

### 2026-02-27 (parity continuation: unknown blend mode fallback)

- Goal:
  - Avoid full-style deserialization failures when Figma introduces blend-mode
    variants not yet mapped in our runtime enum.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - added `FigmaBlendMode::Unknown` via `#[serde(other)]`.
    - mapped unknown blend modes to `BlendMode::Normal`.
    - added regression test:
      - `figma_style_unknown_blend_mode_falls_back_to_normal`
  - Updated mapping reference in `docs/plans/2026-02-08-figma-rendering-pipeline-design.md`:
    - `blendMode` row now documents unknown-value fallback to `Normal`.
- Verification:
  - RED:
    - `cargo test --test figma_json_render_regression figma_style_unknown_blend_mode_falls_back_to_normal -- --nocapture` -> FAIL (`unknown variant PLUS_DARKER`)
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression figma_style_unknown_blend_mode_falls_back_to_normal -- --nocapture` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (21/21)

### 2026-02-27 (parity closeout: explicit coverage for remaining mapping-table rows)

- Goal:
  - Close the remaining unverified Figma mapping rows by adding direct regression tests
    for stroke align/side weights, corner radii mapping, effect variants, mask/clip/opacity,
    linear gradients, and dash-offset alias behavior.
  - Harden effect parsing so unknown effect types do not fail full style deserialization.
- Implementation:
  - Updated `tests/figma_json_render_regression.rs`:
    - Added row-coverage tests:
      - `figma_style_linear_gradient_maps_to_style_engine_gradient`
      - `figma_style_stroke_align_and_side_weights_map`
      - `figma_style_corner_radius_and_rectangle_corner_radii_map`
      - `figma_style_effect_variants_map_to_visual_effects`
      - `figma_style_opacity_clips_and_mask_map`
      - `figma_style_stroke_dash_offset_alias_maps_to_stroke_style_dash_offset`
    - Added unknown-effect fallback behavior:
      - `FigmaEffect::Unsupported` (`#[serde(other)]`) + `None` mapping in `into_effect`
      - `figma_style_unknown_effect_types_are_ignored`
  - Updated mapping reference in `docs/plans/2026-02-08-figma-rendering-pipeline-design.md`:
    - added row:
      - `effects[].type: unknown` -> ignored (unsupported effect fallback)
- Verification:
  - GREEN:
    - `cargo fmt --all` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (28/28)

Parity status:
- Mapping-table rows in `docs/plans/2026-02-08-figma-rendering-pipeline-design.md` now have explicit regression coverage and/or documented fallback behavior in the harness.

### 2026-02-27 (importer foundation: layout + text + prototype graph skeleton in `arthropod`)

- Goal:
  - Move beyond parity-harness-only mapping by adding a production importer surface in `crates/arthropod`
    that can ingest Figma-like JSON into:
    - `Scene` hierarchy
    - `layout_styles` (`FlexStyle`)
    - constraints metadata
    - prototype interaction graph metadata
    - basic text styling (`TextContent`)
- RED:
  - Added new integration tests first in `crates/arthropod/tests/figma_import_layout.rs`:
    - `figma_auto_layout_maps_to_flex_style_and_hierarchy`
    - `figma_constraints_and_positioning_map_to_imported_constraints`
    - `figma_text_node_maps_characters_and_type_style`
    - `figma_prototype_interactions_map_to_graph_edges`
  - Ran:
    - `cargo test -p arthropod --test figma_import_layout -- --nocapture`
  - Result: FAIL (`unresolved import arthropod::figma`)
- GREEN:
  - Implemented `crates/arthropod/src/figma.rs` with:
    - `import_figma_document(json: &str) -> Result<ImportedFigmaDocument, FigmaImportError>`
    - hierarchy import with parent resolution and root fallback
    - Figma auto-layout mapping to `FlexStyle`:
      - `layoutMode` -> `FlexDirection`
      - `itemSpacing` -> `gap`
      - `padding*` -> paddings
      - sizing modes -> fixed/auto width/height behavior
      - `layoutGrow` -> `flex_grow`
    - constraints mapping:
      - horizontal/vertical constraint axes + layout positioning
    - text import for TEXT nodes:
      - `characters` + style fields to `TextContent`
    - prototype metadata import:
      - `prototypeInteractions` -> `PrototypeGraph.edges`
  - Exported API via `crates/arthropod/src/lib.rs` (`pub mod figma`, `pub use figma::import_figma_document`)
  - Added required deps to `crates/arthropod/Cargo.toml` (`serde`, `serde_json`)
  - Ran:
    - `cargo fmt --all`
    - `cargo test -p arthropod --test figma_import_layout -- --nocapture`
  - Result: PASS (4/4)
- REFACTOR:
  - Hardened constraints parsing to accept normalized tokens and common Figma aliases:
    - `LEFT_RIGHT` / `TOP_BOTTOM` -> `ConstraintAxis::Stretch`
  - Cleaned importer implementation per clippy feedback without behavior changes:
    - replaced `FlexStyle::default()` + reassign pattern with direct struct initialization
    - collapsed nested text-node conditional in `to_visual_style()`
  - Re-ran:
    - `cargo test -p arthropod --test figma_import_layout -- --nocapture` -> PASS (4/4)
    - `cargo test -p arthropod` -> PASS
    - `cargo clippy -p arthropod --tests` -> PASS (warnings remain in unrelated existing modules)
  - Cross-check regression:
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (28/28)

### 2026-02-27 (importer slice: component/instance + variant property mapping)

- Goal:
  - Extend production importer output with component semantics needed for Figma design-import parity:
    - component node metadata (`COMPONENT`, `COMPONENT_SET`)
    - instance linkage metadata (`INSTANCE` -> `componentId` / `mainComponent`)
    - variant property mapping from both `variantProperties` and `componentProperties` (`type: VARIANT`)
- RED:
  - Added integration coverage in `crates/arthropod/tests/figma_import_layout.rs`:
    - `figma_component_and_instance_nodes_map_to_metadata`
    - `figma_variant_properties_extract_from_component_properties_variant_type`
  - Ran:
    - `cargo test -p arthropod --test figma_import_layout -- --nocapture`
  - Result: FAIL (expected):
    - missing importer API surface (`ImportedComponentKind`)
    - missing output fields on `ImportedFigmaDocument` (`components`, `instances`, `variant_properties`)
- GREEN:
  - Updated `crates/arthropod/src/figma.rs`:
    - Added public importer metadata types:
      - `ImportedComponentKind`
      - `ImportedComponentNode`
      - `ImportedInstanceNode`
    - Extended `ImportedFigmaDocument` with:
      - `components: HashMap<NodeId, ImportedComponentNode>`
      - `instances: HashMap<NodeId, ImportedInstanceNode>`
      - `variant_properties: HashMap<NodeId, HashMap<String, String>>`
    - Added Figma parsing support for:
      - node types `COMPONENT`, `COMPONENT_SET`, `INSTANCE`
      - `key`, `componentSetId`, `componentId`, `mainComponent`, `mainComponentId`
      - `variantProperties`
      - `componentProperties` with typed property parsing and variant-only projection
    - Added canonical property-name normalization for Figma component property keys:
      - `State#12:0` -> `State`
  - Ran:
    - `cargo test -p arthropod --test figma_import_layout -- --nocapture`
  - Result: PASS (6/6)
- REFACTOR / verification:
  - Ran:
    - `cargo fmt --all` -> PASS
    - `cargo test -p arthropod` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (28/28)
    - `cargo clippy -p arthropod --tests` -> PASS (warnings remain in unrelated existing modules)

### 2026-02-27 (importer slice 2: prototype transition/action detail mapping)

- Goal:
  - Move prototype import beyond trigger+destination by mapping transition and action semantics needed for enterprise Figma import:
    - animation/transition kind
    - easing + duration normalization
    - direction + `matchLayers`
    - overlay behavior metadata
    - back/close/url/no-destination actions
- RED:
  - Added integration coverage in `crates/arthropod/tests/figma_import_layout.rs`:
    - `figma_prototype_transition_details_map_animation_and_easing`
    - `figma_prototype_overlay_and_back_actions_map_behavior`
  - Ran:
    - `cargo test -p arthropod --test figma_import_layout -- --nocapture`
  - Result: FAIL (expected): missing public prototype action/transition APIs and missing edge fields (`action`, transition details, overlay details, timeout metadata).
- GREEN:
  - Extended public prototype import model in `crates/arthropod/src/figma.rs`:
    - new public enums/structs:
      - `PrototypeActionKind`
      - `PrototypeTransitionKind`
      - `PrototypeEasing`
      - `PrototypeDirection`
      - `PrototypeTransition`
      - `PrototypeOverlayPosition`
      - `PrototypeOverlayBackgroundInteraction`
      - `PrototypeOverlayConfig`
    - expanded `PrototypeEdge`:
      - `to_figma_id: Option<String>`
      - `trigger_timeout_ms`
      - `action`
      - `preserve_scroll_position`
      - `transition`
      - `overlay`
      - `url`
  - Added robust Figma prototype parsing:
    - trigger parsing supports both simple string triggers and object triggers (`{ type, timeout/delay }`)
    - interaction parsing supports `actions[]` + legacy fallback interaction fields
    - action parsing supports:
      - `NAVIGATE`, `OPEN_OVERLAY`, `SWAP_OVERLAY`, `CLOSE_OVERLAY`/`CLOSE`, `BACK`, `URL`/`OPEN_URL`, `SCROLL_TO`
    - transition parsing supports:
      - `type`, `duration`, `easing`, `direction`, `matchLayers`
      - legacy aliases (`transitionDuration`, `transitionEasing`, `transitionDirection`)
    - overlay parsing supports:
      - `overlayPositionType`
      - `overlayBackgroundInteraction`
      - `overlayRelativePosition`
  - Added duration normalization helper:
    - values `<= 10` interpreted as seconds and converted to ms
    - larger values interpreted as ms
  - Ran:
    - `cargo test -p arthropod --test figma_import_layout -- --nocapture`
  - Result: PASS (8/8)
- REFACTOR / verification:
  - Ran:
    - `cargo fmt --all` -> PASS
    - `cargo test -p arthropod` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (28/28)
    - `cargo clippy -p arthropod --tests` -> PASS (warnings remain in unrelated existing modules)

### 2026-02-27 (import parity slice 1: advanced auto-layout fidelity)

- Goal:
  - Implement production-level Figma auto-layout fidelity beyond direction/gap/padding:
    - alignment (main-axis + cross-axis)
    - wrapping
    - min/max size constraints
    - child align-self override
    - sizing mode handling (`FILL` / `HUG` / `FIXED`)
- RED:
  - Added importer regression in `crates/arthropod/tests/figma_import_layout.rs`:
    - `figma_auto_layout_maps_alignment_wrap_and_size_constraints`
  - Added layout-engine behavior regressions in `crates/layout-engine/tests/flex_tests.rs`:
    - `test_flex_justify_space_between_distributes_children`
    - `test_flex_wrap_moves_children_to_next_line`
  - Ran:
    - `cargo test -p layout-engine --test flex_tests -- --nocapture`
  - Result: FAIL (expected): missing advanced `FlexStyle` fields + missing exported alignment/wrap enums.
- GREEN:
  - Extended `crates/layout-engine/src/lib.rs`:
    - added advanced style enums:
      - `FlexJustifyContent`
      - `FlexAlign`
      - `FlexWrap`
      - `ItemAlignSelf`
    - expanded `FlexStyle` with:
      - `justify_content`, `align_items`, `align_self`, `wrap`
      - `flex_basis`
      - `min_width`, `max_width`, `min_height`, `max_height`
    - updated taffy conversion to map all new fields into:
      - `justify_content`, `align_items`, `align_self`, `flex_wrap`, `flex_basis`
      - `min_size`, `max_size`
  - Extended importer mapping in `crates/arthropod/src/figma.rs`:
    - parsed Figma layout fields:
      - `primaryAxisAlignItems`
      - `counterAxisAlignItems`
      - `layoutWrap`
      - `layoutAlign`
      - `layoutSizingHorizontal`
      - `layoutSizingVertical`
      - `minWidth`, `maxWidth`, `minHeight`, `maxHeight`
    - mapped to advanced `FlexStyle` semantics (including fill/hug/fixed sizing behavior).
  - Updated explicit style initializer in `crates/arthropod-ecs/src/layout_bridge.rs`
    to remain valid with the expanded `FlexStyle`.
  - Ran:
    - `cargo test -p layout-engine --test flex_tests -- --nocapture`
    - `cargo test -p arthropod --test figma_import_layout -- --nocapture`
  - Result: PASS.
- REFACTOR / verification:
  - Ran:
    - `cargo test -p layout-engine` -> PASS
    - `cargo test -p arthropod` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (28/28)
    - `cargo clippy -p layout-engine -p arthropod --tests` -> PASS (warnings remain in unrelated existing modules)

### 2026-02-27 (import parity slice 2: text completeness semantics)

- Goal:
  - Extend text import to include richer Figma typography semantics beyond basic font/align/decoration:
    - text case transforms
    - vertical alignment
    - text auto-resize mode
    - max lines + truncation overflow
    - paragraph spacing + paragraph indent
- RED:
  - Added importer regression in `crates/arthropod/tests/figma_import_layout.rs`:
    - `figma_text_node_maps_advanced_typography_semantics`
  - Ran:
    - `cargo test -p arthropod --test figma_import_layout figma_text_node_maps_advanced_typography_semantics -- --nocapture`
  - Result: FAIL (expected): text semantics were not yet mapped (`"shipped already"` remained untransformed).
- GREEN:
  - Extended text model in `crates/style-engine/src/text.rs`:
    - new enums:
      - `TextCase`
      - `TextAlignVertical`
      - `TextAutoResize`
      - `TextOverflow`
    - new `TextContent` fields:
      - `text_case`
      - `align_vertical`
      - `auto_resize`
      - `max_lines`
      - `overflow`
      - `paragraph_spacing`
      - `paragraph_indent`
    - added builder methods and updated default/serde tests.
  - Updated exports in `crates/style-engine/src/lib.rs` for new text enums.
  - Extended importer parsing in `crates/arthropod/src/figma.rs`:
    - parses Figma text fields:
      - node-level: `textAutoResize`, `maxLines`, `textTruncation`
      - style-level: `textCase`, `textAlignVertical`, `paragraphSpacing`, `paragraphIndent`
    - maps semantics into `TextContent`
    - applies deterministic text-case transform fallback (`UPPER`, `LOWER`, `TITLE`, small-caps fallback to uppercase)
  - Ran:
    - `cargo test -p arthropod --test figma_import_layout figma_text_node_maps_advanced_typography_semantics -- --nocapture`
  - Result: PASS.
- REFACTOR / verification:
  - Ran:
    - `cargo test -p style-engine` -> PASS
    - `cargo test -p arthropod --test figma_import_layout -- --nocapture` -> PASS (10/10)
    - `cargo test -p arthropod` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (28/28)
    - `cargo clippy -p style-engine -p arthropod --tests` -> PASS (warnings remain in unrelated existing modules)

### 2026-02-27 (import parity slice 3: typed instance override resolution)

- Goal:
  - Implement full component-property override resolution for imported instances (not just variant strings):
    - component property definitions
    - typed instance overrides
    - resolved effective instance properties (defaults + overrides)
    - support `INSTANCE_SWAP` node references
- RED:
  - Added importer regressions in `crates/arthropod/tests/figma_import_layout.rs`:
    - `figma_instance_overrides_resolve_with_component_property_defaults`
    - `figma_instance_swap_overrides_resolve_node_references`
  - Ran:
    - `cargo test -p arthropod --test figma_import_layout figma_instance_overrides_resolve_with_component_property_defaults -- --nocapture`
  - Result: FAIL (expected): missing public typed property APIs and missing output maps on `ImportedFigmaDocument`.
- GREEN:
  - Extended importer public model in `crates/arthropod/src/figma.rs`:
    - new public types:
      - `ImportedComponentPropertyType`
      - `ImportedComponentPropertyValue`
      - `ImportedComponentPropertyDefinition`
      - `ImportedComponentPropertyOverride`
    - extended `ImportedFigmaDocument` with:
      - `component_property_definitions`
      - `instance_property_overrides`
      - `resolved_instance_properties`
  - Added Figma parsing support:
    - `componentPropertyDefinitions`
    - typed `componentProperties` values
    - object node refs for instance-swap values (`{ id: ... }`, `nodeId` alias)
  - Implemented canonical-name normalization + typed conversion pipeline for:
    - definition defaults
    - preferred values
    - instance overrides
  - Implemented runtime resolution pass:
    - base from component defaults
    - merge component-level variant fallback
    - apply instance overrides
    - apply instance variant values as final override layer
  - Ran:
    - `cargo test -p arthropod --test figma_import_layout figma_instance_overrides_resolve_with_component_property_defaults -- --nocapture`
    - `cargo test -p arthropod --test figma_import_layout figma_instance_swap_overrides_resolve_node_references -- --nocapture`
  - Result: PASS.
- REFACTOR / verification:
  - Ran:
    - `cargo test -p arthropod --test figma_import_layout -- --nocapture` -> PASS (12/12)
    - `cargo test -p arthropod` -> PASS
    - `cargo test --test figma_json_render_regression -- --nocapture` -> PASS (28/28)
    - `cargo clippy -p arthropod --tests` -> PASS (warnings remain in unrelated existing modules)
