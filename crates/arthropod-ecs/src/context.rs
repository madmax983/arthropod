use a11y_engine::{A11yTree, ArthropodActionHandler, FocusManager};
use bevy_ecs::prelude::*;
use bevy_ecs::world::EntityWorldMut;
use render_engine::{backend::RectInstance, NodeId, Scene};

use crate::components::SceneNodeRef;
use crate::systems::{
    collect_renderables_system, sync_accessible_nodes_system, update_reactive_colors_system,
    update_reactive_opacity_system, update_reactive_transforms_system, RenderCommands,
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
/// let mut context = FrameworkContext::new();
///
/// // Access Scene from World to get root node
/// let node_id = {
///     let scene = context.world().resource::<Scene>();
///     scene.root()
/// };
///
/// // Spawn entity linked to scene node
/// context.spawn(node_id)
///     .insert(arthropod_ecs::Renderable);
///
/// // Update reactive systems (Scene accessed from World automatically)
/// context.update();
///
/// // Render and get GPU instances (Scene accessed from World automatically)
/// let instances = context.render();
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
        world.insert_resource(Scene::new()); // Scene lives in the World now!
        world.insert_resource(A11yTree::new()); // A11yTree for accessibility
        world.insert_resource(FocusManager::new()); // Focus tracking
        world.insert_resource(ArthropodActionHandler::new()); // Action handler
        world.insert_resource(RenderCommands::default());

        Self {
            world,
            render_schedule: Self::build_render_schedule(),
            update_schedule: Self::build_update_schedule(),
        }
    }

    /// Access the ECS World (immutable)
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Access the ECS World (mutable)
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
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
    /// Scene is accessed from the World as a Resource (no unsafe code needed!)
    pub fn update(&mut self) {
        self.update_schedule.run(&mut self.world);
    }

    /// Run all render systems and return render commands
    ///
    /// This runs the render schedule which collects all visible Renderable entities
    /// and generates RectInstance data for the GPU backend.
    ///
    /// Returns a Vec of RectInstances that can be passed to WgpuBackend::render_instances.
    ///
    /// Scene is accessed from the World as a Resource (no unsafe code needed!)
    pub fn render(&mut self) -> Vec<RectInstance> {
        self.render_schedule.run(&mut self.world);

        // Extract render commands
        std::mem::take(&mut self.world.resource_mut::<RenderCommands>().0)
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
            sync_accessible_nodes_system, // Sync after reactive updates
        ));
        schedule
    }

    /// Add a system to the update schedule
    ///
    /// # Example
    ///
    /// ```
    /// # use arthropod_ecs::FrameworkContext;
    /// # use bevy_ecs::prelude::*;
    /// # let mut context = FrameworkContext::new();
    /// fn my_system() {}
    /// context.add_update_system(my_system);
    /// ```
    pub fn add_update_system<M>(&mut self, system: impl IntoSystemConfigs<M>) {
        self.update_schedule.add_systems(system);
    }
}

impl Default for FrameworkContext {
    fn default() -> Self {
        Self::new()
    }
}
