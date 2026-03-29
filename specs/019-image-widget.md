# 🔭 Vantage: Spec for Image Widget & Asset Pipeline

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
Currently, Arthropod's rendering engine has partial support for image paint types, but there is no high-level `Image` widget in `widget-core`. UI developers need a standard way to display avatars, product photos, and background images without writing custom rendering nodes or dealing with raw textures.

## 2. User Story
**As a** UI Developer,
**I want to** use a standard `<Image>` widget that supports scaling modes (`cover`, `contain`) and corner radii,
**So that** I can easily display avatars and product photos that fit their containers perfectly.

## 3. The "So What?" (Business Value)
*   **Developer Ergonomics**: Images are fundamental to almost every UI. Forcing developers to handle texture decoding and UV mapping manually severely degrades the developer experience.
*   **Visual Quality**: Correctly scaling and clipping images (especially avatars with rounded corners) is a baseline expectation for modern applications.

## 4. Success Metrics
*   **Render Correctness**: `cover` and `contain` modes must perfectly match standard CSS behavior.
*   **Performance**: Decoding a 4K JPEG must not block the main UI thread.

## 5. Gap Analysis

| Feature | Current State | Target State |
| :--- | :--- | :--- |
| **Widget API** | None | `image!(src)` macro and `Image` struct |
| **Scale Modes** | Partial (Engine) | `Fill`, `Contain`, `Cover`, `None`, `ScaleDown` |
| **Corner Clipping**| Solid colors only | Images respect `border_radius` |
| **Async Loading** | None | Background decoding via `AssetServer` |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 Widget API
*   Developers **must** be able to instantiate an image using a local file path or raw bytes.

### 6.2 Scaling & Alignment
*   The widget **must** support `ObjectFit` modes: `Fill`, `Contain`, `Cover`, `None`, `ScaleDown`.
*   The widget **must** support `ObjectPosition` (e.g., centering the crop).

### 6.3 Clipping
*   The image **must** respect layout constraints and corner clipping (e.g., `border_radius` on the parent container or the image itself).

### 6.4 Asset Pipeline
*   Image decoding (PNG, JPG) **must** occur asynchronously off the main thread to prevent UI freezing.

## 7. Constraint Requirements
*   **Memory Management**: Large images must be uploaded to the GPU as textures efficiently, and released when the widget is unmounted.

## 8. Out of Scope (Phase 1)
*   **Network Loading**: Fetching images from HTTP URLs (Phase 2).
*   **Caching Layer**: Advanced LRU disk/memory caching.
*   **SVG Support**: Vector graphics rendering.