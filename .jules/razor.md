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

## [Reduction]
**Bloat:** `AdaptiveThresholds` in `arthropod-ecs` (Speculative Generality)
**Cut:** Replaced dynamic runtime metrics and sliding window with a simple static constant.
**Saved:** 1 file (`adaptive.rs`), ~250 lines, removed VecDeque overhead and per-frame calculations.

## [Reduction]
**Bloat:** `MainThreadSignal<T>` in `arthropod-ecs` (Redundant Wrapper)
**Cut:** Removed wrapper struct and used `ReadSignal<T>` directly (which is already thread-safe).
**Saved:** ~30 lines of boilerplate, removed unnecessary indirection (`.inner()`) and cognitive overhead.

## [Reduction]
**Bloat:** Nested `if let` and `if` statements checking `Option` types
**Cut:** Collapsed nested checks using `.filter()` (e.g., `if let Some(x) = y.filter(|v| condition)`)
**Saved:** ~6 lines across `figma_codegen.rs` and `figma_pull.rs`, resolving `clippy::collapsible_if` warnings.

## [Reduction]
**Bloat:** Passing `&PathBuf` as function arguments
**Cut:** Replaced with `&Path` slice references
**Saved:** Reduced unnecessary allocations and pointer indirection in `figma_pull.rs`, resolving `clippy::ptr_arg` warnings.

## [Reduction]
**Bloat:** Nested `if let` and `if` checking options in spatial graph
**Cut:** Collapsed nested checks using chained `.filter()` calls.
**Saved:** Reduced nesting depth and cognitive load, removed `clippy::collapsible_if` warnings in `spatial-graph/systems.rs`.

## [Reduction]
**Bloat:** `WidgetExt` trait mirroring `widget_core::WidgetBoxed`
**Cut:** Removed duplicate trait definition and its blanket impl, replaced with standard `WidgetBoxed`
**Saved:** ~30 lines of duplicate boilerplate, removed a single-use trait.

## [Reduction]
**Bloat:** Single-implementation traits `RenderBackend` and `A11yBridge`
**Cut:** Removed the traits and moved their methods directly to the concrete implementations (`WgpuBackend` and `AccessKitBridge`), following YAGNI and the "One-Time Trait" principle.
**Saved:** 2 trait definition files simplified, removed 25+ lines of unnecessary abstraction, simplified usage in ~15 other files.
**Bloat:** `HasBackdropMaterial` trait
**Cut:** Replaced with concrete methods on `Window` and `WindowImpl`
**Saved:** 1 trait definition and several unused imports
