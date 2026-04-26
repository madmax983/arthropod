use bevy_ecs::prelude::*;
use render_engine::{NodeContent, Scene};

use crate::components::{
    InteractionState, LayoutStyle, MousePosition, ProgressBarState, ReactiveColor,
    ReactiveComputedText, ReactiveLayoutFlexGrow, ReactiveLayoutWidth, ReactiveOpacity,
    ReactiveText, ReactiveTransform, SceneNodeRef, WidgetStyle,
};

/// Resource that provides access to the reactive runtime within ECS
#[derive(Resource, Clone)]
pub struct RuntimeResource(pub std::sync::Arc<flux_state::Runtime>);

/// System that runs all pending reactive effects
pub fn run_reactive_effects_system(runtime: Res<RuntimeResource>) {
    runtime.0.run_effects();
}

/// System that updates interaction states based on mouse position and hit testing.
///
/// Optimization: Uses `Local<HashSet>` to track hovered nodes instead of allocating
/// a new HashSet on every single frame. This prevents continuous heap allocations
/// and improves consistent frame times.
pub fn update_interaction_state_system(
    mouse_pos: Res<MousePosition>,
    scene: Res<Scene>,
    mut query: Query<(Entity, &SceneNodeRef, &mut InteractionState)>,
    mut hovered_nodes: Local<std::collections::HashSet<render_engine::NodeId>>,
) {
    let hit_id = scene.hit_test(mouse_pos.0.x, mouse_pos.0.y);

    // Create a set of interactive nodes that are actually being hovered.
    // By clearing the local HashSet, we reuse its allocated memory block.
    hovered_nodes.clear();

    if let Some(mut current_id) = hit_id {
        // Bubble up from the hit node to find all interactive ancestors
        // This ensures that if you hover a Text inside a Button, the Button is also considered hovered.
        loop {
            if !hovered_nodes.insert(current_id) {
                break; // Prevent infinite loop DoS in case of cyclic parent structures
            }
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

/// Query type for resolving widget styles based on interaction state
/// A consolidated ECS query mapping standard structural nodes to their unified presentation states and any active user-interaction states.
pub type WidgetStyleQuery<'w> = (
    &'w SceneNodeRef,
    &'w WidgetStyle,
    &'w InteractionState,
    Option<&'w mut LayoutStyle>,
);

/// Filter type for running widget style updates only when necessary
/// An ECS filter ensuring the heavy style resolution logic is completely skipped unless a widget actually changed state or configuration.
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
/// Query type for gathering all reactive components in a single pass
/// A massive "catch-all" query grouping all potential reactive properties into a single ECS pass, drastically cutting down iteration overhead.
pub type ReactiveQuery<'w> = (
    &'w SceneNodeRef,
    Option<&'w mut LayoutStyle>,
    Option<&'w mut ReactiveColor>,
    Option<&'w mut ReactiveText>,
    Option<&'w mut ReactiveComputedText>,
    Option<&'w mut ReactiveTransform>,
    Option<&'w mut ReactiveOpacity>,
    Option<&'w mut ReactiveLayoutWidth>,
    Option<&'w mut ReactiveLayoutFlexGrow>,
    Option<&'w mut ProgressBarState>,
);

fn update_layout_width(
    layout: &mut Option<Mut<'_, LayoutStyle>>,
    width: &mut Option<Mut<'_, ReactiveLayoutWidth>>,
) {
    if let Some(val) = width.as_mut() {
        let new_width = val.signal.get_untracked();
        let delta = (new_width - val.last_value).abs();

        if delta >= 0.0001 {
            val.last_value = new_width;
            if let Some(l) = layout.as_mut() {
                l.0.width = Some(new_width);
            }
        }
    }
}

fn update_progress_bar_width(
    layout: &mut Option<Mut<'_, LayoutStyle>>,
    progress_bar: &mut Option<Mut<'_, ProgressBarState>>,
) {
    if let Some(val) = progress_bar.as_mut() {
        let prog = val.progress.get_untracked().clamp(0.0, 1.0);
        let delta = (prog - val.last_progress).abs();
        if delta >= 0.0001 {
            let total_width = val.total_width;
            val.last_progress = prog;
            if let Some(l) = layout.as_mut() {
                l.0.width = Some(prog * total_width);
            }
        }
    }
}

fn update_layout_flex_grow(
    layout: &mut Option<Mut<'_, LayoutStyle>>,
    flex_grow: &mut Option<Mut<'_, ReactiveLayoutFlexGrow>>,
) {
    if let Some(val) = flex_grow.as_mut() {
        let new_grow = val.signal.get_untracked();
        if (new_grow - val.last_value).abs() >= f32::EPSILON {
            val.last_value = new_grow;
            if let Some(l) = layout.as_mut() {
                l.0.flex_grow = new_grow;
            }
        }
    }
}

fn update_color(
    scene: &mut ResMut<'_, Scene>,
    node_id: render_engine::NodeId,
    color: &mut Option<Mut<'_, ReactiveColor>>,
) {
    if let Some(val) = color.as_mut() {
        let new_color = val.signal.get_untracked();
        if new_color != val.last_value {
            val.last_value = new_color;

            if let Some(style) = scene
                .get_mut(node_id)
                .and_then(|node| match &mut node.content {
                    NodeContent::Styled { style } => Some(style),
                    _ => None,
                })
            {
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

fn update_text(
    scene: &mut ResMut<'_, Scene>,
    node_id: render_engine::NodeId,
    text: &mut Option<Mut<'_, ReactiveText>>,
) {
    if let Some(val) = text.as_mut() {
        // ⚡ Bolt: Check string equality first without allocating, only clone when changed
        let changed = val
            .signal
            .with_untracked(|new_text| *new_text != val.last_value);

        if changed {
            let new_val = val.signal.get_untracked();
            val.last_value = new_val;

            if let Some(text_content) = scene
                .get_mut(node_id)
                .and_then(|node| match &mut node.content {
                    NodeContent::Styled { style } => Some(style),
                    _ => None,
                })
                .and_then(|style| style.text.as_mut())
            {
                text_content.text.clone_from(&val.last_value);
            }
        }
    }
}

fn update_computed_text(
    scene: &mut ResMut<'_, Scene>,
    node_id: render_engine::NodeId,
    computed_text: &mut Option<Mut<'_, ReactiveComputedText>>,
) {
    if let Some(val) = computed_text.as_mut() {
        // ⚡ Bolt: Check string equality first without allocating, only clone when changed
        let changed = val
            .computed
            .with_untracked(|new_text| *new_text != val.last_value);

        if changed {
            let new_val = val.computed.get();
            val.last_value = new_val;

            if let Some(text_content) = scene
                .get_mut(node_id)
                .and_then(|node| match &mut node.content {
                    NodeContent::Styled { style } => Some(style),
                    _ => None,
                })
                .and_then(|style| style.text.as_mut())
            {
                text_content.text.clone_from(&val.last_value);
            }
        }
    }
}

fn update_transform(
    scene: &mut ResMut<'_, Scene>,
    node_id: render_engine::NodeId,
    transform: &mut Option<Mut<'_, ReactiveTransform>>,
) {
    if let Some(val) = transform.as_mut() {
        let new_transform = val.signal.get_untracked();
        if new_transform != val.last_value {
            val.last_value = new_transform;

            if let Some(node) = scene.get_mut(node_id) {
                node.transform = new_transform;
            }
        }
    }
}

fn update_opacity(
    scene: &mut ResMut<'_, Scene>,
    node_id: render_engine::NodeId,
    opacity: &mut Option<Mut<'_, ReactiveOpacity>>,
) {
    if let Some(val) = opacity.as_mut() {
        let new_opacity = val.signal.get_untracked();
        if (new_opacity - val.last_value).abs() >= f32::EPSILON {
            val.last_value = new_opacity;

            if let Some(node) = scene.get_mut(node_id) {
                node.opacity = new_opacity;
            }
        }
    }
}

/// Main system for applying all reactive component updates to the scene in one pass
/// A monolith system applying all dirty tracked state mutations to the visual tree simultaneously, eliminating redundant `Scene` resource locks.
pub fn update_all_reactive_system(mut query: Query<ReactiveQuery<'_>>, mut scene: ResMut<Scene>) {
    for (
        node_ref,
        mut layout,
        mut color,
        mut text,
        mut computed_text,
        mut transform,
        mut opacity,
        mut width,
        mut flex_grow,
        mut progress_bar,
    ) in query.iter_mut()
    {
        update_layout_width(&mut layout, &mut width);
        update_progress_bar_width(&mut layout, &mut progress_bar);
        update_layout_flex_grow(&mut layout, &mut flex_grow);
        update_color(&mut scene, node_ref.0, &mut color);
        update_text(&mut scene, node_ref.0, &mut text);
        update_computed_text(&mut scene, node_ref.0, &mut computed_text);
        update_transform(&mut scene, node_ref.0, &mut transform);
        update_opacity(&mut scene, node_ref.0, &mut opacity);
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

    #[test]
    fn test_update_interaction_state_prevents_cyclic_dos() {
        use crate::components::MousePosition;

        let mut world = World::new();

        let mut scene = Scene::new();
        let root = scene.root();

        // Create a malicious cycle manually by subverting typical add_node rules,
        // to mimic an exploit condition. We make two nodes and cross-parent them.
        let node1 = scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Empty,
                transform: Transform2D::identity(),
                bounds: Rect::new(0.0, 0.0, 100.0, 100.0),
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            },
        );

        let node2 = scene.add_node(
            node1,
            SceneNode {
                content: NodeContent::Empty,
                transform: Transform2D::identity(),
                bounds: Rect::new(0.0, 0.0, 100.0, 100.0),
                children: vec![],
                parent: Some(node1), // node2's parent is node1
                visible: true,
                opacity: 1.0,
            },
        );

        // Subvert the Scene API to create the cycle directly
        // Make node1's parent node2
        scene.get_node_mut(node1).unwrap().parent = Some(node2);

        world.insert_resource(scene);
        world.insert_resource(MousePosition(glam::Vec2::new(50.0, 50.0))); // Overlaps the bounds

        // Create entity with InteractionState
        world.spawn((SceneNodeRef(node1), InteractionState::default()));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_interaction_state_system);

        // This will hang in an infinite loop without the cycle prevention fix
        schedule.run(&mut world);

        // If it finishes, the cycle prevention worked
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

    #[test]
    fn test_merged_system_does_not_trigger_change_detection_when_unchanged() {
        let mut world = World::new();
        let runtime = Runtime::new();

        let color_signal = Signal::new(runtime.clone(), Color::RED);
        let (color_read, color_write) = color_signal.split();

        let mut scene = Scene::new();
        let root = scene.root();

        let node1 = scene.add_node(
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

        let entity = world
            .spawn((SceneNodeRef(node1), ReactiveColor::new(color_read)))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_all_reactive_system);

        // First run should trigger change (value changed from default internal last_value)
        schedule.run(&mut world);
        world.clear_trackers();

        // Second run should NOT trigger change because value is still RED
        schedule.run(&mut world);

        let mut query = world.query::<Ref<ReactiveColor>>();
        let trackers = query.get(&world, entity).unwrap();
        assert!(
            !trackers.is_changed(),
            "ReactiveColor should not be marked as changed when the signal value is identical"
        );

        // Changing value SHOULD trigger change
        color_write.set(Color::BLUE);
        schedule.run(&mut world);
        assert!(
            query.get(&world, entity).unwrap().is_changed(),
            "ReactiveColor should be marked as changed when the signal value changes"
        );
    }
}
