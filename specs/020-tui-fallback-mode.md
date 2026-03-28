# 🔭 Vantage: Spec for TUI Fallback Mode

**Status:** Draft
**Owner:** Vantage (Product)
**Date:** 2026-05-23

## 1. Problem Statement
Currently, `App::run` requires Windows or macOS to initialize the `wgpu` graphical context. On unsupported platforms like Linux or Headless Cloud IDEs, the Quick Start examples fail with an error or panic. This creates poor Developer Experience (DX) for users trying to evaluate the framework.

## 2. User Story
👤 **User Story:** As a Developer evaluating Arthropod on Linux or inside a Cloud IDE (like GitHub Codespaces or a sandbox), I want the application to automatically fall back to a Terminal User Interface (TUI) when graphical window creation fails, so that I can still run examples, test my logic, and see a basic UI representation without crashing.

## 3. Solution Overview
We will implement an automatic TUI fallback for `App::run`.

### Core Components
1.  **Platform Detection**: Enhance `plat-core` to detect if graphical context creation fails (e.g., no display server).
2.  **TUI Backend Initialization**: If `wgpu`/`winit` fails, initialize a `ratatui` + `crossterm` backend automatically.
3.  **Widget Mapping**: The `render-engine` needs a way to map standard UI primitives (text, borders, colors) to ANSI terminal blocks.

## 4. Acceptance Criteria
✅ **Acceptance Criteria:**
- `App::run` must gracefully handle the `winit` failure on unsupported platforms.
- Must initialize a `ratatui` terminal output without the user explicitly calling `App::new_headless()`.
- The `echo_quickstart` example must display "Hello, World!" and "Click Me" in the terminal without changing the `main.rs` code.
- Terminal resize events must correctly update the application's layout engine.
- Keyboard input must correctly route to the focused widget (e.g., selecting a button).

## 5. Out of Scope (Phase 1)
🚫 **Out of Scope:**
- Rendering complex graphical widgets (like `Image`, `Video`, or 3D objects) as ASCII art.
- Full mouse support in the terminal (fallback to keyboard navigation first).
- Exact pixel-perfect matching of fonts/sizes (since terminal grids are discrete).
