//! DirectComposition-compatible swap chain configuration.
//!
//! Provides utilities for creating DXGI swap chains compatible with
//! DirectComposition visual trees.

#![cfg(target_os = "windows")]

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_surface_config() {
        let config = create_composition_surface_config(800, 600, wgpu::PresentMode::Fifo);

        // Verify composition-compatible settings
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.format, wgpu::TextureFormat::Bgra8UnormSrgb);
        assert_eq!(config.alpha_mode, wgpu::CompositeAlphaMode::PreMultiplied);
    }
}
