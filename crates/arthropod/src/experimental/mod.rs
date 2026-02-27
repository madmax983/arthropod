#[cfg(feature = "nova")]
pub mod particles;

#[cfg(feature = "nova")]
pub mod story;

#[cfg(feature = "nova")]
pub mod elastic;

#[cfg(feature = "nova")]
pub mod xray;

#[cfg(feature = "nova")]
pub mod chronos;

#[cfg(feature = "nova")]
pub mod ghost_replay;

#[cfg(feature = "nova")]
pub mod signal_graph;

#[cfg(feature = "nova")]
pub mod noise;

#[cfg(feature = "nova")]
pub mod reactive_particles;

#[cfg(feature = "nova")]
pub mod kinetic_text;

pub mod flux_radar;

// --- Stubs for missing features ---

#[cfg(not(feature = "nova"))]
pub mod ghost_replay {
    #![allow(deprecated)]

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `ghost_replay` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();

    #[deprecated(note = "Requires 'nova' feature. Enable it in Cargo.toml.")]
    pub fn record_event(_world: &mut bevy_ecs::prelude::World, _event: &plat_core::Event) {
        // No-op
    }

    #[deprecated(note = "Requires 'nova' feature. Enable it in Cargo.toml.")]
    pub fn init_ghost_replay(_app: &mut crate::App) {
        eprintln!("ERROR: 'init_ghost_replay' requires 'nova' feature. Enable it in Cargo.toml.");
    }
}

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
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct NarrativeGenerator;

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// You are trying to use `StoryRuntime`, but the `nova` feature is not enabled.
    #[derive(bevy_ecs::prelude::Resource, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct StoryRuntime;

    impl StoryRuntime {
        /// Stub for `new` method
        #[allow(clippy::new_ret_no_self)]
        pub fn new(_story: Story) -> Self {
            panic!("StoryRuntime requires 'nova' feature. Enable it in Cargo.toml.");
        }

        /// Stub for `current_text` method
        pub fn current_text(&self) -> String {
            panic!("StoryRuntime requires 'nova' feature. Enable it in Cargo.toml.");
        }

        /// Stub for `choose` method
        #[allow(clippy::result_unit_err)]
        pub fn choose(&mut self, _index: usize) -> Result<(), String> {
            panic!("StoryRuntime requires 'nova' feature. Enable it in Cargo.toml.");
        }
    }

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// You are trying to use `register_story`, but the `nova` feature is not enabled.
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn register_story(_app: &mut crate::App) {
        eprintln!(
            "ERROR: 'register_story' - This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
        );
    }

    /// ⚠️ **MISSING FEATURE** ⚠️
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    #[allow(deprecated)]
    pub fn generate_narrative(
        _query: bevy_ecs::prelude::Query<
            bevy_ecs::prelude::Entity,
            bevy_ecs::prelude::Added<NarrativeGenerator>,
        >,
        _scene: bevy_ecs::prelude::ResMut<render_engine::Scene>,
    ) {
    }

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// You are trying to use `Story`, but the `nova` feature is not enabled.
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct Story;

    impl Story {
        /// Stub for `new` method
        #[allow(clippy::new_ret_no_self)]
        pub fn new(_start_node: impl Into<String>) -> Self {
            panic!("Story requires 'nova' feature. Enable it in Cargo.toml.");
        }

        /// Stub for `add_passage` method
        pub fn add_passage(&mut self, _passage: Passage) {
            // No-op
        }
    }

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// You are trying to use `Passage`, but the `nova` feature is not enabled.
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct Passage;

    impl Passage {
        /// Stub for `new` method
        #[allow(clippy::new_ret_no_self)]
        pub fn new(_id: impl Into<String>, _text: impl Into<String>) -> Self {
            panic!("Passage requires 'nova' feature. Enable it in Cargo.toml.");
        }

        /// Stub for `add_choice` method
        pub fn add_choice(self, _text: impl Into<String>, _target: impl Into<String>) -> Self {
            self
        }
    }
}

#[cfg(not(feature = "nova"))]
pub mod particles {
    #![allow(deprecated)]
    use bevy_ecs::prelude::{Component, Resource};

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `particles` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();

    #[derive(Component, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct Particle;

    #[derive(Component, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct ParticleEmitter;

    #[derive(Resource, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct ParticleTime;

    #[derive(Resource, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct ParticleGlobalState;

    #[derive(Component, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub enum ForceField {
        #[default]
        None,
    }

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn update_particle_time(
        _time: bevy_ecs::prelude::ResMut<ParticleTime>,
        _state: bevy_ecs::prelude::ResMut<ParticleGlobalState>,
    ) {
    }

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn emit_particles(
        _commands: bevy_ecs::prelude::Commands,
        _emitters: bevy_ecs::prelude::Query<&mut ParticleEmitter>,
        _time: bevy_ecs::prelude::Res<ParticleTime>,
        _scene: bevy_ecs::prelude::ResMut<render_engine::Scene>,
    ) {
    }

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn apply_forces(
        _particles: bevy_ecs::prelude::Query<&mut Particle>,
        _forces: bevy_ecs::prelude::Query<&ForceField>,
        _time: bevy_ecs::prelude::Res<ParticleTime>,
        _scene: bevy_ecs::prelude::Res<render_engine::Scene>,
    ) {
    }

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn update_particles(
        _commands: bevy_ecs::prelude::Commands,
        _particles: bevy_ecs::prelude::Query<(bevy_ecs::prelude::Entity, &mut Particle)>,
        _time: bevy_ecs::prelude::Res<ParticleTime>,
        _scene: bevy_ecs::prelude::ResMut<render_engine::Scene>,
    ) {
    }

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn register_particles(_app: &mut crate::App) {
        eprintln!(
            "ERROR: 'register_particles' - This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
        );
    }
}

#[cfg(not(feature = "nova"))]
pub mod kinetic_text {
    #![allow(deprecated)]

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `kinetic_text` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();

    #[derive(Debug, Clone, Copy, PartialEq)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub enum TextAnimation {
        None,
        Typewriter,
        FadeIn,
        Pulse,
    }

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct KineticText;

    impl KineticText {
        /// Stub for `new` method
        #[allow(clippy::new_ret_no_self)]
        pub fn new<T>(_content: T) -> Self {
            panic!("KineticText requires 'nova' feature. Enable it in Cargo.toml.");
        }

        pub fn animation(self, _animation: TextAnimation) -> Self {
            self
        }

        pub fn size(self, _size: f32) -> Self {
            self
        }

        pub fn color(self, _color: render_engine::Color) -> Self {
            self
        }

        pub fn clock<T>(self, _clock: T) -> Self {
            self
        }
    }
}

#[cfg(not(feature = "nova"))]
pub mod reactive_particles {
    #![allow(deprecated)]
    use bevy_ecs::prelude::Component;

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `reactive_particles` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();

    #[derive(Component, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct ReactiveParticleEmitter {
        pub rate: Option<()>,
        pub color: Option<()>,
        pub size: Option<()>,
        pub spread: Option<()>,
        pub active: Option<()>,
    }

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn sync_reactive_emitters(
        _emitters: bevy_ecs::prelude::Query<(
            &mut super::particles::ParticleEmitter,
            &ReactiveParticleEmitter,
        )>,
    ) {
    }

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn register_reactive_particles(_app: &mut crate::App) {
        eprintln!(
            "ERROR: 'register_reactive_particles' - This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
        );
    }
}

#[cfg(not(feature = "nova"))]
pub mod noise {
    #![allow(deprecated)]

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `noise` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct NoiseSignal;

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct NoiseSignal2D;
}

#[cfg(not(feature = "nova"))]
pub mod xray {
    #![allow(deprecated)]
    use bevy_ecs::prelude::Resource;

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `xray` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();

    #[derive(Resource, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct XRayConfig;

    #[derive(Resource, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct XRayState;

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn update_xray(
        _scene: bevy_ecs::prelude::ResMut<render_engine::Scene>,
        _config: bevy_ecs::prelude::Res<XRayConfig>,
        _state: bevy_ecs::prelude::ResMut<XRayState>,
    ) {
    }

    #[allow(dead_code)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn register_xray(_app: &mut crate::App) {
        eprintln!(
            "ERROR: 'register_xray' - This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
        );
    }
}

#[cfg(not(feature = "nova"))]
pub mod elastic {
    #![allow(deprecated)]
    use bevy_ecs::prelude::Resource;
    use std::marker::PhantomData;

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `elastic` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();

    #[derive(Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct ElasticSignal<T>(PhantomData<T>);

    #[derive(Resource, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct ElasticRegistry;

    #[derive(Resource, Default)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct ElasticTime;

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn update_elastic_time_system(_time: bevy_ecs::prelude::ResMut<ElasticTime>) {}

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn elastic_tick_system(
        _registry: bevy_ecs::prelude::Res<ElasticRegistry>,
        _time: bevy_ecs::prelude::Res<ElasticTime>,
    ) {
    }

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn register_elastic_feature(_world: &mut bevy_ecs::prelude::World) {
        eprintln!(
            "ERROR: 'register_elastic_feature' - This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
        );
    }
}

#[cfg(not(feature = "nova"))]
pub mod chronos {
    #![allow(deprecated)]
    use std::marker::PhantomData;

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `chronos` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct Timeline;

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct RetroSignal<T>(PhantomData<T>);

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct ChronosDebugger;
}

#[cfg(not(feature = "nova"))]
pub mod signal_graph {
    #![allow(deprecated)]
    use bevy_ecs::prelude::Component;

    /// ⚠️ **MISSING FEATURE** ⚠️
    ///
    /// The `signal_graph` module requires the `nova` feature.
    /// Add `features = ["nova"]` to your `arthropod` dependency in `Cargo.toml`.
    pub const MISSING_FEATURE: () = ();

    #[derive(Component)]
    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub struct SignalGraph;

    impl SignalGraph {
        /// Stub for missing feature
        #[allow(clippy::new_ret_no_self)]
        pub fn new<T>(_signal: T, _min: f32, _max: f32) -> Self {
            panic!("SignalGraph requires 'nova' feature. Enable it in Cargo.toml.");
        }

        pub fn with_color(self, _color: render_engine::Color) -> Self {
            self
        }

        pub fn with_history(self, _length: usize) -> Self {
            self
        }
    }

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn update_signal_graphs(_commands: bevy_ecs::prelude::Commands) {}

    #[deprecated(
        note = "This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
    )]
    pub fn register_signal_graph(_app: &mut crate::App) {
        eprintln!(
            "ERROR: 'register_signal_graph' - This feature is EXPERIMENTAL and requires the 'nova' feature. Enable it in Cargo.toml."
        );
    }
}
