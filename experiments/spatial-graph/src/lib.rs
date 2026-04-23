//! Spatial Graph Experiment.
//!
//! This crate provides a 2D infinite spatial node graph system. It demonstrates how to map
//! an infinite world space coordinate system onto a finite screen viewport using an ECS architecture.

pub mod components;
pub mod systems;
pub mod transform;

pub use components::{SpatialNode, Stats, Viewport};
pub use systems::{spatial_transform_system, stats_system};
pub use transform::Camera;
