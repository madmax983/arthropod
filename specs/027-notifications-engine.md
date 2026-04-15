# 🔭 Vantage: Spec for Notifications & Toast Engine

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
Modern applications need non-intrusive ways to provide feedback to users—whether a background task succeeded (e.g., "File Saved"), a network request failed, or a new message arrived.
Currently, developers must build custom absolute-positioned overlays, manage their own timers to auto-dismiss them, and manually handle stacking them so they don't overlap. This leads to inconsistent user experiences and excessive boilerplate for a fundamental UI pattern.

## 2. User Story
**As a** User,
**I want to** receive brief, non-blocking feedback when I perform an action (like saving a document) or when a background event occurs,
**So that** I know the system's state without being interrupted by modal dialogs.

**As a** Developer,
**I want to** trigger a toast notification with a single function call from anywhere in my application,
**So that** I don't have to manage UI state, animations, timers, and layout calculations just to say "Success".

## 3. The "So What?" (Business Value)
*   **User Trust**: Immediate, reliable feedback loop (Action -> Result) builds user confidence in the application's stability.
*   **Developer Velocity**: Standardizes a universal UI pattern. A single line of code (`notify_success("Saved")`) replaces hundreds of lines of custom UI management, saving days of development time across a large application.
*   **Consistency**: Ensures all notifications look, animate, and behave the same way, contributing to a polished, professional product feel.

## 4. Success Metrics
*   **Ergonomics**: Triggering a standard notification should require exactly one line of code from any component.
*   **Performance**: Spawning 10 notifications simultaneously must not cause frame drops (< 16ms overhead).
*   **Reliability**: Auto-dismiss timers must fire accurately even if the main thread is briefly blocked.

## 5. Gap Analysis

| Feature | Current State (Custom) | Notifications Engine (Target) |
| :--- | :--- | :--- |
| **Invocation** | Dispatch state to store -> Render overlay | Imperative API (`toast.info()`) |
| **Layout/Stacking** | Manual absolute positioning | Auto-stacking (Top-Right, Bottom-Center, etc.) |
| **Lifecycle** | Manual timers (`setTimeout` equivalent) | Built-in auto-dismiss, pause on hover |
| **Animation** | Custom transitions | Built-in slide/fade in & out |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 Imperative API
*   The system **must** provide a globally accessible service (or context) to spawn notifications without modifying local component state.
*   It **must** support at least 4 semantic types out of the box: `Info`, `Success`, `Warning`, and `Error` (each with distinct default styling).

### 6.2 Layout & Positioning
*   The engine **must** support configuring the default spawn location (e.g., `TopRight`, `BottomLeft`, `TopCenter`).
*   Multiple notifications **must** stack gracefully, automatically pushing older notifications up or down to make room for new ones without overlapping.

### 6.3 Lifecycle Management
*   Notifications **must** auto-dismiss after a configurable duration (defaulting to e.g., 4000ms).
*   The auto-dismiss timer **must** pause while the user's cursor is hovering over the notification.
*   Notifications **must** provide a manual "close" button (X).

### 6.4 Customization
*   Developers **must** be able to override the default text with custom widgets for the notification body (e.g., adding an action button like "Undo" inside the toast).

## 7. Out of Scope (Phase 1)
*   **OS-Level Notifications**: Integrating with Windows/macOS native notification centers. This engine is strictly for in-app UI notifications.
*   **Notification Center / Inbox**: A persistent list of past notifications that the user can open. This spec only covers ephemeral "Toast" popups.
*   **Swipe-to-Dismiss**: Touch gestures for mobile/tablet to dismiss notifications (Phase 2).