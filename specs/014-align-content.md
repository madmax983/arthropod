# 🔭 Vantage: Spec for Align Content

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Layout Engine V2)

## 1. Context & Problem
Currently, our layout engine supports basic Flexbox properties, and we have drafted a spec for Layout Engine V2 to include more features. However, when we wrap content to multiple lines (using `flex-wrap: wrap`), we currently lack the ability to align those newly created lines along the cross-axis.

Without `align-content`, developers cannot distribute space between rows of items or pack rows tightly together. This makes complex grid-like layouts or wrapping pill-containers (like a tag cloud) extremely tedious to style and align properly within their parent container.

## 2. User Story
**As a** UI Developer,
**I want to** specify how multiple lines of content are aligned along the cross-axis when items wrap,
**So that** I can easily create packed or evenly spaced multi-line layouts (like a gallery or tag cloud) without manual padding calculations.

## 3. The "So What?" (Business Value)
*   **Developer Efficiency**: Eliminates the need for manual row-by-row padding/margin calculations when dealing with wrapping content.
*   **Responsive Design**: Enables layouts that elegantly adapt to varying container sizes by appropriately spacing multiple lines of content.
*   **Parity**: Closes a significant gap with web/CSS Flexbox standards, reducing the learning curve for developers coming from web development.

## 4. Success Metrics
*   **API Ergonomics**: Defining cross-axis line alignment should require a single property on the container.
*   **Visual Correctness**: Multi-line layouts must perfectly respect alignment (e.g., center, space-between, stretch) when wrapping occurs.
*   **Performance**: Layout calculation for complex wrapping scenarios must remain within our < 1ms budget for 1,000 nodes.

## 5. Gap Analysis
| Feature | Current Layout Engine | Target V2 |
| :--- | :--- | :--- |
| **Cross-Axis Item Alignment** | Partial (`align-items` proposed) | Supported |
| **Main-Axis Alignment** | Partial (`justify-content` proposed) | Supported |
| **Cross-Axis Line Alignment** | ❌ Missing | ✅ Add `align-content` support |

## 6. Acceptance Criteria
*   The layout container must accept a property to define how wrapping lines are aligned along the cross-axis.
*   Supported alignment behaviors must include: packing lines to the start, packing to the end, centering lines, stretching lines to fill the container (default), and distributing space between, around, or evenly across lines.
*   The default behavior must match standard CSS Flexbox (stretching lines to fill the cross-axis).
*   If the layout does not wrap (single line), this property should have no effect.

## 7. Constraint Requirements
*   **Backward Compatibility**: Existing layouts must not break. The default behavior should seamlessly integrate with existing, non-wrapping layouts.
*   **Performance**: The constraint solver or layout engine should not introduce new allocation overhead for calculating line distributions.

## 8. Out of Scope
*   **CSS Grid Support**: This feature is strictly for Flexbox cross-axis line alignment. Full 2D grid support remains out of scope for this spec.
*   **Individual Line Overrides**: Aligning a specific wrapping line differently from the others is not part of this specification.
