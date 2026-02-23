use bevy_ecs::prelude::*;
use flux_state::ReadSignal;
use render_engine::{Color, NodeContent, Scene, Vec2, VisualStyle};
use std::collections::VecDeque;
use style_engine::path::{VectorPath, WindingRule};
use style_engine::{Paint, StrokeStyle};

/// Component to visualize a signal as a graph.
#[derive(Component)]
pub struct SignalGraph {
    pub signal: ReadSignal<f32>,
    pub min_value: f32,
    pub max_value: f32,
    pub color: Color,
    pub stroke_width: f32,
    pub history_length: usize,
}

impl SignalGraph {
    pub fn new(signal: ReadSignal<f32>, min: f32, max: f32) -> Self {
        Self {
            signal,
            min_value: min,
            max_value: max,
            color: Color::GREEN,
            stroke_width: 2.0,
            history_length: 100,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_history(mut self, length: usize) -> Self {
        self.history_length = length;
        self
    }
}

/// Internal component to store signal history.
#[derive(Component, Default)]
pub struct SignalHistory {
    values: VecDeque<f32>,
}

/// System to update signal graphs.
pub fn update_signal_graphs(
    mut commands: Commands,
    mut query: Query<(Entity, &SignalGraph, Option<&mut SignalHistory>)>,
    mut scene: ResMut<Scene>,
    node_query: Query<&crate::prelude::SceneNodeRef>,
) {
    for (entity, graph, mut history) in &mut query {
        let value = graph.signal.get();

        // Update history
        if let Some(history) = history.as_mut() {
            history.values.push_back(value);
            if history.values.len() > graph.history_length {
                history.values.pop_front();
            }
        } else {
            let mut new_history = SignalHistory::default();
            new_history.values.push_back(value);
            commands.entity(entity).insert(new_history);
            // We can't use the history in this frame easily without re-querying, so skip
            continue;
        }

        // We need to re-borrow history immutable here, but we have mutable borrow from query.
        // However, we are inside loop. Let's just use the history we have.
        let history = history.unwrap();

        // Get the SceneNode ID from the entity
        #[allow(clippy::collapsible_if)]
        if let Ok(node_ref) = node_query.get(entity) {
            if let Some(node) = scene.get_node_mut(node_ref.0) {
                // Generate path
                let bounds = node.bounds;
                if bounds.width <= 0.0 || bounds.height <= 0.0 {
                    continue;
                }

                if history.values.len() < 2 {
                    continue;
                }

                let mut path = VectorPath::new();
                path.winding_rule = WindingRule::NonZero;

                let step_x = bounds.width / (graph.history_length as f32 - 1.0);
                let range = graph.max_value - graph.min_value;
                let scale_y = if range != 0.0 {
                    bounds.height / range
                } else {
                    1.0
                };

                // Move to first point
                let first_val = history.values[0].clamp(graph.min_value, graph.max_value);
                // Y is typically down in UI, but graphs usually go up.
                // Let's assume 0,0 is top-left.
                // Value min -> bounds.y + bounds.height (bottom)
                // Value max -> bounds.y (top)
                let y = bounds.height - (first_val - graph.min_value) * scale_y;
                path.move_to(Vec2::new(0.0, y));

                for (i, val) in history.values.iter().enumerate().skip(1) {
                    let val = val.clamp(graph.min_value, graph.max_value);
                    let x = i as f32 * step_x;
                    let y = bounds.height - (val - graph.min_value) * scale_y;
                    path.line_to(Vec2::new(x, y));
                }

                // Update node content
                let style = VisualStyle::new()
                    .stroke(StrokeStyle::solid(
                        Paint::solid(graph.color.as_vec4()),
                        graph.stroke_width,
                        style_engine::StrokeAlign::Center,
                    ))
                    .path(path);

                node.content = NodeContent::Styled {
                    style: Box::new(style),
                };

                // Mark dirty
                scene.mark_dirty(node_ref.0);
            }
        }
    }
}

pub fn register_signal_graph(app: &mut crate::App) {
    app.add_update_system(update_signal_graphs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_state::{Runtime, Signal};
    use render_engine::{Scene, SceneNode};

    #[test]
    fn test_signal_graph_updates() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_signal_graphs);

        let runtime = Runtime::new();
        let (read, write) = Signal::new(runtime, 0.0).split();

        // Create scene
        let mut scene = Scene::new();
        let root = scene.root();

        // Add a node for our graph
        let mut node = SceneNode::new(NodeContent::Empty);
        node.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
        let node_id = scene.add_node(root, node);

        world.insert_resource(scene);

        // Spawn entity
        world.spawn((
            SignalGraph::new(read, 0.0, 100.0).with_history(10),
            crate::prelude::SceneNodeRef(node_id),
        ));

        // 1. First run: Initializes history
        schedule.run(&mut world);

        // Check history exists
        assert!(
            world
                .query::<&SignalHistory>()
                .iter(&world)
                .next()
                .is_some()
        );

        // 2. Second run: Should update path (but only 1 point, so no line yet)
        schedule.run(&mut world);

        // 3. Update signal and run
        write.set(50.0);
        schedule.run(&mut world);

        // Now history has 3 points (0.0, 0.0, 50.0)?
        // Wait, run 1: pushes 0.0. History len 1.
        // Run 2: pushes 0.0. History len 2.
        // Run 3: pushes 50.0. History len 3.

        let scene = world.resource::<Scene>();
        let node = scene.get_node(node_id).unwrap();

        if let NodeContent::Styled { style } = &node.content {
            assert!(style.fill_geometry.is_some());
            let path = &style.fill_geometry.as_ref().unwrap()[0];
            // Should have MoveTo and LineTo commands
            assert!(path.commands.len() >= 2);
        } else {
            panic!("Node content should be Styled");
        }
    }
}
