# 🔭 Vantage: Spec for Docking & Split Layout Manager

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
While Arthropod has flexbox-like grid and layout structures (e.g. `Row`, `Column`, `Grid`), complex enterprise tools and IDEs (like Blender, VS Code, or financial trading terminals) require a dynamic **Docking & Split Layout Manager**. Users need the ability to rearrange panels, dock tools to different edges, tear off floating windows, and create tabbed groups. Building this custom with basic layouts is highly complex and error-prone.

## 2. User Story
**As a** Power User (Trader, Developer, or 3D Artist),
**I want to** arbitrarily split my workspace, drag panels to reorder them, and tab them together,
**So that** I can fully customize my workspace to fit my multi-monitor workflow and specific tasks.

## 3. The "So What?" (Business Value)
*   **Workflow Flexibility**: Professional workflows are rarely one-size-fits-all. Providing a native docking system immediately elevates the framework from "basic UI" to "IDE-grade UI."
*   **Standardization**: A robust docking manager is notoriously hard to write. Supplying an out-of-the-box, accessible, and performant implementation reduces development time by weeks for complex B2B applications.
*   **User Retention**: High customizability is a key retention factor for professional software tools.

## 4. Success Metrics
*   **Layout Computation Time**: < 5ms to recalculate bounds when dragging splitters for complex hierarchies.
*   **Interaction Latency**: < 16ms delay when dragging a tab to dock it (60fps feedback).
*   **API Ergonomics**: Declarative setup for default layouts using a simple tree-based configuration API.

## 5. Gap Analysis

| Feature | Current Layout Engine | Docking Manager (Target) |
| :--- | :--- | :--- |
| **Resizing** | Static or flex ratios | User-draggable splitters |
| **Panel Reordering** | Developer-defined | User-draggable (Drag-and-Drop) |
| **Tabbed Views** | Manual implementation | Built-in grouping |
| **Floating Windows** | Window abstraction only | Tear-off panels to floating windows |
| **State Serialization**| None | Built-in save/restore layout state |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 Splitter Controls
*   The system **must** provide horizontal and vertical splitters between docked panels.
*   Users **must** be able to drag splitters to resize adjacent panels.
*   Splitters **must** respect min/max width/height constraints of the panels.

### 6.2 Docking Zones
*   Dragging a panel **must** display docking indicators (Left, Right, Top, Bottom, Center/Tab).
*   Dropping on an edge **must** split the current region.
*   Dropping on the center **must** group the panels as tabs.

### 6.3 Tab Groups
*   Grouped panels **must** display a tab bar.
*   Users **must** be able to reorder tabs within the group via drag-and-drop.
*   Users **must** be able to close tabs if configured as closeable.

### 6.4 Serialization
*   The docking state **must** be serializable to/from JSON (or similar format) to support workspace persistence (tying into the State Persistence Engine).

## 7. Constraint Requirements
*   **Performance**: Drag operations and continuous resizing must not block the main thread or cause stuttering.
*   **Accessibility**: Docking operations must be achievable via keyboard (e.g., focused panel -> shortcut -> move to new region).

## 8. Out of Scope (Phase 1)
*   **Floating/Tear-off Windows**: Ripping a panel out into a new OS window (requires complex multi-window orchestration in Phase 2).
*   **Auto-hide Panels**: Panels that collapse to the edge and slide out on hover.
*   **Custom Docking Shapes**: Non-rectangular window splits.