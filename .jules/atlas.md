## 2024-05-23 - Refactor WgpuBackend
**Tangle:** `WgpuBackend` was a "Blob" struct (1200+ lines) handling WGPU initialization, Rect/Glyph pipeline creation, buffer management, and rendering logic all in one file (`wgpu_backend.rs`). This violated SRP and made the code hard to navigate and test.
**Blueprint:** Split `WgpuBackend` into modular components: `WgpuContext` (device/surface management), `RectPipeline` (rect rendering), and `GlyphPipeline` (text rendering). Each component is now in its own file under `backend/wgpu/`, coordinated by a thinner `WgpuBackend` in `backend/wgpu/mod.rs`. This improves cohesion and makes each part independently testable.

## 2026-01-31 - Decompose WidgetContext
**Tangle:** `WidgetContext` in `widget-core` was a "Blob" (800+ lines) managing Layout, Input, Interaction, Forms, and Decoration logic, violating Single Responsibility Principle.
**Blueprint:** Extracted `LayoutContext`, `InteractionContext`, and `DecorationContext` into separate modules. `WidgetContext` now acts as a cohesive Facade, composing these sub-contexts to maintain API compatibility while delegating implementation details.

## 2026-02-01 - Decompose MCP Test Tools
**Tangle:** `crates/arthropod-mcp/src/tools/test.rs` was a "Blob" (1400+ lines) mixing scene creation, state assertion, render verification, and reactivity setup.
**Blueprint:** Split into `create_scene.rs`, `assert_state.rs`, `verify_render.rs`, and `setup_reactive.rs` under `crates/arthropod-mcp/src/tools/test/`. This improves cohesion and test organization.
