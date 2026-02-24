# 🔭 Vantage: Spec for Virtual Scroll Widget

**Status:** Draft
**Owner:** Vantage (Product Manager)
**Date:** 2024-05-21

## 1. The "User Story" 👤

> **"As a Data Analyst using an Arthropod-based dashboard, I want to scroll through 50,000 transaction records smoothly, so that I can identify anomalies without the application freezing or crashing."**

## 2. The "So What?" (Business Value) 💰

**Problem:**
The current `List` implementation in `widget-core` eagerly renders all children. For a list of 10,000 items, this means creating 10,000 widget instances, 10,000 scene nodes, and calculating layout for all of them *every frame*. This leads to:
- **High Memory Usage:** Linear scaling O(N) with dataset size.
- **Slow Startup:** Initial render blocks the main thread for seconds.
- **Poor UX:** Scrolling becomes jerky (< 30fps) as the layout engine struggles.

**Solution:**
A `VirtualScroll` widget that only renders the items currently visible in the viewport (plus a small buffer). This makes rendering cost O(1) relative to the total dataset size.

**Impact:**
- Enables enterprise-grade data grids and logs.
- Reduces memory footprint significantly for large lists.
- Ensures 60fps performance regardless of list length.

## 3. Gap Analysis 🔍

| Feature | React (`react-window`) | Solid (`solid-virtual`) | Arthropod (`List`) | Target (`VirtualScroll`) |
| :--- | :--- | :--- | :--- | :--- |
| **Rendering Strategy** | Windowing (Virtual) | Windowing (Virtual) | Eager (Render All) | **Windowing (Virtual)** |
| **Startup Time (10k items)** | < 10ms | < 5ms | > 200ms (Est.) | **< 16ms** |
| **Memory Usage** | O(Visible) | O(Visible) | O(Total) | **O(Visible)** |
| **Dynamic Updates** | Yes | Yes (Signals) | Yes (Rebuild All) | **Yes (Surgical)** |

## 4. Acceptance Criteria ✅

To consider this feature "Done", the Engineering team must demonstrate:

1.  **Performance:**
    -   Render a list of **100,000 items** with an initial render time of **< 16ms**.
    -   Maintain **60fps** while scrolling through the list on reference hardware.
    -   Memory usage must not grow linearly with the number of items.

2.  **Functionality:**
    -   **Fixed Height Support:** Must support items with a fixed height (e.g., `item_height: 30.0`).
    -   **Dynamic Data:** Must accept a reactive signal (e.g., `ReadSignal<Vec<T>>`) as the data source. When the signal updates, the list updates efficiently.
    -   **Scrolling:** Must provide a scrollbar or integrate with a parent scroll container.

3.  **DX (Developer Experience):**
    -   API should be declarative and simple (similar to `List`).
    -   Must define an `item_renderer` closure that takes a data item and returns a Widget.

## 5. Out of Scope (Phase 1) 🚫

-   **Variable Row Height:** Phase 2. (Requires measurement and caching).
-   **Grid Virtualization:** Phase 2. (2D scrolling).
-   **Sticky Headers:** Phase 2.
-   **Animated Insertions/Deletions:** Phase 2.

## 6. Metrics for Success 📊

-   **P99 Render Time:** < 16ms for 10k items.
-   **Memory Overhead:** < 50MB for 100k simple text items.
