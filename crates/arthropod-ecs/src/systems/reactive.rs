use bevy_ecs::prelude::*;
use render_engine::{NodeContent, Scene};

use crate::components::{ReactiveColor, ReactiveOpacity, ReactiveTransform, SceneNodeRef};

/// Resource wrapper for Scene - allows systems to access the scene tree
///
/// # Safety
///
/// This resource holds a raw pointer to a Scene. The pointer is guaranteed to be valid
/// for the duration of the system execution because:
/// 1. The Scene is inserted before running systems
/// 2. The Scene is removed immediately after systems complete
/// 3. The systems run synchronously within the update/render call
///
/// This pattern is necessary because bevy_ecs requires resources to be 'static,
/// but we want to temporarily grant systems access to an external Scene tree.
#[derive(Resource)]
pub struct SceneResource {
    scene_ptr: *mut Scene,
}

// SAFETY: SceneResource is only used from the main thread in GUI applications.
// All systems run synchronously within the update() method, and the pointer
// is guaranteed valid for that duration.
unsafe impl Send for SceneResource {}
unsafe impl Sync for SceneResource {}

impl SceneResource {
    /// Create a new SceneResource from a mutable Scene reference
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. The Scene outlives this resource
    /// 2. No other code accesses the Scene while this resource exists
    /// 3. The resource is removed before the Scene reference goes out of scope
    pub unsafe fn new(scene: &mut Scene) -> Self {
        Self {
            scene_ptr: scene as *mut Scene,
        }
    }

    /// Get a mutable reference to the Scene
    ///
    /// # Safety
    ///
    /// This is safe because the SceneResource is only created and destroyed
    /// within controlled update/render methods that guarantee the Scene is valid.
    pub fn get_mut(&mut self) -> &mut Scene {
        unsafe { &mut *self.scene_ptr }
    }
}

/// Update scene node colors from reactive signals
///
/// This system queries all entities with ReactiveColor components and updates
/// the corresponding scene nodes' color properties by polling the signals.
pub fn update_reactive_colors_system(
    query: Query<(&SceneNodeRef, &ReactiveColor)>,
    mut scene: ResMut<SceneResource>,
) {
    let scene = scene.get_mut();
    for (node_ref, reactive) in query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            // Poll the signal to get the current color value
            let new_color = reactive.signal.inner().get_untracked();

            // Update the node's content based on its type
            node.content = match node.content {
                NodeContent::Rect { .. } => NodeContent::Rect { color: new_color },
                NodeContent::RoundedRect { corner_radius, .. } => NodeContent::RoundedRect {
                    color: new_color,
                    corner_radius,
                },
                NodeContent::Empty => NodeContent::Empty,
            };
        }
    }
}

/// Update scene node transforms from reactive signals
///
/// This system queries all entities with ReactiveTransform components and updates
/// the corresponding scene nodes' transform properties by polling the signals.
pub fn update_reactive_transforms_system(
    query: Query<(&SceneNodeRef, &ReactiveTransform)>,
    mut scene: ResMut<SceneResource>,
) {
    let scene = scene.get_mut();
    for (node_ref, reactive) in query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            // Poll the signal to get the current transform value
            node.transform = reactive.signal.inner().get_untracked();
        }
    }
}

/// Update scene node opacity from reactive signals
///
/// This system queries all entities with ReactiveOpacity components and updates
/// the corresponding scene nodes' opacity properties by polling the signals.
pub fn update_reactive_opacity_system(
    query: Query<(&SceneNodeRef, &ReactiveOpacity)>,
    mut scene: ResMut<SceneResource>,
) {
    let scene = scene.get_mut();
    for (node_ref, reactive) in query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            // Poll the signal to get the current opacity value
            node.opacity = reactive.signal.inner().get_untracked();
        }
    }
}
