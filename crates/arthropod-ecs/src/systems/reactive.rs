use bevy_ecs::prelude::*;
use render_engine::{NodeContent, Scene};

use crate::components::{
    InteractionState, LayoutStyle, MousePosition, ReactiveColor, ReactiveComputedText,
    ReactiveLayoutWidth, ReactiveOpacity, ReactiveText, ReactiveTransform, SceneNodeRef,
    WidgetStyle,
};

/// System that updates interaction states based on mouse position and hit testing
pub fn update_interaction_state_system(
    mouse_pos: Res<MousePosition>,
    scene: Res<Scene>,
    mut query: Query<(Entity, &SceneNodeRef, &mut InteractionState)>,
) {
    let hit_id = scene.hit_test(mouse_pos.0.x, mouse_pos.0.y);

    // Create a set of interactive nodes that are actually being hovered
    let mut hovered_nodes = std::collections::HashSet::new();

    if let Some(mut current_id) = hit_id {
        // Bubble up from the hit node to find all interactive ancestors
        // This ensures that if you hover a Text inside a Button, the Button is also considered hovered.
        loop {
            hovered_nodes.insert(current_id);
            if let Some(parent_id) = scene.parent(current_id) {
                current_id = parent_id;
            } else {
                break;
            }
        }
    }

    for (_entity, node_ref, mut state) in query.iter_mut() {
        let is_hovered = hovered_nodes.contains(&node_ref.0);
        if state.hovered != is_hovered {
            state.hovered = is_hovered;
        }
    }
}

pub type WidgetStyleQuery<'w> = (
    &'w SceneNodeRef,
    &'w WidgetStyle,
    &'w InteractionState,
    Option<&'w mut LayoutStyle>,
);

pub type WidgetStyleFilter = Or<(Changed<InteractionState>, Changed<WidgetStyle>)>;

/// System that resolves high-level WidgetStyle into low-level SceneNode properties and LayoutStyle
/// based on the current InteractionState.
///
/// This is the core of the Unified Style System, ensuring that visual and layout properties
/// automatically update when a widget's state (hover, focus, active, disabled) changes.
pub fn update_widget_style_system(
    mut query: Query<WidgetStyleQuery<'_>, WidgetStyleFilter>,
    mut scene: ResMut<Scene>,
) {
    for (node_ref, widget_style, state, layout_style) in query.iter_mut() {
        // 1. Resolve style for current state
        let resolved =
            widget_style
                .0
                .resolve(state.hovered, state.focused, state.active, state.disabled);

        // 2. Update LayoutStyle component if present (triggers layout engine)
        if let Some(mut layout) = layout_style {
            let new_flex = resolved.to_flex_style();
            if layout.0 != new_flex {
                layout.0 = new_flex;
            }
        }

        // 3. Update VisualStyle in Scene Graph
        if let Some(node) = scene.get_mut(node_ref.0) {
            node.content = NodeContent::Styled {
                style: Box::new(resolved.to_visual_style()),
            };
        }
    }
}

/// Merged reactive update system - polls all reactive signals in a single pass
///
/// This system combines color, text, transform, and opacity reactive updates into one
/// system, reducing scheduling overhead and acquiring `ResMut<Scene>` only once
/// instead of multiple times.
///
/// Replaces the individual `update_reactive_colors_system`, `update_reactive_text_system`,
/// `update_reactive_transforms_system`, and `update_reactive_opacity_system`.
pub fn update_all_reactive_system(
    mut color_query: Query<(&SceneNodeRef, &mut ReactiveColor)>,
    mut text_query: Query<(&SceneNodeRef, &mut ReactiveText)>,
    mut computed_text_query: Query<(&SceneNodeRef, &mut ReactiveComputedText)>,
    mut transform_query: Query<(&SceneNodeRef, &mut ReactiveTransform)>,
    mut opacity_query: Query<(&SceneNodeRef, &mut ReactiveOpacity)>,
    mut layout_width_query: Query<(&mut LayoutStyle, &mut ReactiveLayoutWidth)>,
    mut scene: ResMut<Scene>,
) {
    // Layout Width
    for (mut layout, mut reactive) in layout_width_query.iter_mut() {
        let new_width = reactive.signal.get_untracked();
        if (new_width - reactive.last_value).abs() < f32::EPSILON {
            continue;
        }
        reactive.last_value = new_width;
        layout.0.width = Some(new_width);
    }

    // Colors
    for (node_ref, mut reactive) in color_query.iter_mut() {
        let new_color = reactive.signal.get_untracked();
        if new_color == reactive.last_value {
            continue;
        }
        reactive.last_value = new_color;

        if let Some(node) = scene.get_mut(node_ref.0) {
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
    for (node_ref, mut reactive) in text_query.iter_mut() {
        let new_text = reactive.signal.get_untracked();
        if new_text == reactive.last_value {
            continue;
        }
        reactive.last_value = new_text.clone();

        if let Some(node) = scene.get_mut(node_ref.0) {
            if let NodeContent::Styled { ref mut style } = node.content {
                if let Some(ref mut text_content) = style.text {
                    text_content.text = new_text;
                }
            }
        }
    }

    // Computed text content
    for (node_ref, mut reactive) in computed_text_query.iter_mut() {
        let new_text = reactive.computed.get();
        if new_text == reactive.last_value {
            continue;
        }
        reactive.last_value = new_text.clone();

        if let Some(node) = scene.get_mut(node_ref.0) {
            if let NodeContent::Styled { ref mut style } = node.content {
                if let Some(ref mut text_content) = style.text {
                    text_content.text = new_text;
                }
            }
        }
    }

    // Transforms
    for (node_ref, mut reactive) in transform_query.iter_mut() {
        let new_transform = reactive.signal.get_untracked();
        if new_transform == reactive.last_value {
            continue;
        }
        reactive.last_value = new_transform;

        if let Some(node) = scene.get_mut(node_ref.0) {
            node.transform = new_transform;
        }
    }

    // Opacity
    for (node_ref, mut reactive) in opacity_query.iter_mut() {
        let new_opacity = reactive.signal.get_untracked();
        if (new_opacity - reactive.last_value).abs() < f32::EPSILON {
            continue;
        }
        reactive.last_value = new_opacity;

        if let Some(node) = scene.get_mut(node_ref.0) {
            node.opacity = new_opacity;
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
