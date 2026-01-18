use bevy_ecs::prelude::*;
use bevy_ecs::world::EntityWorldMut;
use render_engine::{backend::RectInstance, NodeId, Scene};

use crate::components::SceneNodeRef;
use crate::systems::{
    collect_renderables_system, update_reactive_colors_system, update_reactive_opacity_system,
    update_reactive_transforms_system, RenderCommands, SceneReadResource, SceneResource,
};

/// Enterprise GUI framework context - wraps ECS World
///
/// FrameworkContext is the main API for integrating ECS into Arthropod applications.
/// It manages the ECS World, system schedules, and provides convenient methods for
/// spawning entities linked to scene nodes and running update/render systems.
///
/// # Architecture
///
/// The context uses a hybrid approach:
/// - Scene tree remains custom (HashMap-based for O(1) access and cache-friendly traversal)
/// - ECS entities reference scene nodes via SceneNodeRef components
/// - Systems query ECS components and update scene nodes via NodeId
///
/// # Example
///
/// ```no_run
/// use arthropod_ecs::FrameworkContext;
/// use render_engine::Scene;
///
/// let mut scene = Scene::new();
/// let mut context = FrameworkContext::new();
///
/// // Spawn entity linked to scene node
/// let node_id = scene.root();
/// context.spawn(node_id)
///     .insert(arthropod_ecs::Renderable);
///
/// // Update reactive systems
/// context.update(&mut scene);
///
/// // Render and get GPU instances
/// let instances = context.render(&scene);
/// ```
pub struct FrameworkContext {
    world: World,
    render_schedule: Schedule,
    update_schedule: Schedule,
}

impl FrameworkContext {
    /// Create a new FrameworkContext with initialized ECS World and schedules
    pub fn new() -> Self {
        let mut world = World::new();

        // Initialize resources
        world.insert_resource(RenderCommands::default());

        Self {
            world,
            render_schedule: Self::build_render_schedule(),
            update_schedule: Self::build_update_schedule(),
        }
    }

    /// Spawn a visual entity linked to a scene node
    ///
    /// Creates a new ECS entity with a SceneNodeRef component pointing to the
    /// given NodeId. Additional components can be added via the returned EntityWorldMut.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arthropod_ecs::FrameworkContext;
    /// # use render_engine::NodeId;
    /// # use flux_state::{Runtime, Signal};
    /// # use render_engine::Color;
    /// # let mut context = FrameworkContext::new();
    /// # let node_id = NodeId(0);
    /// # let runtime = Runtime::new();
    /// # let color_signal = Signal::new(runtime, Color::RED);
    /// # let (read_signal, _) = color_signal.split();
    /// context.spawn(node_id)
    ///     .insert(arthropod_ecs::Renderable)
    ///     .insert(arthropod_ecs::ReactiveColor::new(read_signal));
    /// ```
    pub fn spawn(&mut self, node_id: NodeId) -> EntityWorldMut<'_> {
        self.world.spawn(SceneNodeRef(node_id))
    }

    /// Run all update systems (reactive, animation, etc.)
    ///
    /// This runs the update schedule which includes:
    /// - Reactive color updates (polling signals)
    /// - Reactive transform updates
    /// - Reactive opacity updates
    ///
    /// The scene is temporarily inserted as a resource for systems to access.
    pub fn update(&mut self, scene: &mut Scene) {
        // SAFETY: The Scene reference is valid for the duration of this method.
        // We insert it as a resource, run systems synchronously, then immediately remove it.
        // No references escape this scope.
        unsafe {
            self.world.insert_resource(SceneResource::new(scene));
        }
        self.update_schedule.run(&mut self.world);
        self.world.remove_resource::<SceneResource>();
    }

    /// Run all render systems and return render commands
    ///
    /// This runs the render schedule which collects all visible Renderable entities
    /// and generates RectInstance data for the GPU backend.
    ///
    /// Returns a Vec of RectInstances that can be passed to WgpuBackend::render_instances.
    pub fn render(&mut self, scene: &Scene) -> Vec<RectInstance> {
        // SAFETY: The Scene reference is valid for the duration of this method.
        // We insert it as a resource, run systems synchronously, then immediately remove it.
        // No references escape this scope.
        unsafe {
            self.world.insert_resource(SceneReadResource::new(scene));
        }
        self.render_schedule.run(&mut self.world);
        self.world.remove_resource::<SceneReadResource>();

        // Extract render commands
        let commands = self.world.resource::<RenderCommands>();
        commands.0.clone()
    }

    /// Build the render schedule with rendering systems
    fn build_render_schedule() -> Schedule {
        let mut schedule = Schedule::default();
        schedule.add_systems(collect_renderables_system);
        schedule
    }

    /// Build the update schedule with reactive and animation systems
    fn build_update_schedule() -> Schedule {
        let mut schedule = Schedule::default();
        schedule.add_systems((
            update_reactive_colors_system,
            update_reactive_transforms_system,
            update_reactive_opacity_system,
        ));
        schedule
    }
}

impl Default for FrameworkContext {
    fn default() -> Self {
        Self::new()
    }
}
