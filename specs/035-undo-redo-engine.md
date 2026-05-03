# 🔭 Vantage: Spec for Undo/Redo Engine

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
Currently, Arthropod relies on the `flux-state` reactive system and an ad-hoc Command System for state mutations and actions. While fine-grained reactivity enables high performance UI updates, the framework lacks a structured, application-wide standard for state history.
Developers building complex, stateful enterprise applications (e.g., visual node editors, design tools, document processors) are forced to build bespoke history tracking systems from scratch. These custom implementations often fail to integrate with keyboard shortcuts globally and struggle to handle batched mutations elegantly.

## 2. User Story
**As a** Content Creator or Power User,
**I want to** press `Ctrl+Z` to instantly revert unintended destructive actions (like deleting a complex layout or misconfiguring a form), and `Ctrl+Shift+Z` to reapply them,
**So that** I can work confidently without fear of permanent data loss or tedious manual reversion.

**As a** Developer,
**I want to** integrate my domain specific commands with a framework-level History Manager,
**So that** I can enable global Undo/Redo support with minimal boilerplate, ensuring consistent behavior across the application.

## 3. The "So What?" (Business Value)
*   **User Confidence**: Undo is a fundamental expectation for any desktop-class productivity application. Its absence is a dealbreaker for professional adoption.
*   **Developer Velocity**: A centralized Undo/Redo engine significantly reduces the architectural complexity of building non-trivial applications, allowing developers to focus on domain logic rather than state management plumbing.
*   **Data Integrity**: Standardized reversible commands act as a safety net against accidental data corruption during complex operations.

## 4. Success Metrics
*   **Performance**: Undo/Redo operations must execute with zero observable latency overhead (< 16ms to update UI).
*   **Integration**: The History Manager must seamlessly plug into the existing `010-command-system.md` Command Palette.
*   **Memory Efficiency**: The system must provide configurable limits on history depth to prevent memory leaks in long-lived sessions.

## 5. Gap Analysis

| Feature | Current `flux-state` | `UndoRedoEngine` (Target) |
| :--- | :--- | :--- |
| **Mutation Tracking** | Volatile, overwrites previous state | Recorded in a structured History Stack |
| **Reversibility** | Impossible out-of-the-box | First-class `undo()` / `redo()` APIs |
| **Batching** | N/A | Grouping multiple rapid actions into a single Undo step |
| **Shortcut Binding** | Ad-hoc per developer | Standardized `Ctrl+Z` / `Ctrl+Shift+Z` commands |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 Reversibility Mechanism
*   The system **must** require all registered actions to provide a mechanism to both apply and reverse their effects.
*   It **must** support both state-diffing (delta) and snapshotting approaches.

### 6.2 History Manager
*   A centralized engine **must** maintain separate Undo and Redo stacks.
*   It **must** clear the Redo stack whenever a new action is performed after one or more Undo operations.
*   It **must** support configuring a maximum history capacity (e.g., retaining the last 100 actions) to manage memory limit constraints.

### 6.3 Command System Integration
*   The system **must** provide default bindings for standard keyboard shortcuts (`Ctrl+Z`, `Ctrl+Shift+Z`, `Cmd+Z`, `Cmd+Shift+Z`).
*   "Undo" and "Redo" actions **must** be exposed to the Command Registry for accessibility via the Command Palette.

### 6.4 Batching & Debouncing
*   The engine **must** support grouping multiple atomic operations into a single logical "transaction" (e.g., grouping continuous slider movements into one Undo step).

## 7. Out of Scope (Phase 1)
*   **Non-linear History**: Branching history trees (like Git). The MVP is strictly linear.
*   **Automatic State Snapshotting**: The framework will not automatically serialize and snapshot the entire ECS World. Developers are responsible for defining the `undo()` logic for their specific domains.
*   **Collaborative History**: Multiplayer real-time operational transformation (OT) or CRDT-based undo capabilities.
