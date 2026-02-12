# Phase 4 Visual Regression Tests (Playwright)

This harness captures deterministic browser screenshots for the Phase 4 effect fixtures:

- `blur`
- `blend`
- `clipping`

It runs against the wasm example `phase4_visual_web` and compares `#arthropod-canvas` output against committed baselines.

## Setup

```bash
npm install
npx playwright install chromium
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

