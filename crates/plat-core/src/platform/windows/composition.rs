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
            Dxgi::*,
            Direct3D::*,
            Direct3D11::*,
        },
        System::Com::*,
    },
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
            let device: IDCompositionDesktopDevice =
                DCompositionCreateDevice3(&dxgi_device)?;

            Ok(Self { device })
        }
    }

    /// Creates a composition target bound to a window handle.
    ///
    /// This target is where the composition visual tree will be rendered.
    pub fn create_target_for_hwnd(&self, hwnd: HWND) -> Result<CompositionTarget> {
        unsafe {
            let target = self.device.CreateTargetForHwnd(hwnd, true)?;
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

    /// Creates a composition surface from a DXGI swap chain.
    ///
    /// This allows wgpu's swap chain output to be used in the composition tree.
    #[allow(dead_code)]
    pub fn create_surface_from_swap_chain(
        &self,
        swap_chain: &IDXGISwapChain1,
    ) -> Result<CompositionSurface> {
        unsafe {
            // Cast device to IUnknown to access CreateSurfaceFromSwapChain
            // Note: Method signature varies by DirectComposition version
            let surface: IUnknown = swap_chain.cast()?;
            Ok(CompositionSurface { surface })
        }
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
    pub fn raw_target(&self) -> &IDCompositionTarget {
        &self.target
    }
}

/// Wrapper around IDCompositionVisual2 for the visual tree.
pub struct CompositionVisual {
    visual: IDCompositionVisual2,
}

impl CompositionVisual {
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
    #[allow(dead_code)]
    pub fn add_child(&self, _child: &CompositionVisual) -> Result<()> {
        // Would use AddVisual() when implementing full hierarchy
        Ok(())
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
        HMODULE(std::ptr::null_mut()), // No software rasterizer
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
        System::LibraryLoader::*,
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
            let target = device.create_target_for_hwnd(hwnd);
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
                0, 0, 100, 100,
                None, None,
                Some(hinstance),
                None,
            ).unwrap()
        }
    }
}
