//! Gather-apply pattern for reactive systems
//!
//! Splits reactive updates into two phases:
//! 1. **Gather**: Polls signals without Scene access (can run in parallel with read-only systems)
//! 2. **Apply**: Writes buffered changes to Scene (sequential, fast)
//!
//! This decoupling enables the gather phase to overlap with other systems that
//! only need `Res<Scene>` (shared borrow), since gather uses no Scene access at all.

use bevy_ecs::prelude::*;
use render_engine::{Color, NodeContent, NodeId, Scene, Transform2D};

use crate::components::{ReactiveColor, ReactiveOpacity, ReactiveTransform, SceneNodeRef};

/// A buffered reactive change to be applied to the Scene.
pub enum ReactiveChange {
    /// Update node color
    Color(NodeId, Color),
    /// Update node transform
    Transform(NodeId, Transform2D),
    /// Update node opacity
    Opacity(NodeId, f32),
}

/// Resource holding buffered reactive changes between gather and apply phases.
#[derive(Resource, Default)]
pub struct ReactiveChangeBuffer {
    pub changes: Vec<ReactiveChange>,
}

/// Gather phase: polls all reactive signals and buffers changes.
///
/// This system has NO Scene access, so it cannot block other systems that
/// need `Res<Scene>` or `ResMut<Scene>`. The buffer is consumed by
/// `apply_reactive_changes_system`.
pub fn gather_reactive_changes_system(
    color_query: Query<(&SceneNodeRef, &ReactiveColor)>,
    transform_query: Query<(&SceneNodeRef, &ReactiveTransform)>,
    opacity_query: Query<(&SceneNodeRef, &ReactiveOpacity)>,
    mut buffer: ResMut<ReactiveChangeBuffer>,
) {
    buffer.changes.clear();

    for (node_ref, reactive) in color_query.iter() {
        let color = reactive.signal.inner().get_untracked();
        buffer
            .changes
            .push(ReactiveChange::Color(node_ref.0, color));
    }

    for (node_ref, reactive) in transform_query.iter() {
        let transform = reactive.signal.inner().get_untracked();
        buffer
            .changes
            .push(ReactiveChange::Transform(node_ref.0, transform));
    }

    for (node_ref, reactive) in opacity_query.iter() {
        let opacity = reactive.signal.inner().get_untracked();
        buffer
            .changes
            .push(ReactiveChange::Opacity(node_ref.0, opacity));
    }
}

/// Apply phase: writes buffered changes to the Scene.
///
/// This system consumes the change buffer and applies all modifications.
/// It requires `ResMut<Scene>` but no signal access.
pub fn apply_reactive_changes_system(buffer: Res<ReactiveChangeBuffer>, mut scene: ResMut<Scene>) {
    for change in &buffer.changes {
        match change {
            ReactiveChange::Color(node_id, new_color) => {
                if let Some(node) = scene.get_mut(*node_id) {
                    node.content = match node.content {
                        NodeContent::Rect { .. } => NodeContent::Rect { color: *new_color },
                        NodeContent::RoundedRect { corner_radius, .. } => {
                            NodeContent::RoundedRect {
                                color: *new_color,
                                corner_radius,
                            }
                        }
                        NodeContent::Text {
                            ref text,
                            font_size,
                            ..
                        } => NodeContent::Text {
                            text: text.clone(),
                            font_size,
                            color: *new_color,
                        },
                        NodeContent::Empty => NodeContent::Empty,
                    };
                }
            }
            ReactiveChange::Transform(node_id, new_transform) => {
                if let Some(node) = scene.get_mut(*node_id) {
                    node.transform = *new_transform;
                }
            }
            ReactiveChange::Opacity(node_id, new_opacity) => {
                if let Some(node) = scene.get_mut(*node_id) {
                    node.opacity = *new_opacity;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};
    use plat_core::Rect;
    use render_engine::SceneNode;

    #[test]
    fn test_gather_produces_correct_buffer_entries() {
        let mut world = World::new();
        let runtime = Runtime::new();

        let color_signal = Signal::new(runtime.clone(), Color::RED);
        let (color_read, color_write) = color_signal.split();

        let transform_signal = Signal::new(runtime.clone(), Transform2D::identity());
        let (transform_read, transform_write) = transform_signal.split();

        let opacity_signal = Signal::new(runtime.clone(), 1.0_f32);
        let (opacity_read, opacity_write) = opacity_signal.split();

        let node_id = NodeId(42);
        world.spawn((
            SceneNodeRef(node_id),
            ReactiveColor::new(color_read),
            ReactiveTransform::new(transform_read),
            ReactiveOpacity::new(opacity_read),
        ));

        world.insert_resource(ReactiveChangeBuffer::default());

        // Set new values
        color_write.set(Color::BLUE);
        transform_write.set(Transform2D::translate(10.0, 20.0));
        opacity_write.set(0.7);

        // Run gather
        let mut schedule = Schedule::default();
        schedule.add_systems(gather_reactive_changes_system);
        schedule.run(&mut world);

        let buffer = world.resource::<ReactiveChangeBuffer>();
        assert_eq!(buffer.changes.len(), 3);

        // Verify entries (order: colors, transforms, opacity)
        assert!(matches!(buffer.changes[0], ReactiveChange::Color(id, _) if id == node_id));
        assert!(matches!(buffer.changes[1], ReactiveChange::Transform(id, _) if id == node_id));
        assert!(matches!(buffer.changes[2], ReactiveChange::Opacity(id, _) if id == node_id));
    }

    #[test]
    fn test_apply_correctly_modifies_scene() {
        let mut world = World::new();

        let mut scene = Scene::new();
        let root = scene.root();
        let node_id = scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Rect {
                    color: Color::GREEN,
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

        // Manually populate buffer
        let mut buffer = ReactiveChangeBuffer::default();
        buffer
            .changes
            .push(ReactiveChange::Color(node_id, Color::BLUE));
        buffer.changes.push(ReactiveChange::Transform(
            node_id,
            Transform2D::translate(5.0, 10.0),
        ));
        buffer.changes.push(ReactiveChange::Opacity(node_id, 0.3));
        world.insert_resource(buffer);

        // Run apply
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_reactive_changes_system);
        schedule.run(&mut world);

        let scene = world.resource::<Scene>();
        let node = scene.get_node(node_id).unwrap();

        // Color
        match &node.content {
            NodeContent::Rect { color } => {
                assert!((color.b() - 1.0).abs() < 0.001, "Should be BLUE");
            }
            _ => panic!("Expected Rect"),
        }

        // Transform
        let point = glam::Vec2::ZERO;
        let result = node.transform.transform_point(point);
        assert!((result.x - 5.0).abs() < 0.001);
        assert!((result.y - 10.0).abs() < 0.001);

        // Opacity
        assert!((node.opacity - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_gather_apply_integration_matches_monolithic() {
        let runtime = Runtime::new();

        let color_signal = Signal::new(runtime.clone(), Color::RED);
        let (color_read, color_write) = color_signal.split();

        let transform_signal = Signal::new(runtime.clone(), Transform2D::identity());
        let (transform_read, transform_write) = transform_signal.split();

        let opacity_signal = Signal::new(runtime.clone(), 1.0_f32);
        let (opacity_read, opacity_write) = opacity_signal.split();

        // Setup world with gather-apply
        let mut world = World::new();
        let mut scene = Scene::new();
        let root = scene.root();
        let node_id = scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Rect {
                    color: Color::GREEN,
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
        world.insert_resource(ReactiveChangeBuffer::default());

        world.spawn((
            SceneNodeRef(node_id),
            ReactiveColor::new(color_read),
            ReactiveTransform::new(transform_read),
            ReactiveOpacity::new(opacity_read),
        ));

        // Set new values
        color_write.set(Color::BLUE);
        transform_write.set(Transform2D::translate(50.0, 75.0));
        opacity_write.set(0.5);

        // Run gather then apply
        let mut schedule = Schedule::default();
        schedule.add_systems((
            gather_reactive_changes_system,
            apply_reactive_changes_system.after(gather_reactive_changes_system),
        ));
        schedule.run(&mut world);

        // Verify results match what monolithic system would produce
        let scene = world.resource::<Scene>();
        let node = scene.get_node(node_id).unwrap();

        match &node.content {
            NodeContent::Rect { color } => {
                assert!((color.b() - 1.0).abs() < 0.001, "Color should be BLUE");
            }
            _ => panic!("Expected Rect"),
        }

        let point = glam::Vec2::ZERO;
        let result = node.transform.transform_point(point);
        assert!((result.x - 50.0).abs() < 0.001);
        assert!((result.y - 75.0).abs() < 0.001);

        assert!((node.opacity - 0.5).abs() < 0.001);
    }
}
