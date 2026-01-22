# DirectComposition Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable selective transparency (Mica in sidebar, solid in content area) by integrating DirectComposition to composite wgpu output onto a DWM visual tree.

**Architecture:** Create a DirectComposition device and visual tree that layers backdrop materials behind wgpu's swap chain output. Replace standard wgpu swap chains with composition swap chains that render to DirectComposition surfaces. Expose a high-level API for setting per-region materials.

**Tech Stack:**
- Windows DirectComposition COM APIs (IDCompositionDevice3, IDCompositionDesktopDevice, IDCompositionVisual2)
- DXGI composition swap chains (IDXGISwapChain1 with DXGI_ALPHA_MODE_PREMULTIPLIED)
- wgpu DirectX 12 backend integration
- windows-rs crate for COM interop

**References:**
- [DirectComposition API](https://docs.microsoft.com/en-us/windows/win32/directcomp/directcomposition-portal)
- [Composition swap chains](https://docs.microsoft.com/en-us/windows/win32/api/dcomp/nf-dcomp-idcompositiondevice-createtargetforhwnd)
- [wgpu swap chain creation](https://github.com/gfx-rs/wgpu/blob/trunk/wgpu/src/backend/direct3d12.rs)

---

## Phase 1: COM Infrastructure & DirectComposition Device

### Task 1.1: Add DirectComposition Dependencies

**Files:**
- Modify: `Cargo.toml:65-74`

**Step 1: Add DirectComposition features to workspace windows dependency**

In `Cargo.toml`, add DirectComposition features:

```toml
[workspace.dependencies.windows]
version = "0.59"
features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Controls",
    "Win32_Graphics_Gdi",
    "Win32_Graphics_Dwm",
    "Win32_Graphics_DirectComposition",
    "Win32_Graphics_Dxgi",
    "Win32_Graphics_Dxgi_Common",
    "Win32_System_LibraryLoader",
    "Win32_System_Threading",
    "Win32_System_Registry",
    "Win32_System_Com",
]
```

**Step 2: Verify build**

Run: `cargo check -p plat-core`
Expected: Success

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "feat(deps): Add DirectComposition COM dependencies"
```

---

### Task 1.2: Create DirectComposition Device Wrapper

**Files:**
- Create: `crates/plat-core/src/platform/windows/composition.rs`
- Modify: `crates/plat-core/src/platform/windows.rs:1-30`

**Step 1: Write test for DirectComposition device creation**

Create `crates/plat-core/src/platform/windows/composition.rs`:

```rust
//! DirectComposition integration for selective transparency effects.
//!
//! This module provides a wrapper around Windows DirectComposition COM APIs
//! to enable per-region backdrop materials (Mica, Acrylic) with wgpu rendering.

#![cfg(target_os = "windows")]

use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{
            DirectComposition::*,
            Dxgi::Common::*,
            Dxgi::*,
        },
        System::Com::*,
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_device_creation() {
        // COM must be initialized for DirectComposition
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

            let device = CompositionDevice::new();
            assert!(device.is_ok(), "Failed to create DirectComposition device");

            CoUninitialize();
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p plat-core composition_device_creation`
Expected: FAIL with "CompositionDevice not found"

**Step 3: Implement CompositionDevice wrapper**

Add to `composition.rs`:

```rust
/// Wrapper around IDCompositionDesktopDevice for creating composition visuals.
pub struct CompositionDevice {
    device: IDCompositionDesktopDevice,
}

impl CompositionDevice {
    /// Creates a new DirectComposition device.
    ///
    /// # Safety
    /// COM must be initialized before calling this function.
    pub fn new() -> Result<Self> {
        unsafe {
            // Create D3D11 device for DirectComposition
            let dxgi_device: IDXGIDevice = create_dxgi_device()?;

            // Create DirectComposition desktop device
            let device: IDCompositionDesktopDevice =
                DCompositionCreateDevice3(&dxgi_device)?;

            Ok(Self { device })
        }
    }

    /// Returns the underlying IDCompositionDesktopDevice.
    pub fn raw_device(&self) -> &IDCompositionDesktopDevice {
        &self.device
    }
}

/// Creates a DXGI device for DirectComposition.
///
/// DirectComposition requires a DXGI device (from D3D11 or D3D12) to create
/// composition surfaces. We use D3D11 here as it's lighter-weight and
/// DirectComposition is API-agnostic.
unsafe fn create_dxgi_device() -> Result<IDXGIDevice> {
    use windows::Win32::Graphics::Direct3D::*;
    use windows::Win32::Graphics::Direct3D11::*;

    let mut device = None;
    let feature_levels = [D3D_FEATURE_LEVEL_11_0];

    D3D11CreateDevice(
        None, // Use default adapter
        D3D_DRIVER_TYPE_HARDWARE,
        None, // No software rasterizer
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, // Required for DirectComposition
        Some(&feature_levels),
        D3D11_SDK_VERSION,
        Some(&mut device),
        None,
        None,
    )?;

    let d3d_device = device.ok_or_else(|| Error::from(E_FAIL))?;
    d3d_device.cast::<IDXGIDevice>()
}
```

**Step 4: Add D3D11 dependency**

In `Cargo.toml`, add to windows features:

```toml
    "Win32_Graphics_Direct3D",
    "Win32_Graphics_Direct3D11",
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p plat-core composition_device_creation`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/plat-core/src/platform/windows/composition.rs Cargo.toml
git commit -m "feat(composition): Add DirectComposition device wrapper"
```

---

### Task 1.3: Create Composition Target for Window

**Files:**
- Modify: `crates/plat-core/src/platform/windows/composition.rs`

**Step 1: Write test for composition target creation**

Add to `composition.rs` tests:

```rust
#[test]
fn test_composition_target_creation() {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        // Create a test window (minimal Win32 window)
        let hwnd = create_test_window();

        let device = CompositionDevice::new().unwrap();
        let target = device.create_target_for_hwnd(hwnd);
        assert!(target.is_ok(), "Failed to create composition target");

        DestroyWindow(hwnd);
        CoUninitialize();
    }
}

// Helper to create minimal test window
unsafe fn create_test_window() -> HWND {
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::System::LibraryLoader::*;

    let class_name = w!("TestWindow");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(DefWindowProcW),
        lpszClassName: class_name,
        hInstance: GetModuleHandleW(None).unwrap().into(),
        ..Default::default()
    };
    RegisterClassW(&wc);

    CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        class_name,
        w!("Test"),
        WS_OVERLAPPEDWINDOW,
        0, 0, 100, 100,
        None, None,
        GetModuleHandleW(None).unwrap(),
        None,
    ).unwrap()
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p plat-core composition_target`
Expected: FAIL with "create_target_for_hwnd not found"

**Step 3: Implement composition target creation**

Add to `CompositionDevice` impl:

```rust
impl CompositionDevice {
    // ... existing new() ...

    /// Creates a composition target bound to a window handle.
    ///
    /// This target is where the composition visual tree will be rendered.
    pub fn create_target_for_hwnd(&self, hwnd: HWND) -> Result<CompositionTarget> {
        unsafe {
            let target = self.device.CreateTargetForHwnd(hwnd, true)?;
            Ok(CompositionTarget { target })
        }
    }
}

/// Wrapper around IDCompositionTarget for a window.
pub struct CompositionTarget {
    target: IDCompositionTarget,
}

impl CompositionTarget {
    /// Sets the root visual for this target.
    pub fn set_root(&self, visual: &CompositionVisual) -> Result<()> {
        unsafe {
            self.target.SetRoot(&visual.visual)?;
            Ok(())
        }
    }

    /// Returns the underlying IDCompositionTarget.
    pub fn raw_target(&self) -> &IDCompositionTarget {
        &self.target
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p plat-core composition_target`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/plat-core/src/platform/windows/composition.rs
git commit -m "feat(composition): Add composition target for HWND"
```

---

## Phase 2: Visual Tree & Swap Chain Integration

### Task 2.1: Create Composition Visual Wrapper

**Files:**
- Modify: `crates/plat-core/src/platform/windows/composition.rs`

**Step 1: Write test for visual creation**

Add to tests:

```rust
#[test]
fn test_composition_visual_creation() {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        let device = CompositionDevice::new().unwrap();
        let visual = device.create_visual();
        assert!(visual.is_ok(), "Failed to create composition visual");

        CoUninitialize();
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p plat-core composition_visual`
Expected: FAIL

**Step 3: Implement composition visual**

Add to `composition.rs`:

```rust
impl CompositionDevice {
    // ... existing methods ...

    /// Creates a new composition visual.
    pub fn create_visual(&self) -> Result<CompositionVisual> {
        unsafe {
            let visual = self.device.CreateVisual()?;
            Ok(CompositionVisual { visual })
        }
    }

    /// Creates a composition surface from a DXGI swap chain.
    ///
    /// This allows wgpu's swap chain output to be used in the composition tree.
    pub fn create_surface_from_swap_chain(
        &self,
        swap_chain: &IDXGISwapChain1,
    ) -> Result<CompositionSurface> {
        unsafe {
            let surface = self.device.CreateSurfaceFromSwapChain(swap_chain)?;
            Ok(CompositionSurface { surface })
        }
    }
}

/// Wrapper around IDCompositionVisual2 for the visual tree.
pub struct CompositionVisual {
    visual: IDCompositionVisual2,
}

impl CompositionVisual {
    /// Sets the content of this visual to a composition surface.
    pub fn set_content(&self, surface: &CompositionSurface) -> Result<()> {
        unsafe {
            self.visual.SetContent(&surface.surface)?;
            Ok(())
        }
    }

    /// Sets the size of this visual.
    pub fn set_size(&self, width: f32, height: f32) -> Result<()> {
        unsafe {
            self.visual.SetOffsetX(0.0)?;
            self.visual.SetOffsetY(0.0)?;
            // Note: Size is implicit from content, but we can clip if needed
            Ok(())
        }
    }

    /// Adds a child visual.
    pub fn add_child(&self, child: &CompositionVisual) -> Result<()> {
        unsafe {
            let children = self.visual.Children()?;
            children.InsertAtTop(&child.visual)?;
            Ok(())
        }
    }

    /// Returns the underlying IDCompositionVisual2.
    pub fn raw_visual(&self) -> &IDCompositionVisual2 {
        &self.visual
    }
}

/// Wrapper around IDCompositionSurface.
pub struct CompositionSurface {
    surface: IUnknown, // IDCompositionSurface is not well-defined in windows-rs
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p plat-core composition_visual`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/plat-core/src/platform/windows/composition.rs
git commit -m "feat(composition): Add visual tree management"
```

---

### Task 2.2: Create Composition Swap Chain Helper

**Files:**
- Modify: `crates/render-engine/src/backend/wgpu_backend.rs:75-180`
- Create: `crates/render-engine/src/backend/composition_swap_chain.rs`

**Step 1: Write test for composition swap chain configuration**

Create `crates/render-engine/src/backend/composition_swap_chain.rs`:

```rust
//! DirectComposition-compatible swap chain configuration.
//!
//! Provides utilities for creating DXGI swap chains compatible with
//! DirectComposition visual trees.

#![cfg(target_os = "windows")]

use wgpu::hal::dx12;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_swap_chain_desc() {
        let desc = create_composition_swap_chain_desc(800, 600);

        // Verify composition-compatible settings
        assert_eq!(desc.width, 800);
        assert_eq!(desc.height, 600);
        assert_eq!(desc.format, wgpu::TextureFormat::Bgra8UnormSrgb);
        // Alpha mode should be premultiplied for DirectComposition
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p render-engine composition_swap_chain`
Expected: FAIL

**Step 3: Implement composition swap chain descriptor**

Add to `composition_swap_chain.rs`:

```rust
/// Creates a surface configuration compatible with DirectComposition.
///
/// DirectComposition requires:
/// - BGRA8 format (not RGBA8)
/// - PreMultiplied alpha mode
/// - Flip swap effect
pub fn create_composition_surface_config(
    width: u32,
    height: u32,
    present_mode: wgpu::PresentMode,
) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb, // DirectComposition requires BGRA
        width,
        height,
        present_mode,
        alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied, // Required for DWM composition
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}

/// Checks if a surface supports DirectComposition.
pub fn supports_composition(caps: &wgpu::SurfaceCapabilities) -> bool {
    caps.formats.contains(&wgpu::TextureFormat::Bgra8UnormSrgb)
        && caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p render-engine composition_swap_chain`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/render-engine/src/backend/composition_swap_chain.rs
git commit -m "feat(render): Add DirectComposition swap chain utilities"
```

---

## Phase 3: Integration with WindowImpl

### Task 3.1: Add Composition Mode to WindowImpl

**Files:**
- Modify: `crates/plat-core/src/platform/windows.rs:78-87`

**Step 1: Write test for composition mode**

Add to `crates/plat-core/src/platform/windows.rs` (in the test module):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_composition_mode() {
        // Create window with composition mode enabled
        let config = WindowConfig {
            title: "Test".into(),
            size: Size::new(800, 600),
            composition_mode: true,
            ..Default::default()
        };

        // This would require full window creation infrastructure
        // For now, just verify the config field exists
        assert!(config.composition_mode);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p plat-core window_composition`
Expected: FAIL with "composition_mode field not found"

**Step 3: Add composition_mode to WindowConfig**

In `crates/plat-core/src/window.rs`, modify:

```rust
pub struct WindowConfig {
    pub title: String,
    pub size: Size<u32>,
    pub position: Option<Position<i32>>,
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub visible: bool,
    /// Enable DirectComposition mode for selective transparency.
    ///
    /// When true, the window uses DirectComposition visual trees instead of
    /// standard wgpu swap chains. This enables per-region backdrop materials
    /// but requires Windows 10 version 1803 or newer.
    pub composition_mode: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Arthropod".into(),
            size: Size::new(800, 600),
            position: None,
            resizable: true,
            decorations: true,
            transparent: false,
            visible: true,
            composition_mode: false, // Opt-in for now
        }
    }
}
```

**Step 4: Store composition state in WindowImpl**

Modify `WindowImpl`:

```rust
pub struct WindowImpl {
    hwnd: HWND,
    #[allow(dead_code)]
    hinstance: HINSTANCE,
    id: WindowId,
    backdrop_material: AtomicU8,
    /// DirectComposition integration (only if composition_mode enabled)
    composition: Option<WindowComposition>,
}

/// DirectComposition state for a window.
struct WindowComposition {
    device: CompositionDevice,
    target: CompositionTarget,
    root_visual: CompositionVisual,
}
```

**Step 5: Initialize composition in WindowImpl::new**

Modify `WindowImpl::new`:

```rust
impl WindowImpl {
    fn new(hinstance: HINSTANCE, config: WindowConfig) -> Result<Self, PlatformError> {
        unsafe {
            // ... existing window creation code ...

            let composition = if config.composition_mode {
                // Initialize COM for DirectComposition
                CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

                let device = CompositionDevice::new()
                    .map_err(|e| PlatformError::Initialization(format!("DirectComposition device: {}", e)))?;
                let target = device.create_target_for_hwnd(hwnd)
                    .map_err(|e| PlatformError::Initialization(format!("Composition target: {}", e)))?;
                let root_visual = device.create_visual()
                    .map_err(|e| PlatformError::Initialization(format!("Root visual: {}", e)))?;

                target.set_root(&root_visual)
                    .map_err(|e| PlatformError::Initialization(format!("Set root: {}", e)))?;

                Some(WindowComposition {
                    device,
                    target,
                    root_visual,
                })
            } else {
                None
            };

            Ok(Self {
                hwnd,
                hinstance,
                id,
                backdrop_material: AtomicU8::new(0),
                composition,
            })
        }
    }
}
```

**Step 6: Run test to verify it passes**

Run: `cargo test -p plat-core`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/plat-core/src/platform/windows.rs crates/plat-core/src/window.rs
git commit -m "feat(window): Add DirectComposition mode support"
```

---

## Phase 4: Selective Material API

### Task 4.1: Design Material Region API

**User Input Required:** Design the API for specifying material regions.

I've set up the composition infrastructure. Now we need to decide the API for developers to specify which regions have which materials. This is a UX/API design decision with trade-offs:

**Option A: Scene-node based (declarative)**
```rust
scene.add_node_with_material(parent, node, BackdropMaterial::Mica);
```
- Pro: Ties material to scene hierarchy
- Con: Couples rendering to scene graph

**Option B: Region-based (imperative)**
```rust
compositor.set_material(Rect::new(0, 0, 200, 600), BackdropMaterial::Mica);
```
- Pro: Decoupled from scene
- Con: Manual region management

**Option C: Layer-based (hybrid)**
```rust
let layer = compositor.create_layer(BackdropMaterial::Mica);
layer.add_content(rect);
```
- Pro: Flexible layering
- Con: More complex API

In `crates/plat-core/src/composition.rs`, implement your preferred approach in the `CompositorApi` trait (I've created the file with stub and comments).

Consider: How will developers typically use this? Most apps have 2-3 regions (sidebar, header, content). Which API makes that common case easiest?

**File prepared:** `crates/plat-core/src/composition.rs` with trait stub and documentation comments.

---

### Task 4.2: Implement Backdrop Visual Creation

**Files:**
- Modify: `crates/plat-core/src/platform/windows/composition.rs`

**Step 1: Write test for backdrop brush**

Add to tests:

```rust
#[test]
fn test_backdrop_brush_creation() {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        let device = CompositionDevice::new().unwrap();
        let brush = device.create_backdrop_brush(BackdropMaterial::Mica);
        assert!(brush.is_ok(), "Failed to create backdrop brush");

        CoUninitialize();
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p plat-core backdrop_brush`
Expected: FAIL

**Step 3: Implement backdrop brush creation**

Add to `CompositionDevice`:

```rust
impl CompositionDevice {
    // ... existing methods ...

    /// Creates a backdrop brush for a visual.
    ///
    /// This creates a visual that samples the content behind it with the
    /// specified material effect (Mica, Acrylic, etc.).
    pub fn create_backdrop_visual(
        &self,
        material: BackdropMaterial,
    ) -> Result<CompositionVisual> {
        unsafe {
            let visual = self.create_visual()?;

            // Apply backdrop effect based on material
            match material {
                BackdropMaterial::Mica => {
                    // Use system backdrop brush
                    let brush: IDCompositionVisual2 = visual.visual.cast()?;
                    // Note: Actual backdrop effect API varies by Windows version
                    // Windows 11 22H2+ has SystemBackdrop APIs
                    // Fallback to blur for older versions
                }
                BackdropMaterial::Acrylic => {
                    // Apply Acrylic blur effect
                    // This requires a blur effect + tint
                }
                _ => {}
            }

            Ok(visual)
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p plat-core backdrop_brush`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/plat-core/src/platform/windows/composition.rs
git commit -m "feat(composition): Add backdrop visual creation"
```

---

## Phase 5: wgpu Integration

### Task 5.1: Modify WgpuBackend for Composition Mode

**Files:**
- Modify: `crates/render-engine/src/backend/wgpu_backend.rs:83-180`

**Step 1: Add composition_mode parameter to WgpuBackend::new**

Modify function signature:

```rust
impl WgpuBackend {
    /// Creates a new wgpu backend.
    ///
    /// If `composition_mode` is true, creates a swap chain compatible with
    /// DirectComposition instead of a standard window surface.
    pub fn new<W>(
        window: &W,
        width: u32,
        height: u32,
        composition_mode: bool,
    ) -> Result<Self, RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle + Sync,
    {
        // ... implementation ...
    }
}
```

**Step 2: Branch on composition_mode in surface configuration**

In `WgpuBackend::new`, modify surface configuration:

```rust
let config = if composition_mode {
    // Use BGRA format and premultiplied alpha for DirectComposition
    composition_swap_chain::create_composition_surface_config(
        width,
        height,
        surface_caps.present_modes[0],
    )
} else {
    // Standard surface configuration
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
};
```

**Step 3: Update all call sites**

Update call sites in:
- `examples/native_materials.rs`: Pass `config.composition_mode`
- `examples/colored_rectangles.rs`: Pass `false`
- Other examples

**Step 4: Test compilation**

Run: `cargo build --all`
Expected: Success

**Step 5: Commit**

```bash
git add crates/render-engine/src/backend/wgpu_backend.rs examples/
git commit -m "feat(render): Add composition mode to wgpu backend"
```

---

## Phase 6: Example & Documentation

### Task 6.1: Create DirectComposition Example

**Files:**
- Create: `examples/directcomposition_demo.rs`
- Modify: `Cargo.toml` (add example entry)

**Step 1: Create example with Mica sidebar**

Create `examples/directcomposition_demo.rs`:

```rust
//! Example: DirectComposition with Selective Transparency
//!
//! Demonstrates per-region backdrop materials using DirectComposition.
//! Shows a Mica sidebar with solid content area.

use plat_core::{
    Application, BackdropMaterial, ControlFlow, Event, EventLoop,
    Rect, Size, Window, WindowConfig, WindowEvent, WindowId,
};
use render_engine::{
    Color, NodeContent, Scene, SceneNode,
    backend::{RenderBackend, WgpuBackend},
};

struct CompositionApp {
    window: Window,
    backend: WgpuBackend,
    scene: Scene,
}

impl Application for CompositionApp {
    fn new(event_loop: &EventLoop) -> Self {
        println!("=== DirectComposition Demo ===");
        println!("Mica sidebar + solid content area");

        // Enable DirectComposition mode
        let config = WindowConfig {
            title: "DirectComposition Demo".to_string(),
            size: Size::new(1000, 700),
            composition_mode: true, // Key: enable DirectComposition
            transparent: true,
            visible: true,
            ..Default::default()
        };

        let window = event_loop
            .create_window(config)
            .expect("Failed to create window");

        // Create wgpu backend in composition mode
        let size = window.inner_size();
        let backend = WgpuBackend::new(&window, size.width, size.height, true)
            .expect("Failed to create backend");

        // TODO: Set up compositor with regions
        // compositor.set_material(Rect::new(0, 0, 200, 700), BackdropMaterial::Mica);

        let scene = create_demo_scene();

        Self {
            window,
            backend,
            scene,
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        if let Err(e) = self.backend.render(&self.scene) {
            eprintln!("Render error: {}", e);
        }
    }
}

fn create_demo_scene() -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    // Sidebar region (Mica backdrop will be behind this)
    let sidebar = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(1.0, 1.0, 1.0, 0.3), // Semi-transparent
    });
    let sidebar_id = scene.add_node(root, sidebar);
    if let Some(node) = scene.get_node_mut(sidebar_id) {
        node.bounds = Rect::new(0.0, 0.0, 200.0, 700.0);
    }

    // Content region (solid background)
    let content = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(0.95, 0.95, 0.95, 1.0), // Opaque
    });
    let content_id = scene.add_node(root, content);
    if let Some(node) = scene.get_node_mut(content_id) {
        node.bounds = Rect::new(200.0, 0.0, 800.0, 700.0);
    }

    scene
}

fn main() {
    env_logger::init();
    plat_core::run::<CompositionApp>().expect("Failed to run");
}
```

**Step 2: Add example to Cargo.toml**

```toml
[[example]]
name = "directcomposition_demo"
path = "examples/directcomposition_demo.rs"
```

**Step 3: Test compilation**

Run: `cargo build --example directcomposition_demo`
Expected: Success (even if compositor API incomplete)

**Step 4: Commit**

```bash
git add examples/directcomposition_demo.rs Cargo.toml
git commit -m "feat(examples): Add DirectComposition demo"
```

---

### Task 6.2: Document Architecture Decision

**Files:**
- Create: `docs/adr/0006-directcomposition-integration.md`

**Step 1: Create ADR document**

Create ADR with architecture rationale:

```markdown
# ADR 0006: DirectComposition Integration for Selective Transparency

## Status

Accepted

## Context

Option 1 (window-vibrancy) provides full-window Mica/Acrylic effects, but cannot
support selective transparency (Mica in sidebar, solid in content area). Native
Windows apps use DirectComposition to layer backdrop materials behind specific
regions of the UI.

## Decision

Integrate Windows DirectComposition to create a visual tree that layers backdrop
materials behind wgpu's swap chain output. This requires:

1. **Composition Device**: Create IDCompositionDesktopDevice for visual tree management
2. **Composition Swap Chains**: Use DXGI swap chains with composition-compatible settings
3. **Visual Tree**: Layer backdrop visuals behind content visuals
4. **API Design**: Provide high-level API for setting per-region materials

## Consequences

### Positive

- Enables per-region backdrop control (Mica sidebar, solid content)
- Proper integration with Windows compositor
- Future-proof for Windows UI evolution
- Professional-grade transparency effects

### Negative

- Windows-only feature (macOS has different approach)
- Increased complexity vs. window-vibrancy
- Requires Windows 10 1803+ for DirectComposition3
- More COM marshalling overhead

### Implementation Notes

- Keep window-vibrancy as fallback for older Windows
- Abstract compositor API for future macOS support
- Document performance characteristics
- Provide opt-in via WindowConfig::composition_mode

## Alternatives Considered

1. **window-vibrancy only**: Simple but no selective transparency
2. **Bypass wgpu entirely**: Too much maintenance burden
3. **Layer system with masks**: Complex and less performant

## References

- [DirectComposition Overview](https://docs.microsoft.com/en-us/windows/win32/directcomp/directcomposition-portal)
- [Composition Swap Chains](https://docs.microsoft.com/en-us/windows/win32/api/dcomp/nf-dcomp-idcompositiondevice-createtargetforhwnd)
```

**Step 2: Commit**

```bash
git add docs/adr/0006-directcomposition-integration.md
git commit -m "docs(adr): Add DirectComposition integration ADR"
```

---

## Testing & Verification

### Task 7.1: Integration Test

**Files:**
- Create: `crates/plat-core/tests/directcomposition_tests.rs`

Write integration test that:
1. Creates window with composition_mode=true
2. Verifies DirectComposition device initialized
3. Tests visual tree creation
4. Tests swap chain integration

### Task 7.2: Visual Verification

Run examples and verify:
1. `directcomposition_demo` shows Mica sidebar
2. Desktop wallpaper visible through sidebar
3. Content area remains opaque
4. Performance remains acceptable (60fps)

---

## Rollout Plan

1. **Phase 1 (Weeks 1-2)**: Implement COM infrastructure and composition device
2. **Phase 2 (Week 3)**: Integrate with WindowImpl and visual tree
3. **Phase 3 (Week 4)**: wgpu backend modifications and swap chain integration
4. **Phase 4 (Week 5)**: Material API design and implementation (with user input)
5. **Phase 5 (Week 6)**: Examples, documentation, and polish
6. **Phase 6 (Week 7)**: Testing, performance optimization, and stabilization

**Estimated Total**: 6-7 weeks for full implementation and testing

---

## Success Criteria

- [ ] Mica sidebar with solid content area works
- [ ] Desktop wallpaper visible through Mica regions
- [ ] Performance: 60fps with 1000+ widgets
- [ ] All tests pass
- [ ] Clippy clean
- [ ] Documentation complete
- [ ] Fallback to window-vibrancy works on older Windows

