# 🔭 Vantage: Spec for Chronos (Time Travel)

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
The `experiments/chronos` module currently provides a raw implementation of Undo/Redo.
However, users of the Arthropod Editor (Level Designers, Animators) currently hesitate to make complex changes because "fixing it back" is manual and error-prone.
There is no safety net.

## 2. User Story
**As a** Content Creator (Level Designer / Animator),
**I want to** press `Ctrl+Z` (Undo) to revert my last batch of changes,
**So that** I can experiment with scene composition and properties without fear of destroying my work.

## 3. The "So What?" (Business Value)
*   **User Confidence ("Fearless Exploration")**: Users engage more deeply with tools when they trust the "Undo" safety net.
*   **Efficiency**: Reverting a mistake takes 0.1s (keypress) vs 10s-60s (manual value restoration).

## 4. Success Metrics
*   **Latency**: Undo/Redo operations must complete in < 16ms (1 frame) to maintain UI responsiveness.
*   **Memory Safety**: History must not grow unboundedly; strict limits must be enforceable.

## 5. Acceptance Criteria (MVP -> Production)

### 5.1 Bounded History
*   The system **must** support a configurable limit on history depth (e.g., max 100 steps).
*   When the limit is reached, the oldest transaction **must** be dropped to free memory.

### 5.2 Transaction Batching
*   The system **must** support grouping multiple atomic updates into a single "User Action".
*   *Example*: Dragging a slider updates a value 60 times. Pressing Undo once **must** revert to the state *before* the drag started, not just the previous incremental value.
*   **Requirement**: The system must provide a way to mark the start and end of a logical interaction.

### 5.3 Global Integration
*   The history manager **must** be accessible globally so that standard shortcuts (like `Ctrl+Z`) work from anywhere in the application.
*   Widgets (Text Inputs, Sliders) **must** automatically integrate with this system (e.g., committing an action on focus loss or confirmation).

### 5.4 State Resilience
*   The system **must** be able to preserve and restore the state of any data type used by the application, regardless of its structure.
*   The system **must** handle cases where a tracked object is deleted but its history remains (Ghost History) by failing gracefully.

## 6. Technical Notes (For Engineering Context)
*   Evaluate if the current closure-based approach causes excessive memory fragmentation.
*   Consider immutable data structures for large state history if performance becomes a bottleneck.

## 7. Out of Scope (Phase 1)
*   **Selective Undo**: Removing an action from the middle of the stack.
*   **Collaborative Undo**: Handling conflict resolution in multi-user sessions.
*   **Persisted History**: Saving the Undo stack to disk between sessions.
