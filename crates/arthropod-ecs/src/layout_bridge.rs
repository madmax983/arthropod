//! Shared layout logic bridging ECS and Scene
//!
//! This module contains the core layout algorithm used by both:
//! - `arthropod-ecs`: The ECS layout system
//! - `arthropod`: The standalone `auto_layout` utility
//!
//! It bridges the gap between `layout-engine` (Taffy) and `render-engine` (Scene).

use layout_engine::{FlexDirection, FlexStyle, LayoutConstraints, LayoutEngine};
use plat_core::Rect;
use render_engine::{NodeId, Scene};
use std::collections::HashMap;

/// Perform layout calculation on the scene tree.
///
/// Uses flexbox layout from the layout-engine crate to compute positions
/// for all nodes based on their FlexStyle.
///
/// # Arguments
///
/// * `scene` - The scene to layout
/// * `root` - Root node to start layout from
/// * `constraints` - Constraints for the root node
/// * `layout_styles` - Map of NodeId to FlexStyle
/// * `engine` - The layout engine to use (can be reused across frames)
/// * `node_map` - Map from Scene NodeId to layout NodeId (can be reused across frames)
pub fn perform_layout(
    scene: &mut Scene,
    root: NodeId,
    constraints: LayoutConstraints,
    layout_styles: &HashMap<NodeId, FlexStyle>,
    engine: &mut LayoutEngine,
    node_map: &mut HashMap<NodeId, layout_engine::NodeId>,
) {
    engine.clear();
    node_map.clear();

    // Build layout tree recursively
    build_layout_tree(scene, root, engine, node_map, layout_styles);

    // Get layout root
    if let Some(&layout_root) = node_map.get(&root) {
        // Compute layout
        engine.compute_layout(layout_root, constraints);

        // Apply computed layouts to scene
        // We collect updates first to avoid cloning children vectors during recursion
        let mut updates = Vec::with_capacity(node_map.len());
        collect_layout_updates(scene, root, engine, node_map, 0.0, 0.0, &mut updates);

        // Apply updates
        for (id, bounds) in updates {
            if let Some(node) = scene.get_node_mut(id) {
                node.bounds = bounds;
            }
        }
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
        // Iterate over children reference directly to avoid cloning the Vec
        // (This is safe because build_layout_tree only takes &Scene)
        for &child_id in &scene_node.children {
            build_layout_tree(scene, child_id, engine, node_map, layout_styles);

            // Add as child in layout tree
            if let Some(&child_layout) = node_map.get(&child_id) {
                engine.add_child(layout_node, child_layout);
            }
        }
    }
}

/// Collect layout updates into a buffer to avoid cloning children vectors.
fn collect_layout_updates(
    scene: &Scene,
    node_id: NodeId,
    engine: &LayoutEngine,
    node_map: &HashMap<NodeId, layout_engine::NodeId>,
    parent_x: f32,
    parent_y: f32,
    updates: &mut Vec<(NodeId, Rect)>,
) {
    if let Some(&layout_node) = node_map.get(&node_id) {
        if let Some(layout) = engine.get_layout(layout_node) {
            // Compute absolute position
            let abs_x = parent_x + layout.x;
            let abs_y = parent_y + layout.y;

            // Add update to buffer
            updates.push((
                node_id,
                Rect::new(abs_x, abs_y, layout.width, layout.height),
            ));

            // Recurse using reference to children (no clone needed)
            if let Some(scene_node) = scene.get_node(node_id) {
                for &child_id in &scene_node.children {
                    collect_layout_updates(
                        scene, child_id, engine, node_map, abs_x, abs_y, updates,
                    );
                }
            }
        }
    }
}

/// Default style for nodes without explicit layout.
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
        ..FlexStyle::default()
    }
}
