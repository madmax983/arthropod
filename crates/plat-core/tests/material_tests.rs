//! Unit tests for BackdropMaterial API.

use plat_core::{BackdropMaterial, HasBackdropMaterial};

#[test]
fn test_material_enum_variants() {
    // Verify all variants exist and can be created
    let mica = BackdropMaterial::Mica;
    let acrylic = BackdropMaterial::Acrylic;
    let mica_alt = BackdropMaterial::MicaAlt;
    let none = BackdropMaterial::None;

    // Verify they are distinct
    assert_ne!(mica, acrylic);
    assert_ne!(mica, mica_alt);
    assert_ne!(mica, none);
    assert_ne!(acrylic, mica_alt);
    assert_ne!(acrylic, none);
    assert_ne!(mica_alt, none);
}

#[test]
fn test_backdrop_material_default() {
    // Default should be None
    let default = BackdropMaterial::default();
    assert_eq!(default, BackdropMaterial::None);
}

#[test]
fn test_backdrop_material_clone_and_copy() {
    let original = BackdropMaterial::Mica;
    let cloned = original.clone();
    let copied = original; // Copy trait

    assert_eq!(original, cloned);
    assert_eq!(original, copied);
}

#[test]
fn test_backdrop_material_debug() {
    // Verify Debug trait works
    let debug_str = format!("{:?}", BackdropMaterial::Mica);
    assert!(debug_str.contains("Mica"));
}

#[test]
fn test_has_backdrop_material_trait_exists() {
    // Verify trait has required method signatures
    fn _check_api<W: HasBackdropMaterial>(w: &W, m: BackdropMaterial) {
        w.set_backdrop_material(m);
        let _: BackdropMaterial = w.backdrop_material();
    }
    // Note: Actual implementation tests will come in Task A2
    // when we implement HasBackdropMaterial for Window
}
