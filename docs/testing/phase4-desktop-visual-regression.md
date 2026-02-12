# Phase 4 Desktop Visual Regression

This suite captures the native desktop renderer output for Phase 4 reference scenes and compares it against committed golden PNGs.

## Coverage

- `blur`
- `blend`
- `clipping`
- `mask`
- `image`

Scene definitions are shared from `examples/phase4_visual_scenes.rs`.

## Golden Files

Committed baselines:

- `tests/visual/golden/phase4/blur.png`
- `tests/visual/golden/phase4/blend.png`
- `tests/visual/golden/phase4/clipping.png`
- `tests/visual/golden/phase4/mask.png`
- `tests/visual/golden/phase4/image.png`

Debug artifacts from test runs are written to:

- `tests/visual/artifacts/phase4/` (gitignored)

## Commands

Generate/update goldens:

```bash
cargo run --example capture_phase4_visuals
```

Write to a custom directory:

```bash
cargo run --example capture_phase4_visuals -- --out path/to/output
```

Run regression test:

```bash
cargo test --test phase4_desktop_visual_regression -- --nocapture
```

## Diff Rules

The test uses per-channel tolerance and a max differing-pixel ratio:

- per-channel tolerance: `2`
- max differing pixel ratio: `0.02`

This is strict enough to catch meaningful rendering regressions while avoiding flaky failures from tiny raster differences.

You can override thresholds via env vars:

- `ARTHROPOD_VISUAL_CHANNEL_TOLERANCE` (u8)
- `ARTHROPOD_VISUAL_MAX_DIFF_RATIO` (f32)

Example:

```bash
ARTHROPOD_VISUAL_MAX_DIFF_RATIO=0.04 cargo test --test phase4_desktop_visual_regression -- --nocapture
```

## CI Behavior

Windows CI runs this regression test with a slightly relaxed diff ratio for cross-run stability.
If the test fails, CI uploads `tests/visual/artifacts/phase4` as an artifact (`phase4-desktop-visual-artifacts`) so PR review can inspect actual render output quickly.
