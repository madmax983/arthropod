# 🔭 Vantage: Spec for System Tray Integration

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core

## 1. Context & Problem
Many enterprise and productivity applications need to run continuously in the background without cluttering the user's taskbar or requiring an open window. Currently, closing an Arthropod application window terminates the entire application process. There is no built-in mechanism to minimize to the system tray, run background tasks, or provide quick access via a tray icon menu.

## 2. User Story
**As a** User,
**I want to** minimize the application to the system tray and access common actions via a right-click menu,
**So that** the application can run quietly in the background without taking up taskbar space, while still giving me quick access when needed.

**As a** Developer,
**I want to** define a system tray icon, tooltips, and a context menu programmatically,
**So that** I can build background services and daemon-like applications that integrate natively with the host OS.

## 3. The "So What?" (Business Value)
*   **User Retention**: Background applications (like chat apps, sync clients, monitoring tools) stay running longer, increasing engagement.
*   **Desktop Native Feel**: System tray support is a core expectation for native desktop applications. Lacking it makes the framework feel like a toy or purely a web-wrapper.
*   **Resource Efficiency**: Running headlessly in the tray uses significantly fewer resources than keeping a full GUI window open and rendering.

## 4. Success Metrics
*   **OS Support**: Must work seamlessly across Windows, macOS, and Linux.
*   **Responsiveness**: Tray icon menus must open instantly (< 16ms latency).
*   **Resource Usage**: Minimizing the main window to the tray should drop GPU usage to 0% and minimize CPU footprint.

## 5. Gap Analysis

| Feature | Current Arthropod (`App`) | Target (`System Tray`) |
| :--- | :--- | :--- |
| **App Lifecycle** | Bound to Window | Independent of Window (can run headlessly) |
| **Tray Icon** | None | Customizable icon (PNG/ICO) |
| **Quick Actions** | Requires opening app | Native context menu on tray icon |
| **Notifications** | None | Trigger OS notifications from tray |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 Tray Icon Management
*   The system **must** provide an API to set, update, and remove the system tray icon dynamically.
*   It **must** support setting a tooltip string that appears when the user hovers over the tray icon.

### 6.2 Context Menu
*   Developers **must** be able to attach a declarative menu to the tray icon (e.g., standard items, checkable items, submenus, separators).
*   Menu items **must** be able to emit events or dispatch actions back to the Arthropod ECS/Flux state when clicked.

### 6.3 Window Lifecycle Integration
*   The application **must** support a configuration option to "Close to Tray" or "Minimize to Tray" instead of exiting the process.
*   Clicking the tray icon **must** be able to restore/focus the main application window.
*   The application **must** be able to run completely headlessly (Tray only) without ever creating a primary window.

## 7. Out of Scope (Phase 1)
*   Custom rendering *inside* the tray menu (menus must use native OS widgets).
*   Complex animations for the tray icon (static images only for MVP).
