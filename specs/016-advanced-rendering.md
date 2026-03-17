# 🔭 Vantage: Spec for Advanced Rendering (Shadows and Blur)

**Status:** Draft
**Owner:** Vantage (Product)
**Date:** 2026-05-24

## 1. Problem Statement
Modern enterprise applications require visual depth and hierarchy to guide user focus, such as drop shadows for modals and background blur (glassmorphism) for overlays. Currently, Arthropod's rendering pipeline only supports basic fills and strokes, making UIs look flat and outdated compared to native platform expectations.

## 2. User Story
**As a** UI/UX Designer,
**I want** to apply drop shadows, inner shadows, and background blur filters to UI elements,
**So that** I can create visual hierarchy, indicate depth, and match modern native design languages (like macOS Acrylic or Windows Mica).

## 3. So What? (Business Value)
*   **Aesthetics & Perception:** Flat, unstyled UIs are often perceived by users as "cheap" or incomplete. Modern styling directly impacts user trust and perceived application quality.
*   **Usability:** Shadows help delineate overlapping content (e.g., separating a dropdown menu from the content below it), reducing cognitive load.
*   **Competitive Parity:** Frameworks like Electron, Flutter, and native toolkits offer these effects out of the box. Without them, Arthropod cannot compete for premium enterprise tooling.

## 4. Metric Definition
*   **Performance:** Applying a standard drop shadow or blur effect to a single floating widget (like a modal) must not increase frame render time by more than 2ms.
*   **Adoption:** Within 1 month of release, at least 50% of our example applications and internal tools should utilize the new rendering effects.

## 5. Gap Analysis
*   **Current State:** `render-engine` draws flat rectangles and paths.
*   **Competitors:** Web (CSS `box-shadow`, `backdrop-filter`), macOS (NSVisualEffectView), Windows (Acrylic).
*   **The Gap:** We lack a multi-pass rendering pipeline or shader support required to compute blur convolutions and soft shadows efficiently on the GPU.

## 6. Acceptance Criteria
- [ ] **Drop Shadows:** Users can define one or multiple drop shadows on a widget with X/Y offsets, blur radius, spread radius, and color.
- [ ] **Inner Shadows:** Users can define shadows that render *inside* the bounds of a widget.
- [ ] **Background Blur (Backdrop Filter):** Users can define a blur radius that applies to all content rendered *behind* the widget, creating a frosted glass effect.
- [ ] **Clipping:** Shadows must respect the border radius and clipping bounds of their parent widgets.
- [ ] **Performance:** The implementation must utilize GPU acceleration (e.g., Gaussian blur shaders) to maintain 60FPS target on standard hardware.

## 7. Out of Scope (Phase 1)
- 3D lighting or ray-traced shadows.
- Complex CSS filters (e.g., sepia, hue-rotate, contrast).
- Software/CPU fallback rendering for systems without GPU support.
