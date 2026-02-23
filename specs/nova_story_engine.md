# 🔭 Vantage: Spec for Nova Story Engine (Phase 1)

**Status:** Draft
**Owner:** Vantage (Product)
**Date:** 2026-05-23

## 1. Problem Statement
The current "Story Engine" implementation is a stub that prints a single hardcoded text block. Users are confused because the feature is advertised but non-functional.

## 2. User Story
**As a** Narrative Designer or Game Developer,
**I want** to define interactive, branching stories using a structured data model (Passages and Choices),
**So that** I can create text-based games or interactive fiction without writing custom state machine logic for every story.

## 3. Solution Overview
We will implement a proper runtime for the Nova Story Engine.

### Core Components
1.  **Model (`model.rs`)**:
    *   `Story`: The container for all passages.
    *   `Passage`: A single node in the story graph (ID, Text, Choices).
    *   `Choice`: A link to another passage (Text, Target ID).
2.  **Runtime (`runtime.rs`)**:
    *   `StoryRuntime` (ECS Resource): Tracks the `current_passage_id` and `history`.
    *   Methods: `choose(index)` to transition to the next passage.
3.  **System (`system.rs`)**:
    *   `story_view_system`: Reacts to `StoryRuntime` changes.
    *   Updates the `NarrativeGenerator` entity's children in the Scene Graph.
    *   Child 0: The Passage Text (Styled).
    *   Child 1..N: The Choices (Styled as "[1] Choice Text").

## 4. Acceptance Criteria
- [ ] **Data Model**: Can define a `Story` with at least 3 linked passages.
- [ ] **State Tracking**: `StoryRuntime` correctly updates `current_passage_id` when a choice is made.
- [ ] **Rendering**: The Scene Graph reflects the *current* passage text and available choices.
- [ ] **Interaction**: The `story_demo` example allows the user to press number keys (1, 2, etc.) to trigger choices and advance the story.

## 5. Out of Scope (Phase 1)
- Saving/Loading state to disk.
- Scripting (variables, conditionals like `if visited("cave")`).
- Complex UI layout (rendering buttons as actual clickable widgets - currently just text nodes).
- Markdown parsing in the engine (the TUI demo handles this, but the engine just emits raw text).
