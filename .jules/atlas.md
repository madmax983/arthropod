## 2026-05-25 - Break Circular Dependency in Render Engine
**Tangle:** `render_engine::backend::text` depended on `render_engine::backend::wgpu::PrimitiveInstance`, while `render_engine::backend::wgpu` depended on `render_engine::backend::text::TextRenderer`. This created a `wgpu -> text -> wgpu` circular dependency, coupling the text backend to wgpu specifics.
**Blueprint:** Extracted `PrimitiveInstance` (and related POD types/constants) into a new backend-agnostic module `render_engine::primitives`. Both `text` and `wgpu` backends now depend on `primitives`, breaking the cycle. Updated all internal and external references to use the new canonical location (or re-exports).

## 2026-06-01 - Input Engine Unification
**Tangle:** 'Input State' (TextInputState, FormState) and 'Input Logic' (Focus, Validation) were deeply embedded in 'widget-core', preventing 'input-engine' from fulfilling its role as the Input Fusion layer.
**Blueprint:** Moved `TextInputState`, `FormState`, `Validator`, `ValidationState` and associated logic from `widget-core` to `input-engine`. Established `widget-core` as a consumer of `input-engine`, using facade re-exports to maintain backward compatibility.
