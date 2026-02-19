# 🔭 Vantage: Spec for Data Grid (DataTable)

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
While Arthropod has a basic `Grid` layout (flexbox-based) and a spec for `VirtualList` (1D virtualization), it lacks a high-performance **Data Grid** component.
Enterprise applications (Financial Blotters, Admin Panels, Log Viewers) require displaying dense, tabular data with columns, headers, and sorting capabilities.
Using a naive `Column<Row<Text>>` approach scales poorly (O(N*M) widgets) and fails to meet the performance requirements of "Pro" users.

## 2. User Story
**As a** Quantitative Trader,
**I want to** view a real-time grid of 500+ assets with live price updates, sorting by "% Change" instantly,
**So that** I can identify market opportunities without scrolling through paginated lists or waiting for the UI to unfreeze.

## 3. The "So What?" (Business Value)
*   **Information Density**: Professional users value density over whitespace. A Data Grid allows scanning thousands of data points efficiently.
*   **Performance as a Feature**: In high-frequency environments, a grid that lags during scroll or sort is a broken product.
*   **Standardization**: "The Table" is the most common UI pattern in enterprise software. Providing a robust one out-of-the-box reduces development time for every internal tool.

## 4. Success Metrics
*   **Render Time**: < 16ms (60fps) scrolling performance with 1,000,000 rows (virtualized).
*   **Memory Overhead**: Constant memory usage relative to *viewport size*, not dataset size (O(Viewport)).
*   **Interaction Latency**: Sorting 10,000 rows must complete in < 50ms.
*   **Startup Time**: Grid with 100k rows must appear in < 100ms.

## 5. Gap Analysis

| Feature | `widget-core::Grid` (Current) | `specs/004-virtual-list` (Planned) | `DataGrid` (Target) |
| :--- | :--- | :--- | :--- |
| **Layout** | Flexbox Rows/Cols | Single Column | 2D Grid (Rows x Cols) |
| **Virtualization** | None (O(N)) | Vertical (1D) | Vertical & Horizontal (2D) |
| **Headers** | Manual Widgets | None | Built-in, Sortable, Resizable |
| **Selection** | Manual | Single Item | Cell/Row/Range/Multi |
| **Data Source** | Static Children | Dynamic Builder | Lazy Loading Interface |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 2D Virtualization
*   The system **must** only render cells that intersect the visible viewport.
*   It **must** support datasets where the number of rows exceeds standard texture limits (requiring windowed offsets).

### 6.2 Column Definitions
*   Developers **must** be able to define columns with:
    *   **Header**: Title text or widget.
    *   **Width**: Fixed (`100px`), Percentage (`20%`), or Weighted (`1fr`).
    *   **Cell Renderer**: A configuration that maps data to a widget representation.
    *   **Sortable**: Boolean flag.

### 6.3 Interaction
*   **Sorting**: Clicking a header **must** toggle sort order (Asc -> Desc -> None) and update the view without mutation of the original source if possible.
*   **Selection**: Support for Single Row selection via click.
*   **Keyboard Navigation**: Arrow keys **must** move focus between cells/rows.

### 6.4 Styling
*   **Zebra Striping**: Option to alternate row background colors for readability.
*   **Borders**: Configurable cell borders.
*   **Sticky Header**: The header row **must** remain visible while scrolling vertically.

## 7. Constraint Requirements
*   **Main Thread Blocking**: Updates to the grid (e.g., sorting) **must not** block the main thread for more than 16ms. Heavy operations should yield or run in background tasks.
*   **Accessibility**: The grid **must** be navigable via keyboard and expose correct ARIA-like roles (grid, row, gridcell) to screen readers.
*   **Responsiveness**: The grid **must** handle window resizing without layout thrashing.

## 8. Out of Scope (Phase 1)
*   **Cell Editing**: Inline text inputs or dropdowns (Complex state management).
*   **Column Reordering**: Drag-and-drop headers.
*   **Column Resizing**: Dragging splitters.
*   **Pinned Columns**: Freezing the first N columns.
*   **Tree Grid**: Hierarchical data (Filesystem view).
