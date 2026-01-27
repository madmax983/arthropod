//! Unit tests for BackdropMaterial API.

use plat_core::{BackdropMaterial, HasBackdropMaterial};

#[test]
#[ignore] // Requires actual window - run manually with: cargo test -p plat-core --test material_tests -- --ignored
fn test_apply_mica_to_window() {
    use plat_core::{EventLoop, Size, WindowConfig};

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop
        .create_window(WindowConfig {
            title: "Mica Test".into(),
            size: Size::new(400, 300),
            visible: false,
            ..Default::default()
        })
        .expect("Failed to create window");

    // Should not panic and should update state
    window.set_backdrop_material(BackdropMaterial::Mica);
    assert_eq!(window.backdrop_material(), BackdropMaterial::Mica);

    // Test other materials
    window.set_backdrop_material(BackdropMaterial::Acrylic);
    assert_eq!(window.backdrop_material(), BackdropMaterial::Acrylic);

    window.set_backdrop_material(BackdropMaterial::MicaAlt);
    assert_eq!(window.backdrop_material(), BackdropMaterial::MicaAlt);

    window.set_backdrop_material(BackdropMaterial::None);
    assert_eq!(window.backdrop_material(), BackdropMaterial::None);
}

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
    // Test Copy trait (implicit copy on assignment)
    let copied = original;
    // Test Clone trait explicitly (allowed by attribute even though Copy exists)
    #[allow(clippy::clone_on_copy)]
    let cloned = original.clone();

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
    // Actual implementation tested in test_apply_mica_to_window
}
