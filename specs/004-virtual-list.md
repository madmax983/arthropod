# 🔭 Vantage: Spec for Virtual List & Infinite Scroll

**Status**: Approved (Draft Rev 2)
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
The current `List` implementation in `widget-core` (specifically `crates/widget-core/src/list.rs`) renders every child widget upfront.
This scales linearly **O(N)** with the number of items. For large datasets (logs, chat history, data tables) with 10k+ items, this causes massive memory usage and blocks the UI thread during layout/render as the entire widget tree is built and traversed.
Enterprise applications require displaying large amounts of data without performance degradation.

## 2. User Story
**As a** Data Analyst,
**I want to** scroll through 100,000 log entries smoothly,
**So that** I can identify errors and patterns without the application freezing or crashing.

**As a** Chat Application User,
**I want to** scroll back through years of message history,
**So that** I can find a specific conversation without waiting for "load more" buttons.

## 3. The "So What?" (Business Value)
*   **Scalability**: Enables the framework to power data-intensive applications (Dashboards, IDEs).
*   **Performance**: Fixed cost **O(1)** rendering regardless of dataset size.
*   **User Experience**: Immediate feedback and smooth scrolling (60fps) builds trust.

## 4. Success Metrics
*   **Initial Load**: < 16ms for 100,000 items (time to first frame).
*   **Memory Overhead**: Constant memory usage relative to *viewport size*, not total item count.
*   **Frame Rate**: Maintain 60fps while scrolling through complex items.
*   **Input Latency**: < 50ms from scroll event to render update.

## 5. Acceptance Criteria (MVP -> Production)

### 5.1 Viewport Culling (Windowing)
*   The system **must** only create and render widgets that are currently intersecting the scroll viewport (plus a small buffer/overscan area).
*   Off-screen items **must not** exist in the Scene Graph or Layout Tree.

### 5.2 Scroll Container & layout
*   The Virtual List **must** be contained within a scrollable area that handles clipping (`overflow: hidden`).
*   The container **must** simulate the total height of the list using a spacer or absolute positioning, so the scrollbar reflects the true size.

### 5.3 Dynamic Item Building
*   The API **must** support a lazy builder pattern (e.g., `item_builder(index) -> Widget`) rather than taking a pre-allocated `Vec<Widget>`.
*   Items **must** be created on-demand as they scroll into view and destroyed (or recycled) when they leave.

### 5.4 Fixed Height Support
*   For MVP, the system **must** support items with a fixed, known height (passed as a parameter).
*   This allows O(1) calculation of visible indices: `start_index = floor(scroll_y / item_height)`.

### 5.5 Keyboard Navigation
*   The list **must** support keyboard navigation when focused.
*   **Up/Down Arrow**: Move selection up/down one item. Scroll if necessary.
*   **PageUp/PageDown**: Move selection by one viewport height.
*   **Home/End**: Jump to the first/last item.

### 5.6 Accessibility (A11y)
*   The list **must** report the total number of items to assistive technologies (e.g., via `aria-setsize` equivalent).
*   The currently focused item **must** report its index (e.g., via `aria-posinset` equivalent).
*   Screen readers **must** be able to traverse the list as if all items were present.

### 5.7 Dynamic Updates
*   The list **must** handle insertions and deletions in the underlying data source without losing scroll position (unless the removed item was above the viewport).
*   Updates to the data count **must** trigger a re-layout of the scrollbar thumb.

## 6. API Design (Draft)

```rust
// Proposed usage
VirtualList::new()
    .count(100_000)
    .item_height(50.0)
    .builder(|index| {
        // Return a widget for this index
        Text::new(format!("Log entry #{}", index))
    })
    .on_reach_end(|| {
        // Infinite scroll callback
        fetch_more_data();
    })
```

## 7. Infinite Scroll & States

### 7.1 Infinite Scroll
*   The list **must** accept an `on_reach_end` callback (or signal) that triggers when the user scrolls near the bottom (e.g., within 200px or 5 items).
*   This allows for "Infinite Scroll" patterns where data is fetched lazily.

### 7.2 Loading State
*   While waiting for data (e.g., during infinite scroll fetch), the list **should** optionally display a loading indicator at the bottom.

### 7.3 Empty State
*   If `count` is 0, the list **should** optionally display an "Empty State" widget (e.g., "No items found").

## 8. Technical Notes (For Engineering Context)
*   **Architecture**: Implements the "Virtual Window" pattern.
*   **Layout**: `LayoutEngine` needs to handle the "phantom" height.
*   **Recycling**: Consider a `WidgetPool` to reuse widget instances instead of dropping/recreating them to reduce allocator pressure (Phase 2 optimization).
*   **State Management**: Scroll position must be preserved across re-renders unless explicitly reset.

## 9. Out of Scope (Phase 1)
*   **Variable Heights**: Items with unknown/different heights (requires a measurement pass or estimation).
*   **Grid Virtualization**: 2D scrolling for spreadsheets.
*   **Sticky Headers**: Section headers that stick to the top.
*   **Drag & Drop**: Reordering items via drag and drop.
