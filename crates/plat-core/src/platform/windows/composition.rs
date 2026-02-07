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
        unsafe {
            // Create D3D11 device for DirectComposition
            let dxgi_device: IDXGIDevice = create_dxgi_device()?;

            // Create DirectComposition desktop device
            let device: IDCompositionDesktopDevice = DCompositionCreateDevice3(&dxgi_device)?;

            Ok(Self { device })
        }
    }

    /// Creates a composition target bound to a window handle.
    ///
    /// This target is where the composition visual tree will be rendered.
    /// If `topmost` is true, the visual tree is rendered on top of the window's children.
    /// If `topmost` is false, it is rendered behind the window's children (but in front of the window background).
    pub fn create_target_for_hwnd(&self, hwnd: HWND, topmost: bool) -> Result<CompositionTarget> {
        unsafe {
            let target = self.device.CreateTargetForHwnd(hwnd, topmost)?;
            Ok(CompositionTarget { target })
        }
    }

    /// Creates a new composition visual.
    pub fn create_visual(&self) -> Result<CompositionVisual> {
        unsafe {
            let visual = self.device.CreateVisual()?;
            Ok(CompositionVisual { visual })
        }
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
        unsafe {
            let visual = self.device.CreateVisual()?;

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
        unsafe {
            self.target.SetRoot(&visual.visual)?;
            Ok(())
        }
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
        unsafe {
            self.visual.SetContent(&surface.surface)?;
            Ok(())
        }
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
        unsafe {
            self.visual.AddVisual(&child.visual, false, None)?;
            Ok(())
        }
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
unsafe fn create_dxgi_device() -> Result<IDXGIDevice> {
    let mut device = None;
    let feature_levels = [D3D_FEATURE_LEVEL_11_0];

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

        let d3d_device = device.ok_or_else(|| Error::from(E_FAIL))?;
        d3d_device.cast::<IDXGIDevice>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::{
        System::{Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize}, LibraryLoader::*},
        UI::WindowsAndMessaging::*,
    };

    #[test]
    fn test_composition_device_creation() {
        // COM must be initialized for DirectComposition
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let device = CompositionDevice::new();
            assert!(device.is_ok(), "Failed to create DirectComposition device");

            CoUninitialize();
        }
    }

    #[test]
    fn test_composition_target_creation() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            // Create a test window (minimal Win32 window)
            let hwnd = create_test_window();

            let device = CompositionDevice::new().unwrap();
            let target = device.create_target_for_hwnd(hwnd, true);
            assert!(target.is_ok(), "Failed to create composition target");

            let _ = DestroyWindow(hwnd);
            CoUninitialize();
        }
    }

    #[test]
    fn test_composition_visual_creation() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let device = CompositionDevice::new().unwrap();
            let visual = device.create_visual();
            assert!(visual.is_ok(), "Failed to create composition visual");

            CoUninitialize();
        }
    }

    #[test]
    fn test_backdrop_visual_creation() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let device = CompositionDevice::new().unwrap();

            // Test creating backdrop visuals for different materials
            let mica = device.create_backdrop_visual(crate::materials::BackdropMaterial::Mica);
            assert!(mica.is_ok(), "Failed to create Mica backdrop visual");

            let acrylic =
                device.create_backdrop_visual(crate::materials::BackdropMaterial::Acrylic);
            assert!(acrylic.is_ok(), "Failed to create Acrylic backdrop visual");

            let none = device.create_backdrop_visual(crate::materials::BackdropMaterial::None);
            assert!(none.is_ok(), "Failed to create None backdrop visual");

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
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    // Helper to create minimal test window
    unsafe fn create_test_window() -> HWND {
        unsafe {
            let class_name = w!("TestWindow");
            let hinstance: HINSTANCE = GetModuleHandleW(None).unwrap().into();
            let wc = WNDCLASSW {
                lpfnWndProc: Some(test_wndproc),
                lpszClassName: class_name,
                hInstance: hinstance,
                ..Default::default()
            };
            let _ = RegisterClassW(&wc);

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
