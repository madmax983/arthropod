# 🔭 Vantage Spec: Layout System Integration

**Feature**: Automatic Flexbox Layout (ECS System)
**Status**: Approved
**Owner**: Vantage / Jules
**Impact**: High (Core Feature)

## 1. 👤 User Story & Value

**As a** UI Developer,
**I want** my interface to automatically layout itself (Flexbox) whenever I change content or resize the window,
**So that** I don't have to manually call layout functions or calculate pixel coordinates.

**Value Proposition**:
- **Zero-Boilerplate**: Layout just works.
- **Reactive**: Changing a text signal automatically triggers re-layout if the text size changes.
- **Robust**: Eliminates "forgetting to call layout" bugs.

## 2. ✅ Acceptance Criteria

### Functional Requirements
1.  **ECS Integration**:
    - A new system `layout_system` must be running in `FrameworkContext::update`.
    - It must process entities with `SceneNodeRef` and `LayoutStyle` components.
2.  **Automatic Updates**:
    - Changing `FlexStyle` component -> Updates Scene Node bounds.
    - Resizing Window -> Updates Root Node bounds -> Updates Layout.
    - Changing Content (Text) -> Updates Node Size -> Updates Layout.
3.  **Taffy Integration**:
    - The system must bridge ECS components to the `layout-engine` (Taffy).
4.  **Verification**:
    - `crates/arthropod-ecs/tests/layout_integration.rs` must pass.
    - `crates/arthropod/src/app/widget.rs` should no longer need manual `auto_layout` calls.

### Performance Requirements
- Layout calculation for < 1,000 nodes must take < 1ms.
- Ideally, cache the Taffy tree (or rebuild cheaply). For Phase 1, rebuilding every frame is acceptable IF performance remains < 1ms for typical scenes (100-500 nodes).

## 3. 📝 Technical Approach

1.  **System**: `arthropod_ecs::systems::layout::layout_system`.
2.  **Logic**:
    - Acquire `Scene` (Resource) and `Window` (Resource/Config).
    - Iterate `Scene` to build Taffy tree.
    - Apply `FlexStyle` from ECS components (`Query<(Entity, &SceneNodeRef, &LayoutStyle)>`).
    - Compute Layout.
    - Write back to `SceneNode.bounds`.
3.  **Registration**: Add to `arthropod_ecs::context::FrameworkContext::build_update_schedule`.

## 4. 🚫 Out of Scope (Phase 1)
- Incremental Layout (Dirty Marking). Phase 1 will rebuild tree every frame for simplicity/correctness.
- Grid Layout.
- Multi-pass layout for text wrapping (handled by `render-engine` or `text-engine` separately, though layout needs to know text size).

## 5. 📅 Roadmap
1.  Implement `layout_system` in `arthropod-ecs`.
2.  Register in `FrameworkContext`.
3.  Remove manual calls in `WidgetApp`.
4.  Verify with integration test.
