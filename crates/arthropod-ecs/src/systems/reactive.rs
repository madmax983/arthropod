use bevy_ecs::prelude::*;
use render_engine::{NodeContent, Scene};

use crate::components::{ReactiveColor, ReactiveOpacity, ReactiveTransform, SceneNodeRef};

/// Update scene node colors from reactive signals
///
/// This system queries all entities with ReactiveColor components and updates
/// the corresponding scene nodes' color properties by polling the signals.
///
/// Scene is now accessed as a safe Resource - no more unsafe pointer juggling!
pub fn update_reactive_colors_system(
    query: Query<(&SceneNodeRef, &ReactiveColor)>,
    mut scene: ResMut<Scene>,
) {
    for (node_ref, reactive) in query.iter() {
        // ResMut<Scene> implements DerefMut, so we can call Scene methods directly
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
                NodeContent::Text { ref text, font_size, .. } => NodeContent::Text {
                    text: text.clone(),
                    font_size,
                    color: new_color,
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
    mut scene: ResMut<Scene>,
) {
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
    mut scene: ResMut<Scene>,
) {
    for (node_ref, reactive) in query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            // Poll the signal to get the current opacity value
            node.opacity = reactive.signal.inner().get_untracked();
        }
    }
}
