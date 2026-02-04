## [Reduction]
**Bloat:** `DecorationContext`, `LayoutContext`, `InteractionContext`, `InputContext`, `FormContext`
**Cut:** Flattened all into `WidgetContext`
**Saved:** 5 files, ~200 lines of boilerplate delegation code

## [Reduction]
**Bloat:** `AppBuilder` (Builder pattern for object taking 2 arguments)
**Cut:** Replaced with `App::new_windowed` and `App::new_headless`
**Saved:** 1 file (`builder.rs`), ~80 lines, reduced cognitive load by removing "Factory Factory"

## [Reduction]
**Bloat:** `theme_engine::BackgroundMaterial` mirroring `plat_core::BackdropMaterial` via `material_bridge.rs`
**Cut:** Removed duplicate types and bridge layer, used `BackdropMaterial` directly.
**Saved:** 1 file (`material_bridge.rs`), ~100 lines of duplicate type definitions and mapping code.
