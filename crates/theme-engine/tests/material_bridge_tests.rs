//! Tests for the material bridge between theme-engine and plat-core
//!
//! These tests verify the conversion from semantic BackgroundMaterial
//! to platform-specific BackdropMaterial.

use plat_core::BackdropMaterial;
use theme_engine::{BackgroundMaterial, WindowsMaterial};

#[test]
fn test_background_material_to_backdrop_mica() {
    let mica = BackgroundMaterial::Windows(WindowsMaterial::Mica);
    let backdrop: BackdropMaterial = mica.into();
    assert_eq!(backdrop, BackdropMaterial::Mica);
}

#[test]
fn test_background_material_to_backdrop_acrylic() {
    let acrylic = BackgroundMaterial::Windows(WindowsMaterial::Acrylic);
    let backdrop: BackdropMaterial = acrylic.into();
    assert_eq!(backdrop, BackdropMaterial::Acrylic);
}

#[test]
fn test_background_material_to_backdrop_mica_alt() {
    let mica_alt = BackgroundMaterial::Windows(WindowsMaterial::MicaAlt);
    let backdrop: BackdropMaterial = mica_alt.into();
    assert_eq!(backdrop, BackdropMaterial::MicaAlt);
}

#[test]
fn test_background_material_solid_to_none() {
    use glam::Vec4;
    let solid = BackgroundMaterial::Solid(Vec4::ONE);
    let backdrop: BackdropMaterial = solid.into();
    assert_eq!(backdrop, BackdropMaterial::None);
}

#[test]
fn test_background_material_macos_to_none() {
    use theme_engine::MacOSMaterial;
    let macos = BackgroundMaterial::MacOS(MacOSMaterial::Sidebar);
    let backdrop: BackdropMaterial = macos.into();
    assert_eq!(backdrop, BackdropMaterial::None); // Not supported yet
}

#[test]
fn test_background_material_ref_conversion() {
    // Test that we can also convert from references
    let mica = BackgroundMaterial::Windows(WindowsMaterial::Mica);
    let backdrop: BackdropMaterial = (&mica).into();
    assert_eq!(backdrop, BackdropMaterial::Mica);
}

#[test]
fn test_all_windows_materials_are_supported() {
    // Ensure all WindowsMaterial variants have corresponding BackdropMaterial
    let materials = [
        (WindowsMaterial::Mica, BackdropMaterial::Mica),
        (WindowsMaterial::MicaAlt, BackdropMaterial::MicaAlt),
        (WindowsMaterial::Acrylic, BackdropMaterial::Acrylic),
    ];

    for (windows_mat, expected_backdrop) in materials {
        let bg_material = BackgroundMaterial::Windows(windows_mat);
        let backdrop: BackdropMaterial = bg_material.into();
        assert_eq!(
            backdrop, expected_backdrop,
            "WindowsMaterial::{:?} should convert to BackdropMaterial::{:?}",
            windows_mat, expected_backdrop
        );
    }
}

#[test]
fn test_all_macos_materials_return_none() {
    use theme_engine::MacOSMaterial;
    // macOS materials are not yet supported, should all return None
    let macos_materials = [
        MacOSMaterial::Sidebar,
        MacOSMaterial::HeaderView,
        MacOSMaterial::Menu,
        MacOSMaterial::Popover,
        MacOSMaterial::Selection,
    ];

    for macos_mat in macos_materials {
        let bg_material = BackgroundMaterial::MacOS(macos_mat);
        let backdrop: BackdropMaterial = bg_material.into();
        assert_eq!(
            backdrop,
            BackdropMaterial::None,
            "MacOSMaterial::{:?} should convert to BackdropMaterial::None (not yet supported)",
            macos_mat
        );
    }
}
