use bevy_ecs::prelude::*;
use glam::Vec2;
use crate::transform::Camera;

/// A node in the spatial graph.
/// This component stores the world position and size of the node.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SpatialNode {
    pub position: Vec2,
    pub size: Vec2,
}

/// The viewport (camera) for the spatial graph.
/// This resource stores the current camera state.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Viewport {
    pub camera: Camera,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            camera: Camera::new(Vec2::new(800.0, 600.0)),
        }
    }
}
