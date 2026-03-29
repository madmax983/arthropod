# 🔭 Vantage: Spec for macOS Support

**Status**: Draft
**Owner**: Vantage
**Target Release**: Arthropod Core (Phase 3)

## 1. Context & Problem
Arthropod currently provides solid support for Windows. However, macOS support is incomplete, especially regarding platform-native UI elements, styling, and window materials. For Arthropod to be a true enterprise-grade, cross-platform UI framework, it must offer native-feeling applications on macOS that integrate deeply with macOS design language and window management, such as the vibrant backdrop material and native window controls.

## 2. User Story
**As a** Mac User,
**I want** applications built with Arthropod to look and feel like native macOS applications (with vibrant backdrops, standard window controls, and proper retina rendering),
**So that** I have a seamless, high-quality experience that matches my OS environment without uncanny-valley custom window decorations.

**As a** Mac Developer,
**I want** to compile and run my Arthropod applications on macOS with full feature parity,
**So that** I can confidently ship cross-platform products with a single codebase.

## 3. The "So What?" (Business Value)
*   **Market Reach**: Extends the framework's addressable market to macOS users, who represent a significant portion of designers, developers, and premium software consumers.
*   **Trust & Quality**: Native feeling apps (vibrancy, rounded corners, native shadows) build user trust. Non-native apps are often perceived as "cheap" or "clunky".
*   **Developer Experience (DX)**: Allows developers on Macs to build and test locally without virtualization.

## 4. Success Metrics
*   **Performance**: Maintain 60fps on macOS for 10,000+ widgets.
*   **Visual Parity**: 100% rendering parity of standard widgets between macOS and Windows.
*   **Native Integration**: Support for macOS window vibrancy/blur effects with zero visual glitches.

## 5. Acceptance Criteria
*   The framework **must** support allowing windows to use native macOS blurred backgrounds (vibrancy).
*   The framework **must** support native macOS window control buttons (traffic lights) and respect their standard positioning.
*   High-DPI (Retina) displays **must** render text and UI elements sharply without scaling blur.
*   The application event loop **must** integrate cleanly with the OS, properly handling lifecycle events (quit, hide, foreground).
*   Input handling **must** correctly map macOS specific modifiers (Command vs Control) across all widgets (e.g., Command+C for copy).

## 6. Gap Analysis
*   **Current State**: Basic rendering is possible. However, the styling layers lack macOS-specific features like native backdrop integration. We currently fall back to solid colors.
*   **Standard Ecosystem**: Other frameworks like Tauri and Zed have solved this by using native OS bindings for visual effects. We need to integrate similar native capabilities into the core platform layer.

## 7. Out of Scope (Phase 1)
*   Touch Bar integration.
*   macOS specific Menu Bar (global top menu) integration (will be handled in a separate Desktop Menu spec).
*   App Store sandboxing and signing features (this is a deployment concern, not framework core).
