# 🔭 Vantage: Spec for GPU Text Rendering Pipeline

**Status**: Draft
**Owner**: Vantage (Product Manager)
**Target Release**: Arthropod Core (Render Engine Integration)

## 1. 👤 User Story

**As a** UI Developer or Application User,
**I want** text to render crisply, beautifully, and instantaneously at any scale or color,
**So that** my applications feel responsive, look professional, and are readable on modern high-DPI displays without perceptible lag or blurry artifacts.

## 2. 🧐 The "So What?" (Business Value)

Text is the primary medium of information density in any GUI. If our text rendering is slow, the entire application feels sluggish. If it's blurry, it looks amateurish and causes eye strain.

Currently, we have basic text shaping and layout, but we lack a unified, high-performance way to get those shaped glyphs onto the screen using the GPU. If we rely on CPU rasterization for every frame or naively render each character as a separate draw call, we will destroy our frame budget and drain laptop batteries.

**Utility is Revenue**:
A high-performance GPU text pipeline ensures that:
- Complex dashboards with thousands of text elements run at a buttery smooth 60+ FPS.
- Developers can freely use typographic features (like gradient text) without worrying about hidden performance cliffs.
- The framework feels lightweight and premium, driving adoption.

## 3. 🔍 Gap Analysis

| Feature | Current State | Market Standard (e.g., WebGL/Skia) | Target State (GPU Text Pipeline) |
| :--- | :--- | :--- | :--- |
| **Rendering Method** | CPU-bound or fragmented | GPU-accelerated Atlas or SDF | Unified GPU Atlas Pipeline |
| **Performance (1k chars)** | Missing / CPU heavy | < 1ms | < 100μs overhead per frame |
| **Visual Quality** | Basic solid color | Sub-pixel AA, Gradients | High DPI crispness, gradient support |
| **Memory Footprint** | Unbounded | Dynamic caching | LRU or hash-based glyph caching |
| **Integration** | Ad-hoc text nodes | First-class primitives | Unified with `VisualStyle` primitives |

## 4. 📝 Solution Overview

We need a robust **GPU Text Rendering Pipeline** integrated deeply into our `render-engine`.

### Key Capabilities

1.  **Dynamic Glyph Atlas**:
    -   A texture map living on the GPU that acts as a cache for rasterized glyphs.
    -   When new text appears, missing glyphs are rasterized and packed into the atlas.
    -   Frequently used glyphs are drawn directly from the atlas with zero CPU overhead.

2.  **Unified Instanced Rendering**:
    -   Text should not be a special "Text Node" that requires a completely separate rendering path.
    -   Instead, text should be treated as a collection of glyph instances that flow through our existing high-performance primitive pipeline.

3.  **Rich Styling Support**:
    -   The pipeline must support rendering text with solid colors, but also seamlessly support gradient fills (linear, radial) without requiring expensive render-to-texture workarounds.

4.  **High-DPI Awareness**:
    -   The rasterization and atlas must account for the display's scale factor to ensure crisp edges on Retina/4K displays.

## 5. 📊 Metrics (Success Definition)

-   **Performance**: Rendering 1,000 glyphs on screen must add **< 10μs** of overhead to the GPU dispatch time.
-   **Atlas Efficiency**: Cache hit retrieval must be **O(1)**. First-time rasterization penalty should be minimized.
-   **Visual Correctness**: 0 pixel deviation on baseline alignment for standard system fonts across supported platforms.
-   **Resource Usage**: The glyph atlas texture should not exceed a reasonable default size (e.g., 1024x1024) for standard UI workloads, with graceful handling if the atlas fills up.

## 6. ✅ Acceptance Criteria

### Must Have (Phase 1)
-   [ ] **Atlas Generation**: A system to rasterize glyphs and pack them into a GPU texture.
-   [ ] **Cache Management**: Hash-based or similar caching to prevent re-rasterizing known glyphs at the same font size.
-   [ ] **Instanced Dispatch**: Text is drawn using instanced rendering (1 draw call for many glyphs of the same font/texture).
-   [ ] **Styling Integration**: Text inherits and respects the `VisualStyle` colors/gradients defined by the layout system.
-   [ ] **Unified Pipeline**: Text rendering is integrated into the core primitive pipeline, rather than existing as a standalone, incompatible render pass.

### Should Have (Phase 2)
-   [ ] **Sub-pixel Antialiasing (LCD)**: For non-high-DPI screens.
-   [ ] **Dynamic Atlas Resizing**: If the atlas gets full, it gracefully handles eviction or resizing.

### Could Have (Future)
-   [ ] **Signed Distance Fields (SDF)**: For infinite scaling and text outlines/glows without re-rasterizing.

## 7. 🚫 Out of Scope

-   **Text Layout & Shaping**: This specification assumes the text is already shaped and measured by the `text-engine`. This spec is *only* about putting those shaped glyphs on the screen.
-   **Text Input & Editing**: Handled by the `input-engine` and widget specifications.
-   **Complex Text Decorations**: Underlines, strikethroughs (these should be handled by standard primitive rects, not the glyph pipeline).