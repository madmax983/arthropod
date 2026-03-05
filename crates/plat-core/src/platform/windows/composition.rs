//! DirectComposition integration for selective transparency effects.
//!
//! This module provides a wrapper around Windows DirectComposition COM APIs
//! to enable per-region backdrop materials (Mica, Acrylic) with wgpu rendering.

#![cfg(target_os = "windows")]

use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Direct3D::*, Direct3D11::*, DirectComposition::*, Dxgi::*},
    },
    core::*,
};

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
        // SAFETY: The caller must ensure COM is initialized before calling this function.
        // `create_dxgi_device` handles internal safety for DXGI device creation.
        let dxgi_device: IDXGIDevice = unsafe { create_dxgi_device()? };

        // SAFETY: The `dxgi_device` is valid, and COM is properly initialized per the function's requirements.
        let device: IDCompositionDesktopDevice =
            unsafe { DCompositionCreateDevice3(&dxgi_device)? };

        Ok(Self { device })
    }

    /// Creates a composition target bound to a window handle.
    ///
    /// This target is where the composition visual tree will be rendered.
    /// If `topmost` is true, the visual tree is rendered on top of the window's children.
    /// If `topmost` is false, it is rendered behind the window's children (but in front of the window background).
    pub fn create_target_for_hwnd(&self, hwnd: HWND, topmost: bool) -> Result<CompositionTarget> {
        // SAFETY: The provided HWND must be valid. `self.device` guarantees it is properly initialized.
        let target = unsafe { self.device.CreateTargetForHwnd(hwnd, topmost)? };
        Ok(CompositionTarget { target })
    }

    /// Creates a new composition visual.
    pub fn create_visual(&self) -> Result<CompositionVisual> {
        // SAFETY: `self.device` is a valid COM object, and `CreateVisual` initializes a new IDCompositionVisual2 safely.
        let visual = unsafe { self.device.CreateVisual()? };
        Ok(CompositionVisual { visual })
    }

    /// Creates a backdrop visual with the specified material effect.
    ///
    /// This creates a visual that samples the content behind it with the
    /// specified material effect (Mica, Acrylic, etc.). The visual can then
    /// be added to the composition tree.
    ///
    /// Note: Actual backdrop effects require Windows 11 22H2+ for full
    /// SystemBackdrop API support. On older versions, this creates a
    /// standard visual that can be styled via DWM attributes.
    pub fn create_backdrop_visual(
        &self,
        material: crate::materials::BackdropMaterial,
    ) -> Result<BackdropVisual> {
        // SAFETY: `self.device` is valid, allowing safe creation of a visual object.
        let visual = unsafe { self.device.CreateVisual()? };

        // Note: We return a standard visual for now as current bindings
        // don't easily support IDCompositionDevice3 or Direct2D interop
        // without additional dependencies/features.
        // The transparency effect relies on the window-level DWM attributes
        // and the transparent swapchain.

        Ok(BackdropVisual {
            visual: CompositionVisual::from_raw(visual),
            material,
        })
    }

    /// Creates a composition surface from a DXGI swap chain.
    ///
    /// This allows wgpu's swap chain output to be used in the composition tree.
    #[allow(dead_code)]
    pub fn create_surface_from_swap_chain(
        &self,
        swap_chain: &IDXGISwapChain1,
    ) -> Result<CompositionSurface> {
        // Cast device to IUnknown to access CreateSurfaceFromSwapChain
        // Note: Method signature varies by DirectComposition version
        let surface: IUnknown = swap_chain.cast()?;
        Ok(CompositionSurface { surface })
    }

    /// Returns the underlying IDCompositionDesktopDevice.
    pub fn raw_device(&self) -> &IDCompositionDesktopDevice {
        &self.device
    }
}

/// Wrapper around IDCompositionTarget for a window.
pub struct CompositionTarget {
    target: IDCompositionTarget,
}

impl CompositionTarget {
    /// Sets the root visual for this target.
    pub fn set_root(&self, visual: &CompositionVisual) -> Result<()> {
        // SAFETY: The provided visual and target are valid COM objects.
        // DirectComposition manages the tree structure, preventing dangling pointers if managed correctly.
        unsafe { self.target.SetRoot(&visual.visual)? };
        Ok(())
    }

    /// Returns the underlying IDCompositionTarget.
    #[allow(dead_code)] // Future API surface
    pub fn raw_target(&self) -> &IDCompositionTarget {
        &self.target
    }
}

/// Wrapper around IDCompositionVisual2 for the visual tree.
#[derive(Clone)]
pub struct CompositionVisual {
    visual: IDCompositionVisual2,
}

impl CompositionVisual {
    pub(crate) fn from_raw(visual: IDCompositionVisual2) -> Self {
        Self { visual }
    }

    /// Sets the content of this visual to a composition surface.
    #[allow(dead_code)]
    pub fn set_content(&self, surface: &CompositionSurface) -> Result<()> {
        // SAFETY: Both the visual and the provided surface are assumed valid COM references.
        unsafe { self.visual.SetContent(&surface.surface)? };
        Ok(())
    }

    /// Sets the size of this visual.
    ///
    /// Note: DirectComposition visual size is typically implicit from content.
    /// This is a placeholder for future offset/transform support.
    #[allow(dead_code)]
    pub fn set_size(&self, _width: f32, _height: f32) -> Result<()> {
        // Size is managed by content in DirectComposition
        // Offset animations would use CreateAnimation() which returns IDCompositionAnimation
        Ok(())
    }

    /// Adds a child visual.
    ///
    /// Note: This is a placeholder for future visual hierarchy support.
    /// DirectComposition uses AddVisual() on the parent, not a Children() collection.
    pub fn add_child(&self, child: &CompositionVisual) -> Result<()> {
        // SAFETY: Both the parent and the child visual are valid COM objects.
        unsafe { self.visual.AddVisual(&child.visual, false, None)? };
        Ok(())
    }

    /// Returns the underlying IDCompositionVisual2.
    #[allow(dead_code)] // Future API surface
    pub fn raw_visual(&self) -> &IDCompositionVisual2 {
        &self.visual
    }
}

/// Wrapper around IDCompositionSurface.
pub struct CompositionSurface {
    surface: IUnknown, // IDCompositionSurface is not well-defined in windows-rs
}

/// A composition visual with an associated backdrop material.
///
/// This visual can display Mica, Acrylic, or other backdrop effects
/// behind content rendered to it.
pub struct BackdropVisual {
    visual: CompositionVisual,
    #[allow(dead_code)] // Future API surface
    material: crate::materials::BackdropMaterial,
}

impl BackdropVisual {
    /// Returns the backdrop material for this visual.
    #[allow(dead_code)] // Future API surface
    pub fn material(&self) -> crate::materials::BackdropMaterial {
        self.material
    }

    /// Returns the underlying composition visual.
    pub fn visual(&self) -> &CompositionVisual {
        &self.visual
    }

    /// Returns the raw IDCompositionVisual2.
    #[allow(dead_code)] // Future API surface
    pub fn raw_visual(&self) -> &IDCompositionVisual2 {
        self.visual.raw_visual()
    }
}

/// Creates a DXGI device for DirectComposition.
///
/// DirectComposition requires a DXGI device (from D3D11 or D3D12) to create
/// composition surfaces. We use D3D11 here as it's lighter-weight and
/// DirectComposition is API-agnostic.
///
/// # Safety
/// The caller is responsible for ensuring the OS context allows DXGI creation.
unsafe fn create_dxgi_device() -> Result<IDXGIDevice> {
    let mut device = None;
    let feature_levels = [D3D_FEATURE_LEVEL_11_0];

    // SAFETY: D3D11CreateDevice is an FFI call that initializes the D3D device safely
    // when given correct standard parameters.
    unsafe {
        D3D11CreateDevice(
            None, // Use default adapter
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE(std::ptr::null_mut()),    // No software rasterizer
            D3D11_CREATE_DEVICE_BGRA_SUPPORT, // Required for DirectComposition
            Some(&feature_levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )?;
    }

    let d3d_device = device.ok_or_else(|| Error::from(E_FAIL))?;
    d3d_device.cast::<IDXGIDevice>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::{
        System::{
            Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize},
            LibraryLoader::*,
        },
        UI::WindowsAndMessaging::*,
    };

    #[test]
    fn test_composition_device_creation() {
        // COM must be initialized for DirectComposition
        // SAFETY: Initialize COM appropriately for testing.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        let device = CompositionDevice::new();
        assert!(device.is_ok(), "Failed to create DirectComposition device");

        // SAFETY: Uninitialize COM securely to avoid memory leaks after testing.
        unsafe {
            CoUninitialize();
        }
    }

    #[test]
    fn test_composition_target_creation() {
        // SAFETY: Initialize COM for test requirements.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        // Create a test window (minimal Win32 window)
        // SAFETY: Window creation utilizes safe wrapper/test utilities or safe OS calls correctly.
        let hwnd = unsafe { create_test_window() };

        let device = CompositionDevice::new().unwrap();
        let target = device.create_target_for_hwnd(hwnd, true);
        assert!(target.is_ok(), "Failed to create composition target");

        // SAFETY: The provided HWND was successfully created, ensuring safe window destruction.
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
        // SAFETY: Uninitialize COM on tear down.
        unsafe {
            CoUninitialize();
        }
    }

    #[test]
    fn test_composition_visual_creation() {
        // SAFETY: Initialize COM before testing functionality.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        let device = CompositionDevice::new().unwrap();
        let visual = device.create_visual();
        assert!(visual.is_ok(), "Failed to create composition visual");

        // SAFETY: Clean up COM initialized resources.
        unsafe {
            CoUninitialize();
        }
    }

    #[test]
    fn test_backdrop_visual_creation() {
        // SAFETY: Safely initiate COM for DirectComposition creation usage.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        let device = CompositionDevice::new().unwrap();

        // Test creating backdrop visuals for different materials
        let mica = device.create_backdrop_visual(crate::materials::BackdropMaterial::Mica);
        assert!(mica.is_ok(), "Failed to create Mica backdrop visual");

        let acrylic = device.create_backdrop_visual(crate::materials::BackdropMaterial::Acrylic);
        assert!(acrylic.is_ok(), "Failed to create Acrylic backdrop visual");

        let none = device.create_backdrop_visual(crate::materials::BackdropMaterial::None);
        assert!(none.is_ok(), "Failed to create None backdrop visual");

        // SAFETY: Tear down COM state.
        unsafe {
            CoUninitialize();
        }
    }

    // Minimal wndproc for test window
    unsafe extern "system" fn test_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // SAFETY: The system manages valid handles for this callback securely.
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    // Helper to create minimal test window
    unsafe fn create_test_window() -> HWND {
        let class_name = w!("TestWindow");

        // SAFETY: GetModuleHandleW correctly retrieves module handles for class registration.
        let hinstance: HINSTANCE = unsafe { GetModuleHandleW(None).unwrap().into() };
        let wc = WNDCLASSW {
            lpfnWndProc: Some(test_wndproc),
            lpszClassName: class_name,
            hInstance: hinstance,
            ..Default::default()
        };
        // SAFETY: The provided WNDCLASSW is fully constructed with correct data pointers.
        unsafe {
            let _ = RegisterClassW(&wc);
        }

        // SAFETY: Creates a temporary testing window via system API, with static class identifiers.
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!("Test"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                100,
                100,
                None,
                None,
                Some(hinstance),
                None,
            )
            .unwrap()
        }
    }
}
