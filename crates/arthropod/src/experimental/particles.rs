use bevy_ecs::prelude::*;
use plat_core::Rect;
use render_engine::{Color, NodeContent, NodeId, Scene, SceneNode, Vec2};
use std::time::Instant;

/// Component for a single particle
#[derive(Component, Debug, Clone)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub node_id: Option<NodeId>,
}

/// Component for emitting particles
#[derive(Component, Debug, Clone)]
pub struct ParticleEmitter {
    pub rate: f32, // Particles per second
    pub accumulator: f32,
    pub position: Vec2,
    pub spread: Vec2, // Velocity spread
    pub color: Color,
    pub lifetime_range: (f32, f32),
    pub size: f32,
    pub active: bool,
    pub rng_state: u64, // Persisted RNG state
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            rate: 10.0,
            accumulator: 0.0,
            position: Vec2::ZERO,
            spread: Vec2::new(50.0, 50.0),
            color: Color::RED,
            lifetime_range: (1.0, 2.0),
            size: 5.0,
            active: true,
            rng_state: 12345,
        }
    }
}

/// Resource for global time delta
#[derive(Resource, Default)]
pub struct ParticleTime {
    pub dt: f32,
}

/// Resource for global state tracking (time, etc)
#[derive(Resource)]
pub struct ParticleGlobalState {
    pub last_update: Instant,
}

impl Default for ParticleGlobalState {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
        }
    }
}

// Simple LCG RNG
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as f32 / 4294967296.0
    }

    fn next_signed(&mut self) -> f32 {
        self.next_f32() * 2.0 - 1.0
    }
}

/// System to update global particle time
///
/// Updates `ParticleTime` resource based on real elapsed time.
///
/// # Example
///
/// ```
/// # use bevy_ecs::prelude::*;
/// # use arthropod::experimental::particles::{update_particle_time, ParticleTime, ParticleGlobalState};
/// # let mut world = World::new();
/// # world.insert_resource(ParticleTime::default());
/// # world.insert_resource(ParticleGlobalState::default());
/// # let mut schedule = Schedule::default();
/// # schedule.add_systems(update_particle_time);
/// ```
pub fn update_particle_time(mut time: ResMut<ParticleTime>, mut state: ResMut<ParticleGlobalState>) {
    let now = Instant::now();
    // In wasm or some envs duration might panic if time goes back, but here we assume monotonic
    if now >= state.last_update {
        time.dt = now.duration_since(state.last_update).as_secs_f32();
    } else {
        time.dt = 0.016; // Fallback
    }

    state.last_update = now;

    // Cap dt to avoid explosion
    if time.dt > 0.1 {
        time.dt = 0.1;
    }
}

/// System to emit new particles
///
/// Spawns particles based on `ParticleEmitter` configuration.
///
/// # Example
///
/// ```
/// # use bevy_ecs::prelude::*;
/// # use arthropod::experimental::particles::{emit_particles, ParticleTime};
/// # use render_engine::Scene;
/// # let mut world = World::new();
/// # world.insert_resource(ParticleTime::default());
/// # world.insert_resource(Scene::new());
/// # let mut schedule = Schedule::default();
/// # schedule.add_systems(emit_particles);
/// ```
pub fn emit_particles(
    mut commands: Commands,
    mut emitters: Query<&mut ParticleEmitter>,
    time: Res<ParticleTime>,
    mut scene: ResMut<Scene>,
) {
    for mut emitter in &mut emitters {
        if !emitter.active {
            continue;
        }

        emitter.accumulator += time.dt * emitter.rate;

        // Use emitter's RNG state
        let mut rng = Rng::new(emitter.rng_state);

        while emitter.accumulator >= 1.0 {
            emitter.accumulator -= 1.0;

            // Create SceneNode
            let mut node = SceneNode::new(NodeContent::Rect {
                color: emitter.color,
            });
            node.bounds = Rect::new(
                emitter.position.x,
                emitter.position.y,
                emitter.size,
                emitter.size,
            );

            let root = scene.root();
            let node_id = scene.add_node(root, node);

            // Spawn entity
            let vx = rng.next_signed() * emitter.spread.x;
            let vy = rng.next_signed() * emitter.spread.y;
            let lifetime = emitter.lifetime_range.0
                + (emitter.lifetime_range.1 - emitter.lifetime_range.0) * rng.next_f32();

            commands.spawn((
                Particle {
                    velocity: Vec2::new(vx, vy),
                    lifetime,
                    max_lifetime: lifetime,
                    node_id: Some(node_id),
                },
                crate::prelude::Renderable,
                crate::prelude::SceneNodeRef(node_id),
            ));
        }

        // Save RNG state
        emitter.rng_state = rng.state;
    }
}

/// System to update existing particles
///
/// Updates particle positions and lifetimes, and handles despawning dead particles.
///
/// # Example
///
/// ```
/// # use bevy_ecs::prelude::*;
/// # use arthropod::experimental::particles::{update_particles, ParticleTime};
/// # use render_engine::Scene;
/// # let mut world = World::new();
/// # world.insert_resource(ParticleTime::default());
/// # world.insert_resource(Scene::new());
/// # let mut schedule = Schedule::default();
/// # schedule.add_systems(update_particles);
/// ```
pub fn update_particles(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Particle)>,
    time: Res<ParticleTime>,
    mut scene: ResMut<Scene>,
) {
    for (entity, mut particle) in &mut particles {
        particle.lifetime -= time.dt;

        if particle.lifetime <= 0.0 {
            if let Some(node_id) = particle.node_id {
                scene.remove_node(node_id);
            }
            commands.entity(entity).despawn();
        } else if let Some(node_id) = particle.node_id {
            let mut marked = false;
            #[allow(clippy::collapsible_if)]
            if let Some(node) = scene.get_mut(node_id) {
                node.bounds.x += particle.velocity.x * time.dt;
                node.bounds.y += particle.velocity.y * time.dt;
                node.opacity = particle.lifetime / particle.max_lifetime;
                marked = true;
            }
            if marked {
                scene.mark_dirty(node_id);
            }
        }
    }
}

/// Register particle systems with the application
///
/// Initializes resources and adds systems to the app update loop.
///
/// # Example
///
/// ```
/// # use arthropod::prelude::*;
/// # use arthropod::experimental::particles::register_particles;
/// # let mut app = AppBuilder::new().build_headless().unwrap();
/// register_particles(&mut app);
/// ```
pub fn register_particles(app: &mut crate::App) {
    app.world_mut().insert_resource(ParticleTime::default());
    app.world_mut().insert_resource(ParticleGlobalState::default());
    app.add_update_system((update_particle_time, emit_particles, update_particles).chain());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particles_spawn_and_die() {
        let mut world = World::new();
        world.insert_resource(Scene::new());
        world.insert_resource(ParticleTime { dt: 0.1 });
        world.insert_resource(ParticleGlobalState::default());

        let mut update_schedule = Schedule::default();
        // Skip update_particle_time in test to control dt manually
        update_schedule.add_systems((emit_particles, update_particles).chain());

        // Spawn emitter
        world.spawn(ParticleEmitter {
            rate: 20.0, // 2 particles per 0.1s step
            active: true,
            ..Default::default()
        });

        // Run one frame
        update_schedule.run(&mut world);

        // Check particles spawned
        let particle_count = world.query::<&Particle>().iter(&world).count();
        assert!(particle_count >= 1, "Should have spawned particles");

        // Check scene nodes
        let scene = world.resource::<Scene>();
        // Root + particles
        assert!(
            scene.nodes().count() > 1,
            "Scene should have particle nodes"
        );

        // Disable emitter so we don't spawn more
        let mut emitter = world.query::<&mut ParticleEmitter>().single_mut(&mut world);
        emitter.active = false;

        // Fast forward to kill particles
        world.insert_resource(ParticleTime { dt: 10.0 });
        update_schedule.run(&mut world);

        let particle_count_after = world.query::<&Particle>().iter(&world).count();
        assert_eq!(particle_count_after, 0, "All particles should have died");

        // Check scene cleanup
        let scene = world.resource::<Scene>();
        assert_eq!(scene.nodes().count(), 1, "Scene should only have root node");
    }
}
