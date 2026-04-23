//! ECS Components and Resources for the spatial graph.
//!
//! Provides the data structures representing the state of nodes, cameras, and
//! system metrics.

use crate::transform::Camera;
use bevy_ecs::prelude::*;
use glam::Vec2;

/// A node in the spatial graph representing a physical entity in the infinite 2D world space.
///
/// This component tracks where an object is located in the global coordinate system, independent
/// of the screen or camera. The ECS systems will project this `SpatialNode` into a renderable
/// `SceneNode` based on the active [`Viewport`].
///
/// ## Examples
///
/// ```rust
/// use spatial_graph::components::SpatialNode;
/// use glam::Vec2;
///
/// let node = SpatialNode {
///     position: Vec2::new(100.0, 200.0),
///     size: Vec2::new(50.0, 50.0),
/// };
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SpatialNode {
    /// The absolute position in infinite world space.
    pub position: Vec2,
    /// The physical dimensions of the node in world units.
    pub size: Vec2,
}

/// The viewport (camera) resource that determines what part of the world is currently visible.
///
/// This resource acts as the "lens" through which the user views the spatial graph. It encapsulates
/// a [`Camera`] which holds the pan and zoom state. When the camera moves, the `spatial_transform_system`
/// will automatically recalculate the screen bounds for all `SpatialNode` entities.
///
/// ## Examples
///
/// ```rust
/// use spatial_graph::components::Viewport;
/// use spatial_graph::transform::Camera;
/// use glam::Vec2;
///
/// let mut viewport = Viewport::default();
/// viewport.camera.zoom = 2.0; // Zoom in
/// ```
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Viewport {
    /// The mathematical camera configuration mapping world space to screen space.
    pub camera: Camera,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            camera: Camera::new(Vec2::new(800.0, 600.0)),
        }
    }
}

/// Real-time statistics tracking the efficiency of the spatial graph culling.
///
/// This resource is updated every frame by the `stats_system` to provide observability into
/// how many nodes exist in the world versus how many are actually being rendered on screen.
/// It is useful for debugging frustum culling performance.
///
/// ## Examples
///
/// ```rust
/// use spatial_graph::components::Stats;
///
/// let mut stats = Stats::default();
/// stats.total_nodes = 10000;
/// stats.visible_nodes = 42;
///
/// assert_eq!(stats.visible_nodes, 42);
/// ```
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Stats {
    /// The total number of nodes currently registered in the ECS world.
    pub total_nodes: usize,
    /// The number of nodes currently inside the camera's view frustum.
    pub visible_nodes: usize,
}
