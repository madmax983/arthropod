# 🔭 Vantage: Spec for Layout Engine V2 (Flexbox Support)

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
The current `layout-engine` implementation in Arthropod is a minimal wrapper around `taffy` that exposes only basic properties: `direction`, `flex_grow`, `flex_shrink`, and explicit dimensions.
However, it completely lacks support for critical Flexbox features required for modern UI layouts:
*   **Alignment**: `align-items`, `align-content`, `justify-content` (centering items, spacing them out).
*   **Wrapping**: `flex-wrap` (allowing items to flow to multiple lines).
*   **Sizing**: `flex-basis` (initial size before growing/shrinking).

Without these, developers cannot implement basic patterns like "Centered Modal", "Tag Cloud", or "Space-Between Header" without resorting to manual spacer widgets or nested hacks.

## 2. User Story
**As a** UI Developer,
**I want to** set `justify-content: center` and `align-items: center` on a container,
**So that** I can easily center a loading spinner or modal dialog without calculating coordinates manually.

**As a** Designer,
**I want to** create a responsive grid of cards that wraps to the next line when the window is resized,
**So that** the dashboard looks good on both wide and narrow screens.

## 3. The "So What?" (Business Value)
*   **Developer Efficiency**: Reduces the "fight with the layout" time. Centering a div (or widget) should be one line of code.
*   **Code Cleanliness**: Removes the need for invisible "Spacer" widgets just to push content to the right.
*   **Responsiveness**: Enables fluid layouts that adapt to window resizing automatically, a core requirement for enterprise dashboards.

## 4. Success Metrics
*   **Performance**: Layout calculation for 1,000 nodes must remain < 1ms (on par with current implementation).
*   **Parity**: 100% of the `taffy` crate's flexbox capabilities must be exposed via the `FlexStyle` struct.
*   **Ergonomics**: Defining a centered layout should require no more than 3 properties (`direction`, `justify`, `align`).

## 5. Gap Analysis

| Feature | `layout-engine` (Current) | `taffy` (Underlying) | Target V2 |
| :--- | :--- | :--- | :--- |
| **Direction** | `Row` / `Column` | Supported | ✅ Keep |
| **Sizing** | `width` / `height` | Supported | ✅ Keep |
| **Growth** | `flex_grow` / `flex_shrink` | Supported | ✅ Keep |
| **Basis** | ❌ Missing | Supported | ✅ Add `flex_basis` |
| **Main Axis Align** | ❌ Missing | Supported | ✅ Add `justify_content` |
| **Cross Axis Align** | ❌ Missing | Supported | ✅ Add `align_items`, `align_self` |
| **Content Align** | ❌ Missing | Supported | ✅ Add `align_content` |
| **Wrapping** | ❌ Missing | Supported | ✅ Add `flex_wrap` |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 Expanded `FlexStyle` Struct
*   The `FlexStyle` struct **must** include fields for:
    *   `justify_content`: `Start`, `End`, `Center`, `SpaceBetween`, `SpaceAround`, `SpaceEvenly`.
    *   `align_items`: `Start`, `End`, `Center`, `Baseline`, `Stretch`.
    *   `align_self`: Overrides parent's `align_items`.
    *   `align_content`: For multi-line wrapping layouts.
    *   `flex_wrap`: `NoWrap`, `Wrap`, `WrapReverse`.
    *   `flex_basis`: `Auto`, `Points(f32)`, `Percent(f32)`.

### 6.2 Enum Exposure
*   The system **must** re-export or redefine the necessary `taffy` enums (`JustifyContent`, `AlignItems`, `FlexWrap`) so users don't need to depend on `taffy` directly.
*   Default values **must** match the CSS Flexbox specification (e.g., `flex-direction: row`, `align-items: stretch`, `justify-content: flex-start`).

### 6.3 Integration
*   The `convert_style` function in `layout-engine` **must** correctly map these new fields to the `taffy::Style` struct.
*   Existing code using `FlexStyle` **must** continue to compile (backward compatibility via `..Default::default()`).

### 6.4 Testing
*   Unit tests **must** verify that setting `justify_content: Center` actually results in centered coordinates for children.
*   Unit tests **must** verify that `flex_wrap: Wrap` moves children to a new line when width is constrained.

## 7. Constraint Requirements
*   **No Breaking Changes**: The `NodeId` and `LayoutEngine` API surface should remain stable. Only `FlexStyle` is expanding.
*   **Performance**: Do not introduce additional allocation overhead in the style conversion hot path.

## 8. Out of Scope (Phase 1)
*   **CSS Grid**: Full 2D grid layout (Phase 3).
*   **Text Layout Integration**: Complex text wrapping interaction with flexbox (handled by `text-engine` currently).
