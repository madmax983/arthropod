/// Particle systems for visual effects (e.g., confetti, snow, fireworks).
#[cfg(feature = "nova")]
pub mod particles;

/// The Nova Story Engine for interactive narratives.
#[cfg(feature = "nova")]
pub mod story;

/// Elastic and spring-based physics animations for UI elements.
#[cfg(feature = "nova")]
pub mod elastic;

/// X-Ray tool for spatial debugging of the UI scene graph.
#[cfg(feature = "nova")]
pub mod xray;

/// Time manipulation and timeline control utilities.
#[cfg(feature = "nova")]
pub mod chronos;

/// Ghost Replay for recording and replaying user interactions.
#[cfg(feature = "nova")]
pub mod ghost_replay;

/// Visual node-based graph for debugging reactive signals.
#[cfg(feature = "nova")]
pub mod signal_graph;

/// Noise generation utilities for procedural textures and animations.
#[cfg(feature = "nova")]
pub mod noise;

/// Particle systems that react to ECS component changes or UI events.
#[cfg(feature = "nova")]
pub mod reactive_particles;

/// Kinetic typography and animated text utilities.
#[cfg(feature = "nova")]
pub mod kinetic_text;

/// Gesture and multi-touch recognition engine.
#[cfg(feature = "nova")]
pub mod spellcaster;

/// Command palette implementation for quick actions (like Spotlight/Raycast).
#[cfg(feature = "nova")]
pub mod command_palette;

/// Flux Radar for visualizing reactive dependency cycles and hot paths.
#[cfg(feature = "nova")]
pub mod flux_radar;

/// Spatial queries for advanced hit testing and proximity interactions.
#[cfg(feature = "nova")]
pub mod spatial_query;

/// Inspector overlay for visualizing layout bounds and styling properties.
#[cfg(feature = "nova")]
pub mod inspector_overlay;

/// Magneto physics for snapping and attraction interactions.
#[cfg(feature = "nova")]
pub mod magneto;

/// Spotlight overlay effect that highlights specific UI regions.
#[cfg(feature = "nova")]
pub mod spotlight;

/// Parallax scrolling effects for depth illusions.
#[cfg(feature = "nova")]
pub mod parallax;

/// Mouse trail particle effects.
#[cfg(feature = "nova")]
pub mod mouse_trail;

/// Glassmorphism and frosted glass blur overlays.
#[cfg(feature = "nova")]
pub mod glass_overlay;

/// Chaos Monkey testing tool for randomly dropping events and dropping nodes.
#[cfg(feature = "nova")]
pub mod chaos_monkey;

/// Visual feedback indicators for touch gestures.
#[cfg(feature = "nova")]
pub mod gesture_feedback;

/// Draggable nodes using ECS components and pointer inputs.
#[cfg(feature = "nova")]
pub mod draggable;

/// Expanding visual ripples spawned from reactive triggers.
#[cfg(feature = "nova")]
pub mod reactive_ripples;
