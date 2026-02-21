#[cfg(feature = "nova")]
pub mod particles;

#[cfg(feature = "nova")]
pub mod story;

#[cfg(feature = "nova")]
pub mod elastic;

#[cfg(feature = "nova")]
pub mod xray;

// --- Stubs for missing features ---

#[cfg(not(feature = "nova"))]
pub mod story {
    #![allow(deprecated)]
    use bevy_ecs::prelude::Component;

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// You are trying to use `NarrativeGenerator`, but the `nova` feature is not enabled.
    ///
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    #[derive(Component, Default)]
    #[deprecated(note = "Requires 'nova' feature. Enable it in Cargo.toml.")]
    pub struct NarrativeGenerator;

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// You are trying to use `register_story`, but the `nova` feature is not enabled.
    #[deprecated(note = "Requires 'nova' feature. Enable it in Cargo.toml.")]
    pub fn register_story(_app: &mut crate::App) {
        eprintln!("ERROR: 'register_story' requires 'nova' feature. Enable it in Cargo.toml.");
    }

    /// ⚠️ **MISSING FEATURE** ⚠️
    #[deprecated(note = "Requires 'nova' feature. Enable it in Cargo.toml.")]
    #[allow(deprecated)]
    pub fn generate_narrative(
        _query: bevy_ecs::prelude::Query<
            bevy_ecs::prelude::Entity,
            bevy_ecs::prelude::Added<NarrativeGenerator>,
        >,
        _scene: bevy_ecs::prelude::ResMut<render_engine::Scene>,
    ) {
    }
}

#[cfg(not(feature = "nova"))]
pub mod particles {
    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `particles` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();
}

#[cfg(not(feature = "nova"))]
pub mod xray {
    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `xray` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();

    #[allow(dead_code)]
    pub fn register_xray(_app: &mut crate::App) {
        eprintln!("ERROR: 'register_xray' requires 'nova' feature. Enable it in Cargo.toml.");
    }
}

#[cfg(not(feature = "nova"))]
pub mod elastic {
    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `elastic` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();
}
