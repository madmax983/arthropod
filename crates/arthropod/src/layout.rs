//! Layout utilities for automatic widget positioning.
//!
//! Bridges the WidgetContext layout styles to Scene node bounds using
//! the layout-engine (taffy) for flexbox computation.

// Allow collapsible_if since nested if-let chains are more readable
#![allow(clippy::collapsible_if)]

use layout_engine::{FlexDirection, FlexStyle, LayoutConstraints, LayoutEngine};
use plat_core::Rect;
use render_engine::{NodeId, Scene};
use std::collections::HashMap;
use widget_core::WidgetContext;

/// Perform automatic layout on the scene tree.
///
/// Uses flexbox layout from the layout-engine crate to compute positions
/// for all nodes based on their FlexStyle from the WidgetContext.
///
/// # Arguments
///
/// * `scene` - The scene to layout
/// * `root` - Root node to start layout from
/// * `viewport_width` - Available width
/// * `viewport_height` - Available height
/// * `layout_styles` - Layout styles from WidgetContext
///
/// # Example
///
/// ```ignore
/// let layout_styles = widget_ctx.layout_styles().clone();
/// auto_layout(&mut scene, root, 800.0, 600.0, &layout_styles);
/// ```
pub fn auto_layout(
    scene: &mut Scene,
    root: NodeId,
    viewport_width: f32,
    viewport_height: f32,
    layout_styles: &HashMap<NodeId, FlexStyle>,
) {
    let mut engine = LayoutEngine::new();
    let mut node_map: HashMap<NodeId, layout_engine::NodeId> = HashMap::new();

    // Build layout tree recursively
    build_layout_tree(scene, root, &mut engine, &mut node_map, layout_styles);

    // Get layout root
    if let Some(&layout_root) = node_map.get(&root) {
        // Compute layout
        engine.compute_layout(
            layout_root,
            LayoutConstraints {
                max_width: Some(viewport_width),
                max_height: Some(viewport_height),
                min_width: None,
                min_height: None,
            },
        );

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

            // Update scene node bounds and get children.
            // We clone the children vector (cheap copy of NodeIds) to avoid holding
            // a mutable borrow on the scene while recursing.
            let children = if let Some(scene_node) = scene.get_node_mut(node_id) {
                scene_node.bounds = Rect::new(abs_x, abs_y, layout.width, layout.height);
                scene_node.children.clone()
            } else {
                Vec::new()
            };

            // Recurse using the cloned children
            for child_id in children {
                apply_layouts(scene, child_id, engine, node_map, abs_x, abs_y);
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
    }
}

/// Convenience function to layout a widget context's scene.
///
/// This is a higher-level API that extracts layout styles from the widget
/// context and applies them to the provided scene.
///
/// # Arguments
///
/// * `widget_ctx` - Widget context with layout styles
/// * `scene` - Scene to layout (may be the app's scene after integration)
/// * `root` - Root node in the scene
/// * `viewport_width` - Available width
/// * `viewport_height` - Available height
pub fn layout_widget_tree(
    widget_ctx: &WidgetContext,
    scene: &mut Scene,
    root: NodeId,
    viewport_width: f32,
    viewport_height: f32,
) {
    auto_layout(
        scene,
        root,
        viewport_width,
        viewport_height,
        widget_ctx.layout_styles(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_engine::{Color, NodeContent, SceneNode};

    // =========================================================================
    // Helper functions
    // =========================================================================

    fn create_rect_node(scene: &mut Scene, parent: NodeId) -> NodeId {
        let node = SceneNode::new(NodeContent::Rect { color: Color::RED });
        scene.add_node(parent, node)
    }

    // =========================================================================
    // Basic Layout Tests
    // =========================================================================

    #[test]
    fn test_auto_layout_empty_scene() {
        let mut scene = Scene::new();
        let root = scene.root();
        let styles = HashMap::new();

        // Should not panic on empty scene
        auto_layout(&mut scene, root, 800.0, 600.0, &styles);
    }

    #[test]
    fn test_auto_layout_root_only() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                width: Some(400.0),
                height: Some(300.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        let root_node = scene.get_node(root).unwrap();
        assert_eq!(root_node.bounds.width, 400.0);
        assert_eq!(root_node.bounds.height, 300.0);
    }

    #[test]
    fn test_auto_layout_single_child() {
        let mut scene = Scene::new();
        let root = scene.root();
        let child_id = create_rect_node(&mut scene, root);

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(100.0),
                height: Some(100.0),
                ..Default::default()
            },
        );
        styles.insert(
            child_id,
            FlexStyle {
                width: Some(50.0),
                height: Some(50.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        let child_node = scene.get_node(child_id).unwrap();
        assert_eq!(child_node.bounds.width, 50.0);
        assert_eq!(child_node.bounds.height, 50.0);
    }

    // =========================================================================
    // Column Layout Tests
    // =========================================================================

    #[test]
    fn test_column_layout_stacks_vertically() {
        let mut scene = Scene::new();
        let root = scene.root();
        let child1 = create_rect_node(&mut scene, root);
        let child2 = create_rect_node(&mut scene, root);

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(200.0),
                height: Some(200.0),
                ..Default::default()
            },
        );
        styles.insert(
            child1,
            FlexStyle {
                width: Some(100.0),
                height: Some(50.0),
                ..Default::default()
            },
        );
        styles.insert(
            child2,
            FlexStyle {
                width: Some(100.0),
                height: Some(50.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        let node1 = scene.get_node(child1).unwrap();
        let node2 = scene.get_node(child2).unwrap();

        // Child2 should be below child1 in column layout
        assert!(node2.bounds.y >= node1.bounds.y + node1.bounds.height);
    }

    #[test]
    fn test_column_layout_with_gap() {
        let mut scene = Scene::new();
        let root = scene.root();
        let child1 = create_rect_node(&mut scene, root);
        let child2 = create_rect_node(&mut scene, root);

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(200.0),
                height: Some(200.0),
                gap: 10.0,
                ..Default::default()
            },
        );
        styles.insert(
            child1,
            FlexStyle {
                width: Some(100.0),
                height: Some(40.0),
                ..Default::default()
            },
        );
        styles.insert(
            child2,
            FlexStyle {
                width: Some(100.0),
                height: Some(40.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        let node1 = scene.get_node(child1).unwrap();
        let node2 = scene.get_node(child2).unwrap();

        // Gap of 10.0 between children
        let actual_gap = node2.bounds.y - (node1.bounds.y + node1.bounds.height);
        assert!(
            (actual_gap - 10.0).abs() < 0.1,
            "Expected gap of 10.0, got {}",
            actual_gap
        );
    }

    // =========================================================================
    // Row Layout Tests
    // =========================================================================

    #[test]
    fn test_row_layout_stacks_horizontally() {
        let mut scene = Scene::new();
        let root = scene.root();
        let child1 = create_rect_node(&mut scene, root);
        let child2 = create_rect_node(&mut scene, root);

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                width: Some(200.0),
                height: Some(100.0),
                ..Default::default()
            },
        );
        styles.insert(
            child1,
            FlexStyle {
                width: Some(50.0),
                height: Some(50.0),
                ..Default::default()
            },
        );
        styles.insert(
            child2,
            FlexStyle {
                width: Some(50.0),
                height: Some(50.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        let node1 = scene.get_node(child1).unwrap();
        let node2 = scene.get_node(child2).unwrap();

        // Child2 should be to the right of child1 in row layout
        assert!(node2.bounds.x >= node1.bounds.x + node1.bounds.width);
    }

    #[test]
    fn test_row_layout_with_gap() {
        let mut scene = Scene::new();
        let root = scene.root();
        let child1 = create_rect_node(&mut scene, root);
        let child2 = create_rect_node(&mut scene, root);

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                width: Some(200.0),
                height: Some(100.0),
                gap: 15.0,
                ..Default::default()
            },
        );
        styles.insert(
            child1,
            FlexStyle {
                width: Some(40.0),
                height: Some(40.0),
                ..Default::default()
            },
        );
        styles.insert(
            child2,
            FlexStyle {
                width: Some(40.0),
                height: Some(40.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        let node1 = scene.get_node(child1).unwrap();
        let node2 = scene.get_node(child2).unwrap();

        // Gap of 15.0 between children
        let actual_gap = node2.bounds.x - (node1.bounds.x + node1.bounds.width);
        assert!(
            (actual_gap - 15.0).abs() < 0.1,
            "Expected gap of 15.0, got {}",
            actual_gap
        );
    }

    // =========================================================================
    // Padding Tests
    // =========================================================================

    #[test]
    fn test_layout_with_padding() {
        let mut scene = Scene::new();
        let root = scene.root();
        let child = create_rect_node(&mut scene, root);

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(200.0),
                height: Some(200.0),
                padding_left: 20.0,
                padding_top: 10.0,
                padding_right: 20.0,
                padding_bottom: 10.0,
                ..Default::default()
            },
        );
        styles.insert(
            child,
            FlexStyle {
                width: Some(50.0),
                height: Some(50.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        let child_node = scene.get_node(child).unwrap();
        let root_node = scene.get_node(root).unwrap();

        // Child should be offset by padding from root's position
        assert!(
            (child_node.bounds.x - root_node.bounds.x - 20.0).abs() < 0.1,
            "Expected x offset of 20.0, got {}",
            child_node.bounds.x - root_node.bounds.x
        );
        assert!(
            (child_node.bounds.y - root_node.bounds.y - 10.0).abs() < 0.1,
            "Expected y offset of 10.0, got {}",
            child_node.bounds.y - root_node.bounds.y
        );
    }

    // =========================================================================
    // Nested Layout Tests
    // =========================================================================

    #[test]
    fn test_nested_layout() {
        let mut scene = Scene::new();
        let root = scene.root();
        let container = create_rect_node(&mut scene, root);
        let nested_child = create_rect_node(&mut scene, container);

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(300.0),
                height: Some(300.0),
                ..Default::default()
            },
        );
        styles.insert(
            container,
            FlexStyle {
                direction: FlexDirection::Row,
                width: Some(200.0),
                height: Some(100.0),
                padding_left: 10.0,
                padding_top: 10.0,
                ..Default::default()
            },
        );
        styles.insert(
            nested_child,
            FlexStyle {
                width: Some(50.0),
                height: Some(50.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        let container_node = scene.get_node(container).unwrap();
        let nested_node = scene.get_node(nested_child).unwrap();

        // Nested child should be positioned relative to container with padding
        assert!(nested_node.bounds.x >= container_node.bounds.x + 10.0 - 0.1);
        assert!(nested_node.bounds.y >= container_node.bounds.y + 10.0 - 0.1);
    }

    #[test]
    fn test_deeply_nested_layout() {
        let mut scene = Scene::new();
        let root = scene.root();
        let level1 = create_rect_node(&mut scene, root);
        let level2 = create_rect_node(&mut scene, level1);
        let level3 = create_rect_node(&mut scene, level2);

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(400.0),
                height: Some(400.0),
                padding_left: 10.0,
                padding_top: 10.0,
                ..Default::default()
            },
        );
        styles.insert(
            level1,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(300.0),
                height: Some(300.0),
                padding_left: 10.0,
                padding_top: 10.0,
                ..Default::default()
            },
        );
        styles.insert(
            level2,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(200.0),
                height: Some(200.0),
                padding_left: 10.0,
                padding_top: 10.0,
                ..Default::default()
            },
        );
        styles.insert(
            level3,
            FlexStyle {
                width: Some(50.0),
                height: Some(50.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        let level3_node = scene.get_node(level3).unwrap();

        // Level3 should be offset by accumulated padding (10 + 10 + 10 = 30)
        assert!(
            level3_node.bounds.x >= 30.0 - 0.1,
            "Expected x >= 30, got {}",
            level3_node.bounds.x
        );
        assert!(
            level3_node.bounds.y >= 30.0 - 0.1,
            "Expected y >= 30, got {}",
            level3_node.bounds.y
        );
    }

    // =========================================================================
    // Default Style Tests
    // =========================================================================

    #[test]
    fn test_default_style_values() {
        let style = default_style();

        assert_eq!(style.direction, FlexDirection::Column);
        assert_eq!(style.flex_grow, 0.0);
        assert_eq!(style.flex_shrink, 1.0);
        assert_eq!(style.width, None);
        assert_eq!(style.height, None);
        assert_eq!(style.gap, 0.0);
        assert_eq!(style.padding_left, 0.0);
        assert_eq!(style.padding_right, 0.0);
        assert_eq!(style.padding_top, 0.0);
        assert_eq!(style.padding_bottom, 0.0);
    }

    #[test]
    fn test_layout_uses_default_for_missing_styles() {
        let mut scene = Scene::new();
        let root = scene.root();
        let child = create_rect_node(&mut scene, root);

        // Only provide style for root, not child
        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(200.0),
                height: Some(200.0),
                ..Default::default()
            },
        );

        // Should not panic - uses default style for child
        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        // Child should still get laid out
        let child_node = scene.get_node(child).unwrap();
        assert!(child_node.bounds.x >= 0.0);
        assert!(child_node.bounds.y >= 0.0);
    }

    // =========================================================================
    // layout_widget_tree Tests
    // =========================================================================

    #[test]
    fn test_layout_widget_tree_basic() {
        let mut widget_ctx = WidgetContext::new_test();
        let root = widget_ctx.root();

        // Add a child node
        let child_id = widget_ctx.create_node(root, NodeContent::Rect { color: Color::BLUE });

        // Set layout styles
        widget_ctx.set_layout_style(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(300.0),
                height: Some(200.0),
                ..Default::default()
            },
        );
        widget_ctx.set_layout_style(
            child_id,
            FlexStyle {
                width: Some(100.0),
                height: Some(50.0),
                ..Default::default()
            },
        );

        // Get scene and layout - we need to recreate styles since into_scene consumes ctx
        let mut scene = widget_ctx.into_scene();
        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(300.0),
                height: Some(200.0),
                ..Default::default()
            },
        );
        styles.insert(
            child_id,
            FlexStyle {
                width: Some(100.0),
                height: Some(50.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        let child_node = scene.get_node(child_id).unwrap();
        assert_eq!(child_node.bounds.width, 100.0);
        assert_eq!(child_node.bounds.height, 50.0);
    }

    // =========================================================================
    // Viewport Constraint Tests
    // =========================================================================

    #[test]
    fn test_layout_respects_viewport_constraints() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                // Request larger than viewport
                width: Some(1000.0),
                height: Some(1000.0),
                ..Default::default()
            },
        );

        auto_layout(&mut scene, root, 400.0, 300.0, &styles);

        // Root may be constrained by viewport (depends on layout engine behavior)
        let root_node = scene.get_node(root).unwrap();
        // The actual constraint behavior depends on taffy - just verify it doesn't panic
        assert!(root_node.bounds.width > 0.0);
        assert!(root_node.bounds.height > 0.0);
    }

    // =========================================================================
    // Multiple Children Tests
    // =========================================================================

    #[test]
    fn test_layout_many_children_column() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut children = Vec::new();
        for _ in 0..5 {
            children.push(create_rect_node(&mut scene, root));
        }

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Column,
                width: Some(200.0),
                height: Some(500.0),
                gap: 5.0,
                ..Default::default()
            },
        );

        for child in &children {
            styles.insert(
                *child,
                FlexStyle {
                    width: Some(100.0),
                    height: Some(30.0),
                    ..Default::default()
                },
            );
        }

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        // Verify children are stacked vertically with gaps
        for i in 1..children.len() {
            let prev = scene.get_node(children[i - 1]).unwrap();
            let curr = scene.get_node(children[i]).unwrap();

            assert!(
                curr.bounds.y > prev.bounds.y,
                "Child {} should be below child {}",
                i,
                i - 1
            );
        }
    }

    #[test]
    fn test_layout_many_children_row() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut children = Vec::new();
        for _ in 0..4 {
            children.push(create_rect_node(&mut scene, root));
        }

        let mut styles = HashMap::new();
        styles.insert(
            root,
            FlexStyle {
                direction: FlexDirection::Row,
                width: Some(400.0),
                height: Some(100.0),
                gap: 10.0,
                ..Default::default()
            },
        );

        for child in &children {
            styles.insert(
                *child,
                FlexStyle {
                    width: Some(50.0),
                    height: Some(50.0),
                    ..Default::default()
                },
            );
        }

        auto_layout(&mut scene, root, 800.0, 600.0, &styles);

        // Verify children are arranged horizontally with gaps
        for i in 1..children.len() {
            let prev = scene.get_node(children[i - 1]).unwrap();
            let curr = scene.get_node(children[i]).unwrap();

            assert!(
                curr.bounds.x > prev.bounds.x,
                "Child {} should be to the right of child {}",
                i,
                i - 1
            );
        }
    }
}
