# 🔭 Vantage: Spec for Slider Widget

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Widget Pack 2)

## 1. Context & Problem
Currently, our `widget-core` provides `TextInput` for entering numerical data, but typing numbers manually is often slow, error-prone, and provides a poor user experience when adjusting continuous values (e.g., volume, brightness, zoom levels).
Users lack a tactile, visual method to adjust numeric values within a defined range.

## 2. User Story
**As a** Media Application User,
**I want to** adjust the volume and timeline position using a draggable slider,
**So that** I can intuitively control playback without typing numbers.

**As a** Data Analyst using the Dashboard,
**I want to** filter data by adjusting a date or value range via a slider,
**So that** I can quickly explore different data segments visually.

## 3. The "So What?" (Business Value)
*   **User Experience**: Provides a standard, intuitive interaction model expected in all modern GUI applications.
*   **Error Prevention**: Restricts input to a valid `min`/`max` range, eliminating out-of-bounds validation errors.
*   **Accessibility**: Enhances usability for users who prefer mouse/touch interaction over keyboard input.

## 4. Success Metrics
*   **Performance**: Smooth rendering at 60 FPS while dragging the slider thumb.
*   **Accuracy**: The slider's visual position exactly matches the underlying numerical value.
*   **Adoption**: Replaces at least 30% of numeric `TextInput` fields in existing internal tools.

## 5. Acceptance Criteria

### 5.1 Core Functionality
*   The system **must** provide a `Slider` widget that accepts a `min`, `max`, and current `value` (signal).
*   The widget **must** allow users to drag the "thumb" along a "track" to change the value.
*   The widget **must** support a `step` property to lock values to specific increments (e.g., step by 1.0 or 0.5).

### 5.2 Interaction & Events
*   The widget **must** emit an `on_change(value)` event during dragging (continuous updates).
*   The widget **must** emit an `on_release(value)` event when the user finishes dragging (mouse up).
*   Clicking anywhere on the track **must** instantly move the thumb to that position.

### 5.3 Visual Feedback
*   The track **should** visually distinguish between the "filled" portion (min to current value) and the "empty" portion (current value to max).
*   The thumb **should** provide visual feedback on hover and active (dragging) states.

### 5.4 Accessibility & Keyboard Support
*   The widget **must** be focusable via the keyboard (Tab).
*   When focused, the Left/Right arrow keys **must** decrease/increase the value by the `step` amount.
*   When focused, the Home/End keys **must** set the value to `min` and `max` respectively.
*   The widget **must** expose its role, min, max, and current value to accessibility APIs.

## 6. Out of Scope (Phase 1)
*   **Range Slider**: A slider with two thumbs for selecting a range (min/max pair).
*   **Vertical Slider**: A slider oriented vertically instead of horizontally.
*   **Custom Tick Marks**: Visual markers or labels along the track.
