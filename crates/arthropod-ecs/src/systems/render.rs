use bevy_ecs::prelude::*;
use render_engine::{backend::RectInstance, NodeContent, Scene};

use crate::components::{Renderable, SceneNodeRef};

/// Resource for collecting render commands
///
/// Systems write RenderCommands into this resource, which is then
/// extracted and passed to the GPU backend for rendering.
#[derive(Resource, Default, Debug)]
pub struct RenderCommands(pub Vec<RectInstance>);

/// Resource wrapper for read-only Scene access
///
/// # Safety
///
/// This resource holds a raw pointer to a Scene for read-only access.
/// The pointer is guaranteed to be valid for the duration of the system execution because:
/// 1. The Scene is inserted before running render systems
/// 2. The Scene is removed immediately after systems complete
/// 3. The systems run synchronously within the render call
#[derive(Resource)]
pub struct SceneReadResource {
    scene_ptr: *const Scene,
}

// SAFETY: SceneReadResource is only used from the main thread in GUI applications.
// All systems run synchronously within the render() method, and the pointer
// is guaranteed valid for that duration.
unsafe impl Send for SceneReadResource {}
unsafe impl Sync for SceneReadResource {}

impl SceneReadResource {
    /// Create a new SceneReadResource from a Scene reference
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. The Scene outlives this resource
    /// 2. The resource is removed before the Scene reference goes out of scope
    pub unsafe fn new(scene: &Scene) -> Self {
        Self {
            scene_ptr: scene as *const Scene,
        }
    }

    /// Get a reference to the Scene
    ///
    /// # Safety
    ///
    /// This is safe because the SceneReadResource is only created and destroyed
    /// within controlled render methods that guarantee the Scene is valid.
    pub fn get(&self) -> &Scene {
        unsafe { &*self.scene_ptr }
    }
}

/// Collect all visible renderables into GPU instances
///
/// This system queries all entities marked as Renderable, fetches their
/// corresponding scene nodes, and generates RectInstances for the GPU backend.
/// Invisible nodes and nodes with zero opacity are filtered out.
pub fn collect_renderables_system(
    query: Query<&SceneNodeRef, With<Renderable>>,
    scene: Res<SceneReadResource>,
    mut commands: ResMut<RenderCommands>,
) {
    commands.0.clear();
    let scene = scene.get();

    for node_ref in query.iter() {
        if let Some(node) = scene.get(node_ref.0) {
            // Skip invisible nodes and fully transparent nodes
            if !node.visible || node.opacity <= 0.0 {
                continue;
            }

            // Generate render instances based on node content type
            match &node.content {
                NodeContent::Rect { color } | NodeContent::RoundedRect { color, .. } => {
                    commands.0.push(RectInstance {
                        pos: [node.bounds.x, node.bounds.y],
                        size: [node.bounds.width, node.bounds.height],
                        color: [
                            color.r(),
                            color.g(),
                            color.b(),
                            color.a() * node.opacity, // Apply opacity
                        ],
                    });
                }
                NodeContent::Empty => {
                    // Empty nodes have no visual representation
                }
            }
        }
    }
}
