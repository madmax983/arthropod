# 🔭 Vantage: Spec for Tooltip Widget

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Widget Layer)

## 1. Context & Problem
Currently, our framework lacks a native mechanism to display contextual help or secondary information when a user interacts with a widget. Users are often confused by dense UI elements (like icon-only buttons or complex data grids) because there's no way to provide on-demand explanations without cluttering the main layout.

## 2. User Story
**As a** UI Designer,
**I want to** attach a small, transient pop-up containing descriptive text to any UI element,
**So that** users can discover what an icon, button, or complex metric means without leaving their current workflow.

## 3. The "So What?" (Business Value)
*   **User Retention & Onboarding**: Reduces user frustration by providing immediate, context-sensitive help, making complex professional software easier to learn.
*   **Screen Real Estate**: Allows designers to build cleaner, denser interfaces (e.g., icon-heavy toolbars) without sacrificing clarity.
*   **Accessibility**: Provides a standard pathway to expose detailed descriptions for screen readers and keyboard-only users navigating the UI.

## 4. Success Metrics
*   **Metric Definition**: Success = Tooltip renders on screen within 16ms of hover trigger; 0 overlapping z-index bugs with other overlays (modals/dropdowns); API requires no more than one property configuration per widget.

## 5. Gap Analysis
| Feature | Current Arthropod Framework | Target V1 |
| :--- | :--- | :--- |
| **Hover State Detection** | Yes (`input-engine`) | ✅ Leverage existing input events |
| **Overlay/Z-Index System** | Partial (Spec 009 Draft) | ✅ Integrate with global overlay layer |
| **Contextual Help UI** | ❌ Missing | ✅ Add standard Tooltip widget/behavior |

## 6. Acceptance Criteria
*   Any standard widget must be able to accept a `tooltip` property containing text or another simple layout.
*   The tooltip must appear when the user hovers over the parent element for a configurable delay (default: 500ms).
*   The tooltip must disappear immediately when the user's cursor leaves the parent element.
*   The tooltip must automatically position itself relative to the parent element (e.g., top, bottom, left, right) while ensuring it stays within the visible bounds of the application window.
*   The tooltip must render above all other standard UI elements (using the highest z-index/overlay layer).

## 7. Constraint Requirements
*   **Performance**: Attaching tooltips to thousands of items (e.g., in a data grid) must not incur performance penalties unless they are actively triggered.
*   **Platform Independence**: Tooltip positioning and rendering must work identically on Windows, macOS, and headless environments.

## 8. Out of Scope
*   **Rich Interactive Tooltips**: Tooltips containing interactive elements like forms, links, or buttons (use a Popover instead).
*   **Custom Animation Timings**: Hardcoded fade-in/out animations are acceptable for Phase 1. Fully reactive customizable motion is deferred to Spec 003 integration.
