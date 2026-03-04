#[cfg(feature = "nova")]
mod implementation {
    use bevy_ecs::prelude::*;
    use flux_state::{GraphSnapshot, NodeInfo, NodeType, Runtime};
    use render_engine::{
        Affine2, Color, NodeContent, NodeId, Paint, Scene, SceneNode, StrokeStyle, Transform2D,
        Vec2, VisualStyle,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// Wrapper for the Flux Runtime to be used as a Resource.
    #[derive(Resource, Clone)]
    pub struct FluxRuntime(pub Arc<Runtime>);

    /// Configuration for the Flux Radar visualization.
    #[derive(Resource)]
    pub struct FluxRadarConfig {
        pub enabled: bool,
        pub update_interval: Duration,
        pub node_radius: f32,
        pub layer_spacing: f32,
    }

    impl Default for FluxRadarConfig {
        fn default() -> Self {
            Self {
                enabled: true, // Auto-enable if nova feature is active
                update_interval: Duration::from_millis(200),
                node_radius: 15.0,
                layer_spacing: 150.0,
            }
        }
    }

    /// Runtime state for the Flux Radar.
    #[derive(Resource)]
    pub struct FluxRadarState {
        pub root_node: Option<NodeId>,
        pub last_update: Instant,
        pub snapshot: Option<GraphSnapshot>,
    }

    impl Default for FluxRadarState {
        fn default() -> Self {
            Self {
                root_node: None,
                last_update: Instant::now(),
                snapshot: None,
            }
        }
    }

    /// System to update the Flux Radar visualization.
    pub fn update_flux_radar(
        mut scene: ResMut<Scene>,
        mut state: ResMut<FluxRadarState>,
        config: Res<FluxRadarConfig>,
        runtime: Option<Res<FluxRuntime>>,
    ) {
        if !config.enabled {
            if let Some(root) = state.root_node {
                scene.remove_node(root);
                state.root_node = None;
            }
            return;
        }

        // Ensure we have a runtime
        let runtime = match runtime {
            Some(r) => r,
            None => return,
        };

        // Check update interval
        let now = Instant::now();
        if now.duration_since(state.last_update) < config.update_interval {
            return;
        }
        state.last_update = now;

        // Take snapshot (access inner Arc)
        let snapshot = runtime.0.inspect_graph();
        state.snapshot = Some(snapshot.clone());

        // Ensure root node exists
        let root_id = if let Some(id) = state.root_node {
            // Check if still valid (scene might have been cleared)
            if scene.get_node(id).is_none() {
                let root = scene.root();
                let id = scene.add_node(root, SceneNode::new(NodeContent::Empty));
                state.root_node = Some(id);
                id
            } else {
                // Clear children (naive redraw)
                // We collect children first to avoid borrow issues
                let children = scene.get_node(id).unwrap().children.clone();
                for child in children {
                    scene.remove_node(child);
                }
                id
            }
        } else {
            let root = scene.root();
            let id = scene.add_node(root, SceneNode::new(NodeContent::Empty));
            state.root_node = Some(id);
            id
        };

        // --- Layout ---
        let mut signals = Vec::new();
        let mut computeds = Vec::new();
        let mut effects = Vec::new();

        for node in &snapshot.nodes {
            match node.node_type {
                NodeType::Signal => signals.push(node),
                NodeType::Computed => computeds.push(node),
                NodeType::Effect => effects.push(node),
            }
        }

        // Sort by ID for stability
        signals.sort_by_key(|n| n.id);
        computeds.sort_by_key(|n| n.id);
        effects.sort_by_key(|n| n.id);

        let width = 800.0; // Assume a canvas width
        #[allow(unused_variables)]
        let center_x = width / 2.0;

        let mut node_positions = HashMap::new();

        // Helper to layout a row
        let layout_row =
            |nodes: &[&NodeInfo], y: f32, positions: &mut HashMap<flux_state::NodeId, Vec2>| {
                let count = nodes.len();
                if count == 0 {
                    return;
                }
                let spacing = width / (count as f32 + 1.0);
                for (i, node) in nodes.iter().enumerate() {
                    let x = spacing * (i as f32 + 1.0);
                    positions.insert(node.id, Vec2::new(x, y));
                }
            };

        let start_y = 50.0;
        layout_row(&signals, start_y, &mut node_positions);
        layout_row(
            &computeds,
            start_y + config.layer_spacing,
            &mut node_positions,
        );
        layout_row(
            &effects,
            start_y + config.layer_spacing * 2.0,
            &mut node_positions,
        );

        // --- Render Edges ---
        // (Source, Subscriber)
        let edge_style =
            Box::new(VisualStyle::new().solid_fill(Color::rgba(1.0, 1.0, 1.0, 0.2).as_vec4()));

        for (subscriber, source) in &snapshot.dependencies {
            if let (Some(&start), Some(&end)) =
                (node_positions.get(source), node_positions.get(subscriber))
            {
                let diff: Vec2 = end - start;
                let length = diff.length();
                if length > 0.0 {
                    let angle = diff.y.atan2(diff.x);

                    let mut edge_node = SceneNode::new(NodeContent::Styled {
                        style: edge_style.clone(),
                    });

                    // Transform: rotate and position at midpoint
                    // Or position at start and rotate
                    edge_node.bounds = plat_core::Rect::new(0.0, -1.0, length, 2.0); // 2px thick line

                    let transform = Transform2D::translate(start.x, start.y)
                        .compose(&Transform2D(Affine2::from_angle(angle)));

                    edge_node.transform = transform;

                    scene.add_node(root_id, edge_node);
                }
            }
        }

        // --- Render Nodes ---
        for node in &snapshot.nodes {
            if let Some(&pos) = node_positions.get(&node.id) {
                let is_stale = snapshot.stale_nodes.contains(&node.id);

                let color = match node.node_type {
                    NodeType::Signal => Color::GREEN,
                    NodeType::Computed => Color::rgba(0.0, 1.0, 1.0, 1.0), // Cyan
                    NodeType::Effect => Color::rgba(1.0, 1.0, 0.0, 1.0),   // Yellow
                };

                let mut style = VisualStyle::new();

                // Fill
                if is_stale {
                    // Flash Red
                    style = style.solid_fill(Color::RED.as_vec4());
                } else {
                    style = style.solid_fill(color.as_vec4());
                }

                // Stroke
                style = style.stroke(StrokeStyle::solid(
                    Paint::Solid(Color::WHITE.as_vec4()),
                    2.0,
                    style_engine::StrokeAlign::Inside,
                ));

                // Radius (Circle)
                style = style.corner_radius(config.node_radius);

                let mut scene_node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(style),
                });

                let r = config.node_radius;
                scene_node.bounds = plat_core::Rect::new(-r, -r, r * 2.0, r * 2.0);
                scene_node.transform = Transform2D::translate(pos.x, pos.y);

                // Add tooltip/label if needed (omitted for simplicity, maybe add text node later)

                scene.add_node(root_id, scene_node);
            }
        }
    }

    pub fn register_flux_radar(app: &mut crate::App) {
        if app.world().get_resource::<FluxRadarConfig>().is_none() {
            app.world_mut().insert_resource(FluxRadarConfig::default());
        }
        if app.world().get_resource::<FluxRadarState>().is_none() {
            app.world_mut().insert_resource(FluxRadarState::default());
        }

        // Inject FluxRuntime if missing, using the App's runtime
        if app.world().get_resource::<FluxRuntime>().is_none() {
            let runtime = app.runtime().clone();
            app.world_mut().insert_resource(FluxRuntime(runtime));
        }

        app.add_update_system(update_flux_radar);
    }
}

#[cfg(not(feature = "nova"))]
mod implementation {
    pub fn register_flux_radar(_: &mut crate::App) {
        // No-op when nova feature is disabled
        log::info!("Flux Radar requires 'nova' feature to be enabled.");
    }
}

pub use implementation::register_flux_radar;
#[cfg(feature = "nova")]
pub use implementation::{FluxRadarConfig, FluxRadarState, FluxRuntime};

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use flux_state::Signal;

    #[test]
    fn test_flux_radar_registration() {
        let mut app = crate::App::new_headless().unwrap();
        register_flux_radar(&mut app);

        assert!(app.world().get_resource::<FluxRadarConfig>().is_some());
        assert!(app.world().get_resource::<FluxRadarState>().is_some());
    }

    #[test]
    fn test_flux_radar_lifecycle() {
        let mut app = crate::App::new_headless().unwrap();

        // Register feature
        register_flux_radar(&mut app);

        assert!(app.world().get_resource::<FluxRadarConfig>().is_some());
        assert!(app.world().get_resource::<FluxRadarState>().is_some());
        assert!(app.world().get_resource::<FluxRuntime>().is_some());

        // Create signal
        // We use the runtime already in the app
        let runtime = app.runtime().clone();
        let _s1 = Signal::new(runtime, 10);

        // Run update (advances time if mocked?)
        // In real app, update calls systems.
        // We need to simulate time passing for update_interval?
        // update_interval is 200ms.
        // But the first update should run immediately because last_update is initialized to now().
        // Wait, if last_update is now(), and we call update() immediately, duration is 0 < 200ms.
        // So it returns early!

        // We need to force update or wait.
        // Or we can modify last_update in state.

        {
            let mut state = app.world_mut().resource_mut::<FluxRadarState>();
            // Set last_update to past
            state.last_update = std::time::Instant::now() - std::time::Duration::from_secs(1);
        }

        app.update();

        // Verify state
        let state = app.world().resource::<FluxRadarState>();
        assert!(state.root_node.is_some());
        assert!(state.snapshot.is_some());
        // Should have 1 signal + root
        assert_eq!(state.snapshot.as_ref().unwrap().nodes.len(), 1);
    }
}
