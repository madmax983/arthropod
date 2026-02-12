# Phase 4 Visual Regression Tests (Playwright)

This harness captures deterministic browser screenshots for the Phase 4 effect fixtures:

- `blur`
- `blend`
- `clipping`

It runs against the wasm example `phase4_visual_web` and compares `#arthropod-canvas` output against committed baselines.

Configured browser projects:

- Chromium (`Desktop Chrome`)
- Firefox (`Desktop Firefox`)

## Setup

```bash
npm install
npx playwright install chromium firefox
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
npm run visual:test:firefox
```

## Fixture App

The test fixture app is:

`examples/phase4_visual_web.rs`

Cases are selected with query params:

- `/?case=blur`
- `/?case=blend`
- `/?case=clipping`

On first successful frame, the app sets:

- `body[data-arthropod-ready="1"]`
- `body[data-arthropod-case="<case>"]`

Playwright waits for these markers before snapshot capture.

If WebGPU is unavailable for a browser/runtime, the fixture sets:

- `body[data-arthropod-ready="0"]`
- `body[data-arthropod-error="<message>"]`

The Playwright test skips that browser project only for explicit WebGPU-unavailable startup errors, and fails for all other startup errors.
