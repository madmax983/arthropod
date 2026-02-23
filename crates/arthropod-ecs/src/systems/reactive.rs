use bevy_ecs::prelude::*;
use render_engine::{NodeContent, Scene};

use crate::components::{
    ReactiveColor, ReactiveComputedText, ReactiveOpacity, ReactiveText, ReactiveTransform,
    SceneNodeRef,
};

/// Merged reactive update system - polls all reactive signals in a single pass
///
/// This system combines color, text, transform, and opacity reactive updates into one
/// system, reducing scheduling overhead and acquiring `ResMut<Scene>` only once
/// instead of multiple times.
///
/// Replaces the individual `update_reactive_colors_system`, `update_reactive_text_system`,
/// `update_reactive_transforms_system`, and `update_reactive_opacity_system`.
pub fn update_all_reactive_system(
    color_query: Query<(&SceneNodeRef, &ReactiveColor)>,
    text_query: Query<(&SceneNodeRef, &ReactiveText)>,
    computed_text_query: Query<(&SceneNodeRef, &ReactiveComputedText)>,
    transform_query: Query<(&SceneNodeRef, &ReactiveTransform)>,
    opacity_query: Query<(&SceneNodeRef, &ReactiveOpacity)>,
    mut scene: ResMut<Scene>,
) {
    // Colors
    for (node_ref, reactive) in color_query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            let new_color = reactive.signal.get_untracked();
            if let NodeContent::Styled { ref mut style } = node.content {
                if !style.fills.is_empty() {
                    style.fills[0] = render_engine::Paint::Solid(new_color.as_vec4());
                } else {
                    style
                        .fills
                        .push(render_engine::Paint::Solid(new_color.as_vec4()));
                }
            }
        }
    }

    // Text content
    for (node_ref, reactive) in text_query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            let new_text = reactive.signal.get_untracked();
            if let NodeContent::Styled { ref mut style } = node.content {
                if let Some(ref mut text_content) = style.text {
                    text_content.text = new_text;
                }
            }
        }
    }

    // Computed text content
    for (node_ref, reactive) in computed_text_query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            let new_text = reactive.computed.get();
            if let NodeContent::Styled { ref mut style } = node.content {
                if let Some(ref mut text_content) = style.text {
                    text_content.text = new_text;
                }
            }
        }
    }

    // Transforms
    for (node_ref, reactive) in transform_query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            node.transform = reactive.signal.get_untracked();
        }
    }

    // Opacity
    for (node_ref, reactive) in opacity_query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            node.opacity = reactive.signal.get_untracked();
        }
    }
}

/// Update scene node colors from reactive signals
///
/// This system queries all entities with ReactiveColor components and updates
/// the corresponding scene nodes' color properties by polling the signals.
///
/// Scene is now accessed as a safe Resource - no more unsafe pointer juggling!
#[deprecated(
    since = "0.2.0",
    note = "Use `update_all_reactive_system` instead, which merges all reactive updates into a single pass"
)]
pub fn update_reactive_colors_system(
    query: Query<(&SceneNodeRef, &ReactiveColor)>,
    mut scene: ResMut<Scene>,
) {
    for (node_ref, reactive) in query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            let new_color = reactive.signal.get_untracked();
            if let NodeContent::Styled { ref mut style } = node.content {
                if !style.fills.is_empty() {
                    style.fills[0] = render_engine::Paint::Solid(new_color.as_vec4());
                } else {
                    style
                        .fills
                        .push(render_engine::Paint::Solid(new_color.as_vec4()));
                }
            }
        }
    }
}

/// Update scene node transforms from reactive signals
///
/// This system queries all entities with ReactiveTransform components and updates
/// the corresponding scene nodes' transform properties by polling the signals.
#[deprecated(
    since = "0.2.0",
    note = "Use `update_all_reactive_system` instead, which merges all reactive updates into a single pass"
)]
pub fn update_reactive_transforms_system(
    query: Query<(&SceneNodeRef, &ReactiveTransform)>,
    mut scene: ResMut<Scene>,
) {
    for (node_ref, reactive) in query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            node.transform = reactive.signal.get_untracked();
        }
    }
}

/// Update scene node opacity from reactive signals
///
/// This system queries all entities with ReactiveOpacity components and updates
/// the corresponding scene nodes' opacity properties by polling the signals.
#[deprecated(
    since = "0.2.0",
    note = "Use `update_all_reactive_system` instead, which merges all reactive updates into a single pass"
)]
pub fn update_reactive_opacity_system(
    query: Query<(&SceneNodeRef, &ReactiveOpacity)>,
    mut scene: ResMut<Scene>,
) {
    for (node_ref, reactive) in query.iter() {
        if let Some(node) = scene.get_mut(node_ref.0) {
            node.opacity = reactive.signal.get_untracked();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};
    use plat_core::Rect;
    use render_engine::{Color, SceneNode, Transform2D};

    /// Test that the merged system updates all three property types in a single pass
    #[test]
    fn test_merged_system_updates_all_properties() {
        let mut world = World::new();

        let runtime = Runtime::new();

        // Create signals for each property type
        let color_signal = Signal::new(runtime.clone(), Color::RED);
        let (color_read, color_write) = color_signal.split();

        let transform_signal = Signal::new(runtime.clone(), Transform2D::identity());
        let (transform_read, transform_write) = transform_signal.split();

        let opacity_signal = Signal::new(runtime.clone(), 1.0_f32);
        let (opacity_read, opacity_write) = opacity_signal.split();

        // Setup scene
        let mut scene = Scene::new();
        let root = scene.root();
        let node_id = scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Styled {
                    style: Box::new(
                        render_engine::VisualStyle::new().solid_fill(Color::RED.as_vec4()),
                    ),
                },
                transform: Transform2D::identity(),
                bounds: Rect::new(0.0, 0.0, 100.0, 100.0),
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            },
        );

        world.insert_resource(scene);

        // Spawn entity with all three reactive components
        world.spawn((
            SceneNodeRef(node_id),
            ReactiveColor::new(color_read),
            ReactiveTransform::new(transform_read),
            ReactiveOpacity::new(opacity_read),
        ));

        // Update signals
        color_write.set(Color::BLUE);
        transform_write.set(Transform2D::translate(50.0, 75.0));
        opacity_write.set(0.5);

        // Run merged system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_all_reactive_system);
        schedule.run(&mut world);

        // Verify all three properties updated
        let scene = world.resource::<Scene>();
        let node = scene.get_node(node_id).unwrap();

        // Color should be BLUE
        match &node.content {
            NodeContent::Styled { style } => {
                if let Some(render_engine::Paint::Solid(color)) = style.fills.first() {
                    assert!(
                        (color.x - Color::BLUE.r()).abs() < 0.001
                            && (color.z - Color::BLUE.b()).abs() < 0.001,
                        "Color should be BLUE"
                    );
                } else {
                    panic!("Expected solid fill");
                }
            }
            _ => panic!("Expected Styled node"),
        }

        // Transform should be translated
        let point = glam::Vec2::new(0.0, 0.0);
        let result = node.transform.transform_point(point);
        assert!((result.x - 50.0).abs() < 0.001, "Transform X mismatch");
        assert!((result.y - 75.0).abs() < 0.001, "Transform Y mismatch");

        // Opacity should be 0.5
        assert!((node.opacity - 0.5).abs() < 0.001, "Opacity mismatch");
    }

    /// Test merged system with entities having only some reactive components
    #[test]
    fn test_merged_system_partial_components() {
        let mut world = World::new();
        let runtime = Runtime::new();

        // Entity 1: only color
        let color_signal = Signal::new(runtime.clone(), Color::RED);
        let (color_read, color_write) = color_signal.split();

        // Entity 2: only opacity
        let opacity_signal = Signal::new(runtime.clone(), 1.0_f32);
        let (opacity_read, opacity_write) = opacity_signal.split();

        let mut scene = Scene::new();
        let root = scene.root();

        let node1 = scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Styled {
                    style: Box::new(
                        render_engine::VisualStyle::new().solid_fill(Color::GREEN.as_vec4()),
                    ),
                },
                transform: Transform2D::identity(),
                bounds: Rect::new(0.0, 0.0, 50.0, 50.0),
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            },
        );

        let node2 = scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Empty,
                transform: Transform2D::identity(),
                bounds: Rect::default(),
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            },
        );

        world.insert_resource(scene);

        world.spawn((SceneNodeRef(node1), ReactiveColor::new(color_read)));
        world.spawn((SceneNodeRef(node2), ReactiveOpacity::new(opacity_read)));

        // Update signals
        color_write.set(Color::BLUE);
        opacity_write.set(0.3);

        // Run merged system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_all_reactive_system);
        schedule.run(&mut world);

        let scene = world.resource::<Scene>();

        // Node 1: color changed, opacity unchanged
        let n1 = scene.get_node(node1).unwrap();
        match &n1.content {
            NodeContent::Styled { style } => {
                if let Some(render_engine::Paint::Solid(color)) = style.fills.first() {
                    assert!((color.z - 1.0).abs() < 0.001, "Node1 color should be BLUE");
                } else {
                    panic!("Expected solid fill");
                }
            }
            _ => panic!("Expected Styled node"),
        }
        assert!((n1.opacity - 1.0).abs() < 0.001, "Node1 opacity unchanged");

        // Node 2: opacity changed
        let n2 = scene.get_node(node2).unwrap();
        assert!(
            (n2.opacity - 0.3).abs() < 0.001,
            "Node2 opacity should be 0.3"
        );
    }
}
