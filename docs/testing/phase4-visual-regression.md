# Phase 4 Visual Regression Tests (Playwright)

This harness captures deterministic browser screenshots for the Phase 4 effect fixtures:

- `blur`
- `blend`
- `clipping`
- `mask`
- `image`

It runs against the wasm example `phase4_visual_web` and compares `#arthropod-canvas` output against committed baselines.

Configured browser projects:

- Chromium (`Desktop Chrome`)
- Firefox (`Desktop Firefox`)
- WebKit (`Desktop Safari`)

## Setup

```bash
npm install
npx playwright install chromium firefox webkit
```

## Update Baselines

```bash
npm run visual:update
```

This writes snapshots under:

`tests/visual/phase4-visual.spec.mjs-snapshots/`

## Run Regression Check

```bash
npm run visual:test
```

Run browser-specific checks:

```bash
npm run visual:test:chromium
npm run visual:test:chromium:headed
npm run visual:test:chromium:headed:adapter
npm run visual:test:webgpu:headed
npm run visual:test:firefox
npm run visual:test:webkit
```

Capture/update Chromium WebGPU baselines with a real adapter:

```bash
npm run visual:update:chromium:headed
```

## Fixture App

The test fixture app is:

`examples/phase4_visual_web.rs`

Cases are selected with query params:

- `/?case=blur`
- `/?case=blend`
- `/?case=clipping`
- `/?case=mask`
- `/?case=image`

On first successful frame, the app sets:

- `body[data-arthropod-ready="1"]`
- `body[data-arthropod-case="<case>"]`

Playwright waits for these markers before snapshot capture.

If WebGPU is unavailable for a browser/runtime, the fixture sets:

- `body[data-arthropod-ready="0"]`
- `body[data-arthropod-error="<message>"]`

The Playwright test skips that browser project when adapter preflight fails or when startup reports explicit WebGPU-unavailable errors, and fails for all other startup errors.

Notes:

- Playwright performs a `navigator.gpu.requestAdapter()` preflight per test and skips immediately when no adapter is available in that browser/runtime.
- In headless Chromium, `navigator.gpu` may exist while `requestAdapter()` still returns `null`; use the headed Chromium command for true WebGPU capture.
- If wasm panics during startup/render, the fixture panic hook now marks `data-arthropod-ready="0"` so tests skip/fail deterministically instead of timing out.
- The workflow `.github/workflows/webgpu-headed-visual.yml` runs a dedicated headed Chromium WebGPU adapter gate plus headed phase4 visual regression on `workflow_dispatch` and nightly schedule.
