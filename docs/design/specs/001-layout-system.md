# 🔭 Vantage Spec: Layout System Integration

**Feature**: Automatic Flexbox Layout
**Status**: Draft
**Owner**: Vantage
**Impact**: High (Core Feature)

## 1. 👤 User Story & Value

**As a** UI Developer,
**I want** to use declarative layout primitives (Row, Column) and properties (Padding, Gap),
**So that** my interface automatically adapts to content size and window resizing without me writing manual pixel math.

**Value Proposition**:
- Reduces boilerplate code (no more `node.bounds = ...` in event loops).
- Enables responsive design.
- Aligns with modern UI frameworks (React, Flutter, SwiftUI).

## 2. ✅ Acceptance Criteria

### Functional Requirements
1.  **ECS Integration**:
    - A new system `layout_system` must be added to `FrameworkContext`.
    - It must process entities with `SceneNodeRef` and `LayoutStyle` components.
2.  **Taffy Integration**:
    - The system must construct a Taffy tree mirroring the Scene tree (for nodes with layout).
    - It must synchronize `FlexStyle` changes from ECS to Taffy.
3.  **Output Sync**:
    - Computed layout (x, y, width, height) must be written back to the corresponding `SceneNode`'s `bounds`.
    - This must happen every frame (or intelligently when dirty).
4.  **Integration Test**:
    - A test case (e.g., `tests/layout_integration.rs`) must demonstrate a hierarchy (Row -> [Item, Item]) correctly positioning children based on the parent's size.

### Non-Functional Requirements
1.  **Performance**:
    - Layout calculation for < 1,000 nodes must take < 1ms.
    - Should ideally use caching/dirty marking to avoid full re-layout every frame if nothing changed.

## 3. 📝 Technical Approach (High Level)

*Note: Engineering to refine.*

1.  **Layout System**: A `bevy_ecs` system that runs in `FrameworkContext::update`.
2.  **State Management**: `LayoutEngine` should be a Resource in the World.
3.  **Synchronization**:
    - Iterate `Scene` to build/update Taffy tree.
    - Apply constraints (Window size) to the root.
    - `layout_engine.compute_layout()`.
    - Apply results to `SceneNode.bounds`.

## 4. 🚫 Out of Scope (Phase 1)
- Grid layout (Flexbox only for now).
- Absolute positioning within flex containers (keep it simple).
- Z-index sorting (handled by Scene tree order).

## 5. 📅 Roadmap
1.  Add `LayoutEngine` as a Resource in `FrameworkContext`.
2.  Implement `layout_system`.
3.  Add `layout_system` to `FrameworkContext::update_schedule`.
4.  Verify with `colored_rectangles.rs` (refactor to use Layout) or a new example.
