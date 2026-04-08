# 🔭 Vantage: Spec for Native File Dialog Support

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
Arthropod currently lacks a standardized way to interact with the host OS file system for opening, saving, or selecting directories. Users building desktop applications (e.g., text editors, image viewers, IDEs) must manually integrate third-party crates (like `rfd` or `native-dialog`) and wire them up to Arthropod's event loop, which often leads to poor UX (blocking the main thread) or complex asynchronous workarounds.

## 2. User Story
**As a** Desktop Application Developer,
**I want to** trigger native OS file picker dialogs (Open File, Save File, Pick Folder) natively through the Arthropod API,
**So that** my users have a familiar, secure, and seamless experience when interacting with their file system, without me having to write platform-specific integration code.

## 3. The "So What?" (Business Value)
*   **Table Stakes for Desktop**: Any serious desktop framework must provide out-of-the-box file system access. Missing this forces developers to look at alternatives like Tauri or Electron.
*   **Developer Velocity**: Standardizing this reduces boilerplate and prevents common mistakes (like blocking the render thread while waiting for user input).

## 4. Success Metrics
*   **Zero Blocking**: The main event/render loop must *not* block while the dialog is open.
*   **Cross-Platform Consistency**: Must work natively on Windows, macOS, and Linux (via standard protocols like XDG Desktop Portal).

## 5. Acceptance Criteria (MVP -> Production)

### 5.1 Core Dialog Types
*   The system **must** support:
    *   Single File Selection (Open)
    *   Multiple File Selection (Open)
    *   Directory Selection
    *   File Save (with default name and extension filters)

### 5.2 Asynchronous API
*   The API **must** be fully asynchronous, returning a Future or integrating with Arthropod's ECS event system (e.g., emitting a `FileSelectedEvent`).
*   The main thread **must not** freeze while the user is interacting with the native OS dialog.

### 5.3 Filtering and Configuration
*   The system **must** allow developers to set file type filters (e.g., "Images", `*.png`, `*.jpg`).
*   The system **must** allow setting a default starting directory.

## 6. Technical Notes (For Engineering Context)
*   Investigate integrating existing crates like `rfd` (Rust File Dialog) as the underlying engine to avoid reinventing the wheel for platform-specific bindings.
*   Ensure the dialogs are correctly parented to the Arthropod main window (e.g., passing the raw window handle to the dialog crate) so they behave modally if required by the OS.

## 7. Out of Scope (Phase 1)
*   **Custom UI Dialogs**: This feature is strictly for native OS dialogs. Rendering a custom file picker using Arthropod widgets is out of scope.
*   **Mobile Support**: iOS/Android file picking protocols are out of scope for Phase 1.
