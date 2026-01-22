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
