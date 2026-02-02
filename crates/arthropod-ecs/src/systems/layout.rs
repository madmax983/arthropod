//! Layout system for Arthropod ECS
//!
//! Automatically computes flexbox layout for scene nodes with LayoutStyle components.

use crate::components::{LayoutConstraintsResource, LayoutStyle, SceneNodeRef};
use bevy_ecs::prelude::*;
use layout_engine::{FlexDirection, FlexStyle, LayoutConstraints, LayoutEngine};
use plat_core::Rect;
use render_engine::{NodeId, Scene};
use std::collections::HashMap;
use std::marker::PhantomData;

/// Layout system - runs in the update schedule
///
/// 1. Builds a temporary layout tree from Scene + LayoutStyle components
/// 2. Computes layout using layout-engine (Taffy)
/// 3. Updates SceneNode bounds
pub fn layout_system(
    mut scene: ResMut<Scene>,
    constraints_res: Option<Res<LayoutConstraintsResource>>,
    query: Query<(&SceneNodeRef, &LayoutStyle)>,
) {
    // 1. Collect all layout styles into a lookup map
    // This allows us to look up styles by NodeId during recursive scene traversal
    let layout_styles: HashMap<NodeId, FlexStyle> = query
        .iter()
        .map(|(node_ref, style)| (node_ref.0, style.0.clone()))
        .collect();

    if layout_styles.is_empty() {
        return;
    }

    let root = scene.root();
    let constraints = constraints_res
        .map(|r| r.0.clone())
        .unwrap_or(LayoutConstraints::default());

    // 2. Perform layout calculation
    // We use a helper function similar to arthropod::layout::auto_layout
    // but adapted for the system context
    perform_layout(&mut scene, root, constraints, &layout_styles);
}

fn perform_layout(
    scene: &mut Scene,
    root: NodeId,
    constraints: LayoutConstraints,
    layout_styles: &HashMap<NodeId, FlexStyle>,
) {
    let mut engine = LayoutEngine::new();
    let mut node_map: HashMap<NodeId, layout_engine::NodeId> = HashMap::new();

    // Build layout tree recursively
    build_layout_tree(scene, root, &mut engine, &mut node_map, layout_styles);

    // Get layout root
    if let Some(&layout_root) = node_map.get(&root) {
        // Compute layout
        engine.compute_layout(layout_root, constraints);

        // Apply computed layouts to scene
        apply_layouts(scene, root, &engine, &node_map, 0.0, 0.0);
    }
}

/// Build the layout tree recursively.
fn build_layout_tree(
    scene: &Scene,
    node_id: NodeId,
    engine: &mut LayoutEngine,
    node_map: &mut HashMap<NodeId, layout_engine::NodeId>,
    layout_styles: &HashMap<NodeId, FlexStyle>,
) {
    // Get style for this node (or default)
    let style = layout_styles
        .get(&node_id)
        .cloned()
        .unwrap_or_else(default_style);

    // Create layout node
    let layout_node = engine.create_node(style);
    node_map.insert(node_id, layout_node);

    // Process children
    if let Some(scene_node) = scene.get_node(node_id) {
        // Iterate over children reference directly
        for &child_id in &scene_node.children {
            build_layout_tree(scene, child_id, engine, node_map, layout_styles);

            // Add as child in layout tree
            if let Some(&child_layout) = node_map.get(&child_id) {
                engine.add_child(layout_node, child_layout);
            }
        }
    }
}

/// Guard to restore children to a node on Drop, ensuring panic safety.
struct ChildrenGuard<'a> {
    scene: *mut Scene,
    node_id: NodeId,
    children: Vec<NodeId>,
    _marker: PhantomData<&'a mut Scene>,
}

impl<'a> Drop for ChildrenGuard<'a> {
    fn drop(&mut self) {
        // SAFETY: The scene pointer is valid because the guard is created from a valid
        // &mut Scene reference that outlives the guard's scope.
        unsafe {
            if let Some(scene) = self.scene.as_mut() {
                if let Some(node) = scene.get_node_mut(self.node_id) {
                    node.children = std::mem::take(&mut self.children);
                }
            }
        }
    }
}

/// Apply computed layouts to scene node bounds.
fn apply_layouts(
    scene: &mut Scene,
    node_id: NodeId,
    engine: &LayoutEngine,
    node_map: &HashMap<NodeId, layout_engine::NodeId>,
    parent_x: f32,
    parent_y: f32,
) {
    if let Some(&layout_node) = node_map.get(&node_id) {
        if let Some(layout) = engine.get_layout(layout_node) {
            // Compute absolute position
            let abs_x = parent_x + layout.x;
            let abs_y = parent_y + layout.y;

            // Update scene node bounds and process children.
            // We use std::mem::take to avoid cloning the children vector (O(N) allocations).
            let children = if let Some(scene_node) = scene.get_node_mut(node_id) {
                scene_node.bounds = Rect::new(abs_x, abs_y, layout.width, layout.height);
                std::mem::take(&mut scene_node.children)
            } else {
                Vec::new()
            };

            // Create guard to restore children on scope exit (or panic)
            let guard = ChildrenGuard {
                scene: scene as *mut _,
                node_id,
                children,
                _marker: PhantomData,
            };

            // Recurse using the children from the guard
            for &child_id in &guard.children {
                apply_layouts(scene, child_id, engine, node_map, abs_x, abs_y);
            }
        }
    }
}

fn default_style() -> FlexStyle {
    FlexStyle {
        direction: FlexDirection::Column,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        width: None,
        height: None,
        gap: 0.0,
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
    }
}
