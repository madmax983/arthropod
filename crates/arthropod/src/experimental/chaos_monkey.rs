//! Chaos Monkey Simulator
//!
//! An experimental testing feature that automatically finds and clicks
//! visible `Clickable` nodes in the UI to simulate random user interactions.

#[cfg(feature = "nova")]
use arthropod_ecs::components::{Clickable, SceneNodeRef};
#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use render_engine::Scene;
#[cfg(feature = "nova")]
use std::time::{Duration, Instant};

/// Configuration for the Chaos Monkey simulator.
#[cfg(feature = "nova")]
#[derive(Resource)]
pub struct ChaosMonkeyConfig {
    /// Whether the simulator is active.
    pub enabled: bool,
    /// The time interval between simulated clicks.
    pub click_interval: Duration,
}

#[cfg(feature = "nova")]
impl Default for ChaosMonkeyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            click_interval: Duration::from_millis(500),
        }
    }
}

/// State for the Chaos Monkey simulator.
#[cfg(feature = "nova")]
#[derive(Resource)]
pub struct ChaosMonkeyState {
    /// The time the last click was simulated.
    pub last_click: Instant,
    /// A simple internal state for the LCG pseudo-random number generator.
    pub rng_state: u64,
}

#[cfg(feature = "nova")]
impl Default for ChaosMonkeyState {
    fn default() -> Self {
        Self {
            last_click: Instant::now(),
            rng_state: 0x1337_CAFE_BABE_BEEF,
        }
    }
}

#[cfg(feature = "nova")]
impl ChaosMonkeyState {
    /// A simple Linear Congruential Generator to avoid pulling in `rand`.
    fn next_random(&mut self) -> usize {
        // LCG constants from Numerical Recipes
        self.rng_state = self
            .rng_state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        self.rng_state as usize
    }
}

/// System that maintains the chaos monkey simulation.
#[cfg(feature = "nova")]
pub fn update_chaos_monkey(
    config: Res<ChaosMonkeyConfig>,
    mut state: ResMut<ChaosMonkeyState>,
    query: Query<(&Clickable, &SceneNodeRef)>,
    scene: Res<Scene>,
) {
    if !config.enabled {
        return;
    }

    let now = Instant::now();
    if now.duration_since(state.last_click) < config.click_interval {
        return;
    }

    // Find all valid clickable targets
    let mut clickables = Vec::new();

    for (clickable, node_ref) in query.iter() {
        if let Some(node) = scene.get_node(node_ref.0) {
            // Only click nodes that are actually visible
            if node.visible && node.opacity > 0.0 {
                clickables.push(clickable.clone());
            }
        }
    }

    if clickables.is_empty() {
        // Nothing to click right now
        return;
    }

    // Pick a random target
    let target_idx = state.next_random() % clickables.len();
    let target = &clickables[target_idx];

    // Simulate the click!
    (target.callback)();

    // Reset the interval
    state.last_click = now;
}

/// Registers the chaos monkey feature.
#[cfg(feature = "nova")]
pub fn register_chaos_monkey(app: &mut crate::App) {
    app.world_mut()
        .insert_resource(ChaosMonkeyConfig::default());
    app.world_mut().insert_resource(ChaosMonkeyState::default());
    app.add_update_system(update_chaos_monkey);
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use render_engine::{NodeContent, Rect, SceneNode, Transform2D};
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_chaos_monkey_clicks_visible_node() {
        let mut scene = Scene::new();
        let root = scene.root();

        // Add a node
        let mut node = SceneNode::new(NodeContent::Empty);
        node.visible = true;
        node.opacity = 1.0;
        node.transform = Transform2D::IDENTITY;
        node.bounds = Rect::default();

        let id = scene.add_node(root, node);

        let mut world = World::new();
        world.insert_resource(scene);

        // Use a short interval so it triggers immediately after default state (Instant::now())
        // But wait, the system uses duration_since, we can just tweak the state.
        world.insert_resource(ChaosMonkeyConfig {
            enabled: true,
            click_interval: Duration::from_millis(100),
        });

        let mut state = ChaosMonkeyState::default();
        // Force the last click to be long ago
        state.last_click = Instant::now() - Duration::from_secs(1);
        world.insert_resource(state);

        let clicked = Arc::new(Mutex::new(false));
        let clicked_clone = clicked.clone();

        world
            .spawn(Clickable {
                callback: Arc::new(move || {
                    *clicked_clone.lock().unwrap() = true;
                }),
            })
            .insert(SceneNodeRef(id));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_chaos_monkey);

        // Run tick
        schedule.run(&mut world);

        // Assert it was clicked
        assert!(
            *clicked.lock().unwrap(),
            "Chaos Monkey should have clicked the node"
        );
    }

    #[test]
    fn test_chaos_monkey_ignores_invisible_node() {
        let mut scene = Scene::new();
        let root = scene.root();

        let mut node = SceneNode::new(NodeContent::Empty);
        node.visible = false; // INVISIBLE
        node.opacity = 1.0;
        node.transform = Transform2D::IDENTITY;
        node.bounds = Rect::default();

        let id = scene.add_node(root, node);

        let mut world = World::new();
        world.insert_resource(scene);

        world.insert_resource(ChaosMonkeyConfig {
            enabled: true,
            click_interval: Duration::from_millis(100),
        });

        let mut state = ChaosMonkeyState::default();
        state.last_click = Instant::now() - Duration::from_secs(1);
        world.insert_resource(state);

        let clicked = Arc::new(Mutex::new(false));
        let clicked_clone = clicked.clone();

        world
            .spawn(Clickable {
                callback: Arc::new(move || {
                    *clicked_clone.lock().unwrap() = true;
                }),
            })
            .insert(SceneNodeRef(id));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_chaos_monkey);

        // Run tick
        schedule.run(&mut world);

        // Assert it was NOT clicked
        assert!(
            !*clicked.lock().unwrap(),
            "Chaos Monkey should NOT click invisible nodes"
        );
    }
}
