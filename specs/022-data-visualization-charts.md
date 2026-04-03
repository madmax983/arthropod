# 🔭 Vantage: Spec for Data Visualization Engine (Charts & Graphs)

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
Enterprise and B2B applications are fundamentally about making sense of data. While we have specified a Data Grid (`006-data-grid.md`) for tabular inspection, humans process visual trends much faster than numbers.
Currently, Arthropod has no native charting or data visualization capabilities. Developers needing to render a simple line chart for "Revenue over Time" must drop down to raw `wgpu` or `Canvas` APIs, essentially writing a chart library from scratch.
This leads to:
1.  **Blocker for Enterprise Adoption**: Dashboards are a non-negotiable requirement for analytics tools, CRMs, and financial software.
2.  **Performance Pitfalls**: Naive implementations of charts (e.g., creating a SceneNode for every single data point) will cripple the render engine for large datasets.
3.  **Inconsistent Visuals**: Third-party integrations will clash with Arthropod's declarative API and reactive state system.

## 2. User Story
**As a** Business Analyst / End User,
**I want to** see a high-performance, interactive line chart of our server metrics over the last 30 days,
**So that** I can instantly spot anomalies without reading thousands of rows of logs.

**As a** Framework Developer,
**I want to** bind a reactive `Signal<Vec<DataPoint>>` to a `LineChart` widget,
**So that** the chart updates automatically and efficiently when new data arrives, without writing low-level drawing code.

## 3. The "So What?" (Business Value)
*   **Unlocks Core Markets**: Without data visualization, Arthropod cannot be used for analytics dashboards, trading platforms, or monitoring tools.
*   **Showcase Performance**: Charting is the perfect testbed for our GPU-accelerated rendering. Rendering 100,000 data points smoothly proves our performance claims.
*   **Developer Velocity**: A declarative charting API turns a month-long bespoke rendering task into a 5-minute integration.

## 4. Success Metrics
*   **Performance**: Must render and update a line chart with 100,000 data points at 60 FPS (<16ms frame time).
*   **Reactivity**: Modifying the underlying `Signal` data must update the chart without a full re-render of the surrounding layout.
*   **API Ergonomics**: Creating a basic chart must require no more than 10 lines of code.

## 5. Gap Analysis

| Feature | Current State | Target State (`DataVizEngine`) |
| :--- | :--- | :--- |
| **Drawing Primitives** | Rectangles, Text | Lines, Curves, Polygons, Arcs |
| **Data Binding** | Manual | Reactive (`Signal` integration) |
| **Interactivity** | Custom Hit Testing | Built-in Tooltips, Hover Effects |
| **Scales & Axes** | None | Automatic D3-style Scales (Linear, Time, Log) |
| **Performance** | DOM-style nodes | Instanced/Batched GPU primitives |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 Core Chart Types
*   The MVP **must** support the three most common chart types:
    *   **Line Chart**: For time-series and continuous data.
    *   **Bar Chart**: For categorical comparisons.
    *   **Scatter Plot**: For distribution analysis.

### 6.2 Declarative API & Data Binding
*   Charts **must** accept data via `flux-state` Signals to enable live updates.
*   The API **must** separate data definition from visual configuration.

```rust
// Illustrative API
LineChart::new(data_signal)
    .x_axis(|d| d.timestamp)
    .y_axis(|d| d.value)
    .color(Theme::primary())
    .curve(Curve::MonotoneX)
```

### 6.3 Scales and Axes Engine
*   The system **must** provide automatic scale calculation to map data domains (e.g., $0 - $1,000,000) to visual ranges (e.g., 0px - 500px).
*   It **must** support Linear, Logarithmic, and Time/Date scales.
*   It **must** automatically generate human-readable axis ticks and grid lines.

### 6.4 High-Performance Rendering
*   Charts **must not** use standard `SceneNode` UI elements for individual data points (to avoid ECS overhead).
*   They **must** utilize batching or instanced rendering via `wgpu` (e.g., a single draw call for a line with 10k segments).

### 6.5 Interactivity
*   Charts **must** support crosshairs and tooltips that follow the cursor to show exact values.
*   They **must** integrate with the Overlay system (`009-overlays-and-popups.md`) for rendering tooltips above other UI.

## 7. Out of Scope (Phase 1)
*   **Complex Chart Types**: Pie charts, Radar charts, Heatmaps, Candlestick (Phase 2).
*   **Animations**: Animated transitions when data changes (Phase 2).
*   **Zoom/Pan**: Interactive zooming and panning of the chart area.
