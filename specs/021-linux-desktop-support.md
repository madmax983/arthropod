# 🔭 Vantage: Spec for Linux Desktop Support

**Status:** Draft
**Owner:** Vantage (Product)
**Target Release:** Arthropod Core

## 1. Problem Statement
Currently, `App::run` requires Windows or macOS to initialize the `wgpu` graphical context. On Linux platforms (X11 or Wayland), users are forced to use `App::new_headless()` or rely on TUI fallbacks. This severely limits the adoption of Arthropod as a truly cross-platform GUI framework, as developers cannot build native graphical applications for Linux.

## 2. User Story
👤 **User Story:**
As a Developer building a cross-platform application,
I want Arthropod to support native graphical window creation and rendering on Linux (Wayland/X11),
so that I can ship a unified codebase to Linux desktop users without compromising on performance or visual quality.

## 3. The "So What?" (Business Value)
*   **Market Reach**: Opens the framework to the significant Linux developer and user market.
*   **True Cross-Platform**: Fulfills the promise of a cross-platform Rust GUI framework.
*   **Developer Experience**: Allows developers working on Linux to build and test their GUI applications natively.

## 4. Success Metrics
*   **Compatibility**: `App::run` successfully launches on standard Ubuntu (GNOME/Wayland) and a major X11 environment (e.g., XFCE or older Ubuntu).
*   **Performance**: Frame rates and rendering latency are comparable to the macOS/Windows backends.
*   **Input Handling**: Full support for keyboard, mouse, and scroll events matching the behavior on other platforms.

## 5. Acceptance Criteria
✅ **Acceptance Criteria:**
- The `plat-core` crate must implement a Linux-specific platform backend utilizing `winit` for Wayland and X11 support.
- The `render-engine` must successfully create a `wgpu` surface and render the scene graph on Linux display servers.
- The `echo_quickstart` example must display "Hello, World!" and "Click Me" in a native graphical window on a supported Linux environment when executed with `cargo run`.
- Standard input events (pointer down/up/move, keyboard strokes, window resizing) must be correctly routed to the application state.
- The framework must handle missing dependencies gracefully, providing clear error messages rather than raw panics if underlying libraries (like `libxkbcommon` or Wayland protocols) are absent.

## 6. Out of Scope (Phase 1)
🚫 **Out of Scope:**
- Advanced platform-specific window features (like setting custom backdrop materials similar to Windows Mica/Acrylic).
- Client-Side Decoration (CSD) beyond basic window borders and title bars provided by the window manager.
- Support for obscure or legacy Linux window managers outside of standard Wayland/X11 environments.
