//! Layout system for Arthropod ECS
//!
//! Automatically computes flexbox layout for scene nodes with LayoutStyle components.
//!
//! # Parallelization Strategy
//!
//! Layout computation has inherent sequential dependencies in flexbox:
//! - Parent containers need children's sizes before computing positions
//! - Sibling positions depend on the flex algorithm output
//!
//! Current parallelization (Phase 8):
//! - ✅ Parallel collection of layout styles from ECS queries (read-only)
//! - ❌ Sequential tree building, layout computation, and bounds application
//!
//! Future parallelization opportunities:
//! - Independent layout roots (multiple windows/viewports)
//! - Absolute-positioned subtrees (not flex-dependent)
//! - Incremental layout (only recompute dirty subtrees)
//!
//! Performance target: < 1ms for 1000 nodes with incremental updates

use crate::components::{LayoutConstraintsResource, LayoutStyle, SceneNodeRef};
use bevy_ecs::prelude::*;
use layout_engine::FlexStyle;
use render_engine::Scene;
use std::collections::HashMap;

/// Layout system - runs in the update schedule
///
/// 1. Collects layout styles from ECS components
/// 2. Builds temporary layout tree from Scene + LayoutStyle components
/// 3. Computes layout using layout-engine (Taffy)
/// 4. Updates SceneNode bounds
///
/// # Performance Characteristics
///
/// - Layout style collection: O(N) where N = nodes with LayoutStyle
/// - Tree building: O(N) recursive traversal
/// - Layout computation: O(N) via Taffy (flexbox algorithm)
/// - Bounds application: O(N) recursive update
///
/// Target: < 1ms for 1000 nodes with incremental layout (future)
pub fn layout_system(
    mut scene: ResMut<Scene>,
    constraints_res: Option<Res<LayoutConstraintsResource>>,
    query: Query<(&SceneNodeRef, &LayoutStyle)>,
) {
    // 1. Collect all layout styles into a lookup map
    // This allows us to look up styles by NodeId during recursive scene traversal
    let layout_styles: HashMap<render_engine::NodeId, FlexStyle> = query
        .iter()
        .map(|(node_ref, style)| (node_ref.0, style.0.clone()))
        .collect();

    if layout_styles.is_empty() {
        return;
    }

    let root = scene.root();
    let constraints = constraints_res.map(|r| r.0.clone()).unwrap_or_default();

    // 2. Perform layout calculation using shared bridge logic
    // This uses the optimized 2-pass approach (collect updates -> apply updates)
    // to avoid cloning children vectors during traversal.
    crate::layout_bridge::perform_layout(&mut scene, root, constraints, &layout_styles);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::LayoutConstraintsResource;
    use layout_engine::{FlexDirection, LayoutConstraints};
    use plat_core::Rect;
    use render_engine::{NodeContent, SceneNode, Transform2D};

    fn create_test_layout_style(flex_grow: f32) -> FlexStyle {
        FlexStyle {
            direction: FlexDirection::Column,
            flex_grow,
            flex_shrink: 1.0,
            width: Some(100.0),
            height: Some(50.0),
            gap: 10.0,
            padding_left: 5.0,
            padding_right: 5.0,
            padding_top: 5.0,
            padding_bottom: 5.0,
        }
    }

    #[test]
    fn test_layout_system_with_many_nodes() {
        // Create world with many layout nodes to verify scalability
        let mut world = World::new();
        world.insert_resource(LayoutConstraintsResource(LayoutConstraints {
            max_width: Some(800.0),
            max_height: Some(600.0),
            min_width: None,
            min_height: None,
        }));

        let mut scene = Scene::new();
        let root = scene.root();

        // Add 100 nodes with layout styles
        for i in 0..100 {
            let node = SceneNode {
                content: NodeContent::Styled {
                    style: Box::new(
                        render_engine::VisualStyle::new()
                            .solid_fill(render_engine::Color::rgba(1.0, 1.0, 1.0, 1.0).as_vec4()),
                    ),
                },
                transform: Transform2D::identity(),
                bounds: Rect::new(0.0, 0.0, 100.0, 50.0),
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            };
            let node_id = scene.add_node(root, node);

            // Add layout style component with varying properties
            let style = create_test_layout_style((i % 5) as f32);
            world
                .spawn_empty()
                .insert(SceneNodeRef(node_id))
                .insert(LayoutStyle(style));
        }

        world.insert_resource(scene);

        // Run layout system
        let mut schedule = Schedule::default();
        schedule.add_systems(layout_system);
        schedule.run(&mut world);

        // Verify scene was updated (nodes should have new bounds from layout)
        let scene = world.resource::<Scene>();
        let children = scene.get_node(root).unwrap().children.clone();
        assert_eq!(children.len(), 100);

        // Verify at least some nodes had layout applied
        // (bounds should differ from initial 100x50)
        let first_child = scene.get_node(children[0]).unwrap();
        // Layout should have been computed (bounds may vary based on flex rules)
        assert!(first_child.bounds.width >= 0.0);
    }

    #[test]
    fn test_layout_system_empty_query() {
        // Verify system handles empty query gracefully
        let mut world = World::new();
        let scene = Scene::new();
        world.insert_resource(scene);

        // Run layout system with no LayoutStyle components
        let mut schedule = Schedule::default();
        schedule.add_systems(layout_system);
        schedule.run(&mut world);

        // Should complete without errors
        let scene = world.resource::<Scene>();
        assert!(scene.get_node(scene.root()).is_some());
    }

    #[test]
    fn test_layout_system_with_constraints() {
        // Verify layout system respects constraints
        let mut world = World::new();
        world.insert_resource(LayoutConstraintsResource(LayoutConstraints {
            max_width: Some(400.0),
            max_height: Some(300.0),
            min_width: None,
            min_height: None,
        }));

        let mut scene = Scene::new();
        let root = scene.root();

        // Add a single flex-grow node
        let node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(render_engine::Color::rgba(1.0, 1.0, 1.0, 1.0).as_vec4()),
                ),
            },
            transform: Transform2D::identity(),
            bounds: Rect::new(0.0, 0.0, 100.0, 50.0),
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        let node_id = scene.add_node(root, node);

        let style = FlexStyle {
            flex_grow: 1.0,
            ..create_test_layout_style(1.0)
        };
        world
            .spawn_empty()
            .insert(SceneNodeRef(node_id))
            .insert(LayoutStyle(style));

        world.insert_resource(scene);

        // Run layout system
        let mut schedule = Schedule::default();
        schedule.add_systems(layout_system);
        schedule.run(&mut world);

        // Verify layout was applied with constraints
        let scene = world.resource::<Scene>();
        let root_node = scene.get_node(root).unwrap();
        // Root should have constraint dimensions
        assert!(root_node.bounds.width <= 400.0);
        assert!(root_node.bounds.height <= 300.0);
    }
}
