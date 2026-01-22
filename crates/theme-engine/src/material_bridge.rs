//! Bridge between theme-engine materials and plat-core backdrop materials
//!
//! This module provides conversions from semantic `BackgroundMaterial` values
//! to platform-specific `BackdropMaterial` that can be applied to windows.
//!
//! # Architecture
//!
//! Theme-engine defines `BackgroundMaterial` as a semantic abstraction over
//! platform-specific window materials. The bridge converts these to
//! plat-core's `BackdropMaterial` which represents actual window backdrop
//! effects that can be applied.
//!
//! # Example
//!
//! ```
//! use theme_engine::{BackgroundMaterial, WindowsMaterial};
//! use plat_core::BackdropMaterial;
//!
//! let material = BackgroundMaterial::Windows(WindowsMaterial::Mica);
//! let backdrop: BackdropMaterial = material.into();
//! assert_eq!(backdrop, BackdropMaterial::Mica);
//! ```
//!
//! # Platform Support
//!
//! - **Windows**: Mica, MicaAlt, and Acrylic are all supported
//! - **macOS**: Not yet supported (returns `None`)
//! - **Solid**: Falls back to `None` (no special backdrop effect)

use crate::{BackgroundMaterial, WindowsMaterial};
use plat_core::BackdropMaterial;

impl From<BackgroundMaterial> for BackdropMaterial {
    fn from(material: BackgroundMaterial) -> Self {
        match material {
            BackgroundMaterial::Windows(WindowsMaterial::Mica) => BackdropMaterial::Mica,
            BackgroundMaterial::Windows(WindowsMaterial::MicaAlt) => BackdropMaterial::MicaAlt,
            BackgroundMaterial::Windows(WindowsMaterial::Acrylic) => BackdropMaterial::Acrylic,
            BackgroundMaterial::Solid(_) => BackdropMaterial::None,
            BackgroundMaterial::MacOS(_) => BackdropMaterial::None, // TODO: macOS support
        }
    }
}

impl From<&BackgroundMaterial> for BackdropMaterial {
    fn from(material: &BackgroundMaterial) -> Self {
        match material {
            BackgroundMaterial::Windows(WindowsMaterial::Mica) => BackdropMaterial::Mica,
            BackgroundMaterial::Windows(WindowsMaterial::MicaAlt) => BackdropMaterial::MicaAlt,
            BackgroundMaterial::Windows(WindowsMaterial::Acrylic) => BackdropMaterial::Acrylic,
            BackgroundMaterial::Solid(_) => BackdropMaterial::None,
            BackgroundMaterial::MacOS(_) => BackdropMaterial::None, // TODO: macOS support
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MacOSMaterial;
    use glam::Vec4;

    #[test]
    fn test_windows_mica_converts_to_mica() {
        let material = BackgroundMaterial::Windows(WindowsMaterial::Mica);
        assert_eq!(BackdropMaterial::from(material), BackdropMaterial::Mica);
    }

    #[test]
    fn test_windows_mica_alt_converts_to_mica_alt() {
        let material = BackgroundMaterial::Windows(WindowsMaterial::MicaAlt);
        assert_eq!(BackdropMaterial::from(material), BackdropMaterial::MicaAlt);
    }

    #[test]
    fn test_windows_acrylic_converts_to_acrylic() {
        let material = BackgroundMaterial::Windows(WindowsMaterial::Acrylic);
        assert_eq!(BackdropMaterial::from(material), BackdropMaterial::Acrylic);
    }

    #[test]
    fn test_solid_converts_to_none() {
        let material = BackgroundMaterial::Solid(Vec4::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(BackdropMaterial::from(material), BackdropMaterial::None);
    }

    #[test]
    fn test_macos_converts_to_none() {
        let material = BackgroundMaterial::MacOS(MacOSMaterial::Sidebar);
        assert_eq!(BackdropMaterial::from(material), BackdropMaterial::None);
    }

    #[test]
    fn test_ref_conversion() {
        let material = BackgroundMaterial::Windows(WindowsMaterial::Mica);
        assert_eq!(BackdropMaterial::from(&material), BackdropMaterial::Mica);
    }
}
