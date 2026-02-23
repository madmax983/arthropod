# 🔭 Vantage: Spec for Router & Navigation System

**Status**: Approved
**Owner**: Vantage
**Target Release**: Arthropod Core (Feature Promotion)

## 1. Context & Problem
Enterprise applications are rarely single-screen experiences. They require navigating between different views (Dashboard, Settings, Details), managing history (Back/Forward), and sharing deep links to specific states.
Currently, Arthropod lacks a built-in routing solution. Developers must manually swap root widgets and manage state, leading to inconsistent navigation patterns, no URL synchronization on web, and poor deep-linking support.

## 2. User Story
**As a** Web Application Developer,
**I want to** define routes declaratively (e.g., `/users/:id`),
**So that** my users can navigate using browser controls, bookmark specific pages, and I can organize my code logically.

**As a** User,
**I want to** click a link in an email and be taken directly to the specific invoice it references,
**So that** I don't have to navigate manually from the home screen.

## 3. The "So What?" (Business Value)
*   **Deep Linking**: Essential for sharing content and integrating with external workflows (email, chat).
*   **Standardization**: Provides a "paved road" for application structure, reducing onboarding time for new developers.
*   **Web Compatibility**: Critical for Wasm targets where the URL bar is the source of truth for navigation state.

## 4. Success Metrics
*   **Setup Time**: A developer should be able to add a new route in < 5 minutes.
*   **Performance**: Route transitions must occur in < 16ms (1 frame) for cached views.
*   **Correctness**: The browser URL must always be in sync with the internal router state (and vice versa).

## 5. Gap Analysis

| Feature | `App::run` (Current) | Manual State Management | `Router` (Target) |
| :--- | :--- | :--- | :--- |
| **View Switching** | Static Root | Custom Enums | Declarative Route Map |
| **URL Sync** | None | Manual `web-sys` calls | Automatic (Bi-directional) |
| **Parameters** | None | Ad-hoc parsing | Typed Extraction (`/user/:id`) |
| **Guards** | None | Manual Checks | `before_enter` hooks |
| **History** | None | Manual Stack | Browser History Integration |

## 6. Acceptance Criteria (MVP -> Production)

### 6.1 Declarative Configuration
*   The system **must** allow defining routes as a map of paths to Widgets or Builders.
*   It **must** support dynamic parameters in paths (e.g., `/post/:id`).
*   It **must** support a "Not Found" (404) fallback route.

```rust
Router::new()
    .route("/", || HomeWidget::new())
    .route("/users/:id", |params| UserProfile::new(params.get("id")))
    .fallback(|| NotFoundWidget::new())
```

### 6.2 URL Synchronization (Web Target)
*   On Web (Wasm), the router **must** listen to `popstate` events to update the view when the user clicks Back/Forward.
*   Navigating programmatically (`router.push("/new-path")`) **must** update the browser URL without reloading the page.
*   On Desktop, the router **must** maintain an internal history stack that mimics browser behavior.

### 6.3 Navigation API
*   The system **must** provide a `Navigator` service (likely via Context) accessible to all widgets.
*   API must include: `push(path)`, `replace(path)`, `back()`, `forward()`.
*   Components like `Link` widgets **must** be provided to handle navigation declaratively.

### 6.4 Route Guards
*   The system **must** support `before_enter` hooks to prevent navigation (e.g., "Login Required").
*   If a guard fails, it **must** be able to redirect to another route (e.g., `/login`).

## 7. Constraint Requirements
*   **Flux Integration**: The current route **must** be exposed as a `Signal<Route>` so widgets can reactively update based on location changes.
*   **Async Support**: Route loaders **should** support async operations (e.g., fetching data before showing the view), potentially showing a loading indicator. (Phase 2 optimization, but API should allow it).
*   **Platform Agnostic**: The core routing logic **must** work on both Desktop and Web, with platform-specific adapters for the History API.

## 8. Out of Scope (Phase 1)
*   **Nested Routers**: Independent routers for subsections of the page (e.g., Tab View).
*   **Transition Animations**: Slide/Fade effects between routes.
*   **Code Splitting**: Lazy loading of Wasm modules for routes (requires build tool support).
