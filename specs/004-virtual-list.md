# 🔭 Vantage: Spec for Virtual List & Infinite Scroll

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
The current `List` implementation in `widget-core` renders every child widget upfront.
This scales linearly O(N) with the number of items. For large datasets (logs, chat history, data tables) with 10k+ items, this causes massive memory usage and blocks the UI thread during layout/render.
Enterprise applications require displaying large amounts of data without performance degradation.

## 2. User Story
**As a** Data Analyst,
**I want to** scroll through 100,000 log entries smoothly,
**So that** I can identify errors and patterns without the application freezing or crashing.

## 3. The "So What?" (Business Value)
*   **Scalability**: Enables the framework to power data-intensive applications (Dashboards, IDEs).
*   **Performance**: Fixed cost O(1) rendering regardless of dataset size.
*   **User Experience**: Immediate feedback and smooth scrolling (60fps) builds trust.

## 4. Success Metrics
*   **Initial Load**: < 16ms for 100,000 items (time to first frame).
*   **Memory Overhead**: Constant memory usage relative to *viewport size*, not total item count.
*   **Frame Rate**: Maintain 60fps while scrolling through complex items.

## 5. Acceptance Criteria (MVP -> Production)

### 5.1 Viewport Culling (Windowing)
*   The system **must** only create and render widgets that are currently intersecting the scroll viewport (plus a small buffer).
*   Off-screen items **must not** exist in the Scene Graph or Layout Tree.

### 5.2 Scroll Container
*   The Virtual List **must** be contained within a scrollable area that handles clipping (`overflow: hidden`).
*   The container **must** simulate the total height of the list using a spacer or absolute positioning, so the scrollbar reflects the true size.

### 5.3 Dynamic Item Building
*   The API **must** support a lazy builder pattern (e.g., `item_builder(index) -> Widget`) rather than taking a pre-allocated `Vec<Widget>`.
*   Items **must** be created on-demand as they scroll into view and destroyed (or recycled) when they leave.

### 5.4 Fixed Height Support
*   For MVP, the system **must** support items with a fixed, known height (passed as a parameter).
*   This allows O(1) calculation of visible indices: `start_index = floor(scroll_y / item_height)`.

## 6. Technical Notes (For Engineering Context)
*   **Architecture**: Implements the "Virtual Window" pattern.
*   **Layout**: `LayoutEngine` needs to handle the "phantom" height.
*   **Recycling**: Consider a `WidgetPool` to reuse widget instances instead of dropping/recreating them to reduce allocator pressure (Phase 2 optimization).

## 7. Out of Scope (Phase 1)
*   **Variable Heights**: Items with unknown/different heights (requires a measurement pass or estimation).
*   **Grid Virtualization**: 2D scrolling for spreadsheets.
*   **Sticky Headers**: Section headers that stick to the top.
