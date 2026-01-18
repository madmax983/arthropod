# ADR 0005: Platform Abstraction Strategy (plat-core)

**Status:** Accepted

**Date:** 2026-01-17

**Deciders:** Architecture Team

## Context

Arthropod needs a platform abstraction layer that enables cross-platform GUI development across Windows, macOS, Linux, iOS, Android, and Web. Key requirements:

1. **Thin Abstraction**: Don't hide platform capabilities
2. **Raw GPU Access**: Higher layers choose rendering strategy
3. **Native Feel**: Expose platform differences when needed
4. **Unified Events**: Common input model across all platforms
5. **System Integration**: Clipboard, drag-drop, dialogs, notifications
6. **Performance**: Minimal overhead, direct API calls where possible

## Decision

We implement `plat-core` as a **thin, capability-exposing abstraction** rather than a lowest-common-denominator API.

### Architecture

```
┌─────────────────────────────────────────────┐
│  Application Code (arthropod framework)    │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│  plat-core (Platform Abstraction Layer)    │
│  - Window management (creation, events)    │
│  - Input normalization (keyboard, mouse,   │
│    touch, pen, gamepad)                     │
│  - GPU surface (raw handles for wgpu)      │
│  - System integration (clipboard, dialogs) │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│  Platform-Specific Implementations          │
│  - Windows (Win32 + winit)                 │
│  - macOS (Cocoa + winit)                   │
│  - Linux (GTK/Wayland + winit)             │
│  - iOS (UIKit + winit)                     │
│  - Android (NativeActivity + winit)        │
│  - Web (DOM/Canvas + winit/web-sys)        │
└─────────────────────────────────────────────┘
```

### Core Responsibilities

**Window Management**:
```rust
pub struct Window { ... }

impl Window {
    pub fn create(config: WindowConfig) -> Result<Self>;
    pub fn inner_size(&self) -> Size;
    pub fn set_title(&self, title: &str);
    pub fn request_redraw(&self);
    // Platform-specific extensions available via traits
}
```

**Unified Input Model**:
```rust
pub enum Event {
    Window { window_id: WindowId, event: WindowEvent },
    DeviceEvent { device_id: DeviceId, event: DeviceEvent },
}

pub enum WindowEvent {
    Resized(Size),
    CloseRequested,
    Focused(bool),
    CursorMoved { position: PhysicalPosition },
    MouseInput { button: MouseButton, state: ElementState },
    KeyboardInput { key: Key, state: ElementState },
    Touch(Touch),  // Unified touch model
    // ...
}
```

**Raw GPU Access**:
```rust
// Expose raw handles, don't abstract GPU
pub trait HasRawWindowHandle {
    fn raw_window_handle(&self) -> RawWindowHandle;
}

pub trait HasRawDisplayHandle {
    fn raw_display_handle(&self) -> RawDisplayHandle;
}

// Higher layers (render-engine) use these for wgpu
```

**System Integration**:
```rust
pub struct Clipboard;
impl Clipboard {
    pub fn read_text(&self) -> Result<String>;
    pub fn write_text(&self, text: &str) -> Result<()>;
}

pub struct FileDialog;
impl FileDialog {
    pub fn open_file(&self, options: FileDialogOptions) -> Result<Option<PathBuf>>;
    pub fn save_file(&self, options: FileDialogOptions) -> Result<Option<PathBuf>>;
}
```

### Platform Support Matrix

| Platform | Windowing | Graphics | Accessibility | Status |
|----------|-----------|----------|---------------|--------|
| Windows  | Win32     | DX12/Vulkan | UI Automation | ✅ Implemented (Phase 1) |
| macOS    | Cocoa     | Metal    | NSAccessibility | 🚧 Planned (Phase 1) |
| Linux    | GTK/Wayland | Vulkan | AT-SPI      | 🚧 Planned (Phase 1) |
| iOS      | UIKit     | Metal    | UIAccessibility | 📅 Planned (Phase 5) |
| Android  | NativeActivity | Vulkan/GLES | AccessibilityService | 📅 Planned (Phase 5) |
| Web      | DOM/Canvas | WebGPU/WebGL | ARIA    | 📅 Planned (Phase 6) |

## Rationale

### Why Thin Abstraction?

1. **Expose Capabilities**: Platforms have different features (macOS Vibrancy, Windows Mica, iOS safe areas). Don't hide them.

2. **Performance**: Direct platform API calls when possible. No translation layers.

3. **Future-Proof**: New platform APIs can be exposed without redesigning abstraction.

4. **Flexibility**: Higher layers choose rendering strategy (wgpu, skia, native compositor).

### Why winit as Foundation?

**Pros:**
- Battle-tested in production (Bevy, Alacritty, etc.)
- Excellent platform support (6 platforms)
- Raw window handle support (for wgpu)
- Active development, responsive maintainers
- Pure Rust, no unsafe foreign bindings for core logic

**Cons:**
- Some platform quirks (event loop design)
- Opinionated event model (adapt-able)

**Decision**: Use winit as foundation, extend with platform-specific modules for features winit doesn't cover.

### Why Not Alternatives?

**1. SDL2/SDL3**
- **Pros**: Mature, comprehensive, game-focused
- **Cons**: C library (unsafe bindings), game-centric API, larger overhead
- **Rejected**: Prefer pure Rust for safety and ergonomics

**2. GLFW**
- **Pros**: Lightweight, OpenGL-focused
- **Cons**: C library, older design, less platform integration
- **Rejected**: winit is more modern and Rust-native

**3. Custom Per-Platform**
- **Pros**: Full control, optimal performance
- **Cons**: Massive effort (3+ different APIs), maintenance burden
- **Rejected**: winit provides 80% of what we need

**4. tauri/wry**
- **Pros**: Web-based rendering
- **Cons**: Not native rendering, webview overhead
- **Rejected**: Different use case (web views vs native GPU rendering)

## Consequences

### Positive

- **Thin API**: Minimal abstraction overhead
- **Platform Respect**: Can expose platform-specific features
- **Raw GPU Access**: Higher layers control rendering
- **Pure Rust**: Memory safety, no C++ bindings for core
- **Battle-Tested**: winit used in production GUI apps and games
- **Maintainable**: ~6K lines of code vs ~30K for custom implementation

### Negative

- **Platform Differences**: Apps must handle some platform variations
- **winit Dependency**: Tied to winit's event model and quirks
- **Not Fully Featured**: Some OS features require custom extensions
- **Async Model**: winit's event loop design can be restrictive

### Mitigations

- **Extension Traits**: Platform-specific features via traits
- **Event Normalization**: plat-core smooths over winit quirks
- **Documentation**: Clear guidance on platform differences
- **Escape Hatches**: Access to raw handles when needed

## Implementation Status

### Phase 1 (Complete)
- ✅ Window creation (Windows)
- ✅ Event loop and input handling
- ✅ Raw window handle exposure
- ✅ Basic windowing (resize, close, etc.)

### Phase 1 (Remaining)
- 🚧 macOS and Linux support
- 🚧 Clipboard integration
- 🚧 File dialogs
- 🚧 System tray (optional)

### Future Phases
- Phase 5: iOS and Android support
- Phase 6: Web support (DOM + Canvas)

## Platform-Specific Extensions

We expose platform differences via extension traits:

```rust
#[cfg(target_os = "windows")]
pub trait WindowsExt {
    fn enable_mica(&self) -> Result<()>;
    fn set_titlebar_color(&self, color: Color);
}

#[cfg(target_os = "macos")]
pub trait MacOSExt {
    fn set_vibrancy(&self, material: VibrancyMaterial) -> Result<()>;
    fn set_titlebar_appearance(&self, appearance: TitlebarAppearance);
}

#[cfg(target_os = "ios")]
pub trait IOSExt {
    fn safe_area_insets(&self) -> EdgeInsets;
    fn set_status_bar_style(&self, style: StatusBarStyle);
}
```

## Testing Strategy

**Unit Tests**: Mock window/event APIs for platform-independent tests

**Integration Tests**: Platform-specific tests on each target

**Manual Testing**: Visual testing on physical devices

**CI**: Test on Windows (GitHub Actions), macOS (GitHub Actions), Linux (Docker)

## Performance Characteristics

- **Event Handling**: Sub-microsecond overhead for event dispatch
- **Window Creation**: < 50ms on typical hardware
- **Input Latency**: Direct platform API calls (minimal overhead)
- **Memory**: ~1-2 KB per window (platform handles)

## References

- winit documentation: https://docs.rs/winit/
- raw-window-handle: https://docs.rs/raw-window-handle/
- Platform Documentation:
  - Windows: Win32 windowing API
  - macOS: Cocoa NSWindow
  - Linux: GTK/Wayland windowing
- [Arthropod Design Doc](../design/arthropod-design-doc.md)

## Future Considerations

- **Accessibility**: Platform bridges for screen readers (AT-SPI, UI Automation, NSAccessibility)
- **System Integration**: Drag-drop, notifications, system tray
- **Mobile Gestures**: Multi-touch, force touch, gesture recognizers
- **Web Deployment**: Canvas + WebGPU via winit's web-sys integration
