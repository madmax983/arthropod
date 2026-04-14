# 🔭 Vantage: Spec for Carousel

## 1. Context & Problem
Currently, there is no standardized way to display a set of items in a horizontal, swipeable or pageable format. Developers must build custom container widgets and wire up state management to track the active item. A native Carousel widget solves this by providing a reusable component with built-in state.

## 2. User Story
**As a** UI Designer,
**I want** a Carousel widget that allows me to page horizontally through a collection of widgets,
**So that** I can efficiently display galleries, tutorials, or featured content within a constrained space.

**As a** Developer,
**I want** the Carousel to expose its active index via `flux-state` signals,
**So that** I can easily observe which item is in view and programmatically control the active item.

## 3. The "So What?" (Business Value)
*   **Reusability**: Standardizes a very common UI pattern (galleries, onboarding).
*   **Reactivity**: Deeply integrates with `flux-state` so users don't have to manually wire up active index tracking.

## 4. Success Metrics
*   **Performance**: Switching between items does not cause layout recalculation for non-visible items, keeping rendering under 16ms.

## 5. Acceptance Criteria
*   The API must provide a `Carousel` widget that takes a list of child widgets.
*   The widget must use `flux-state` signals to keep track of the active index.
*   Users must be able to read the current active index via a signal.
*   Users must be able to write to the current active index signal to change the visible item.
*   If the list of children is empty, the widget should gracefully handle it (e.g., render an empty state or empty layout).

## 6. Out of Scope (Phase 1)
*   Smooth animations or physics-based scrolling between items.
*   Infinite looping (wrapping from the last item back to the first).
*   Vertical carousels.
