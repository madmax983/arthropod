# Phase 1/2 Catch-Up Status (Code vs Plan)

This note records where implementation was brought in line with
`docs/plans/2026-02-08-figma-rendering-pipeline-design.md`, and where we are
intentionally deviating for now.

## Implemented in this pass

- ECS render collection now uses the same styled-instance fallback generator as
  backend test collection:
  - `create_node_instances()` now calls `create_primitive_instances(...)`.
  - This enables ECS fallback support for:
    - gradient fills (representative color fallback),
    - stroke instances,
    - drop shadow instances.
  - Text remains intentionally skipped in ECS fallback, since shaping and glyph
    atlas upload are backend-owned operations.

- `VisualStyle` now includes Phase 2 model fields:
  - `corner_smoothing: f32`
  - `clips_content: bool`
  - `fill_geometry: Option<Vec<VectorPath>>`
  - `stroke_geometry: Option<Vec<VectorPath>>`
  - Existing `path(...)` builder now maps to `fill_geometry` for compatibility.

- `StrokeStyle` now uses Figma-like stacked stroke paints:
  - `paints: Vec<Paint>`
  - `solid(...)` initializes the first paint layer
  - `top_paint()` is used by renderer paths

- Primitive flags now follow the planned packed layout for fill/stroke/blend/cap/join/glyph:
  - fill type bits are distinct per gradient family
  - blend mode bits are packed from `BlendMode`
  - stroke cap/join bits are packed from `StrokeStyle`
  - drop-shadow marker moved to an internal high bit to avoid overlap with blend bits

- Gradient text bounds transport no longer overloads `corner_radii`:
  - glyph text bounds are carried in `gradient_params.yzw + stroke_params.x`
  - shader uses those fields for text-space UV mapping

## Intentional deviations (documented)

None currently tracked in this catch-up note.

## Follow-up tasks

- Continue parity hardening from the master render plan
  (`docs/plans/2026-02-08-figma-rendering-pipeline-design.md`).

