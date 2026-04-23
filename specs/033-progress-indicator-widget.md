# Spec: Progress Indicator Widget (Progress Bar & Spinner)

## 👤 User Story
As an Application User, I want to see a visual indication of ongoing background processes (like file downloads, data processing, or network requests), so that I know the application hasn't frozen and I can estimate how much longer the task will take.

## 🎯 Business Value ("So What?")
- **Reduces User Frustration:** Applications that lack feedback during operations lasting >1 second are frequently perceived as "frozen," leading to premature app termination and data corruption.
- **Improves Perceived Performance:** Visual feedback makes waiting feel shorter, keeping users engaged with the application.

## 📊 Metric Definition
- **Success =**
  - Standard progress bars and indeterminate spinners can be embedded in any Layout without requiring custom rendering code.
  - Frame drops during indeterminate spinner animations remain at 0 on both integrated and discrete GPUs.

## 🕳️ Gap Analysis
Currently, developers must hack together rectangles and manual timer loops to indicate progress, which is unmaintainable and often leads to performance issues. We lack a native, high-performance way to show "loading" or "progress" states within the core widget library.

## ✅ Acceptance Criteria
1. **Determinate Progress Bar:** Must support setting a concrete progress value (e.g., `0.0` to `1.0` or `0%` to `100%`) which visually updates the bar's fill.
2. **Indeterminate Spinner/Bar:** Must provide an "indeterminate" mode for operations of unknown duration (e.g., an infinitely looping animation).
3. **Accessibility Integration:** The widget must automatically expose its progress state (value, max, min, or "busy") to the Accessibility Engine.
4. **Theming Support:** Must integrate with the Theme Engine to allow customization of colors (track, fill, spinner ring) and sizes.

## 🚫 Out of Scope
- Creating a unified "Task Manager" or background job queue (this is purely the UI representation).
- Step-by-step wizard indicators (e.g., "Step 1 of 4" discrete progress bubbles).
