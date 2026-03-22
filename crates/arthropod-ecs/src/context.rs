use a11y_engine::{A11yTree, ArthropodActionHandler, FocusManager};
use bevy_ecs::prelude::*;
use bevy_ecs::world::EntityWorldMut;
use render_engine::{backend::PrimitiveInstance, NodeId, Scene};

use crate::components::{FrameSignalResource, MousePosition, SceneNodeRef};
use crate::systems::{
    apply_a11y_bounds_system, collect_renderables_system, gather_a11y_bounds_system, layout_system,
    A11yBoundsBuffer, ReactiveChangeBuffer, RenderCommands,
};

use crate::systems::RuntimeResource;
use anim_graph::ecs::{timeline_system, TimeResource};
use render_engine::Color;

use crate::systems::run_reactive_effects_system;

#[cfg(feature = "parallel-reactive")]
use crate::systems::{apply_reactive_changes_system, gather_reactive_changes_system};

#[cfg(not(feature = "parallel-reactive"))]
use crate::systems::{
    update_all_reactive_system, update_interaction_state_system, update_progress_bar_direct_system,
    update_widget_style_system,
};

/// System that increments the frame signal each frame
pub fn update_frame_signal_system(res: Res<FrameSignalResource>) {
    res.write_handle.update(|f| *f = f.wrapping_add(1));
}

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
    /// Unified schedule: reactive → layout → [a11y_sync || render_collect]
    frame_schedule: Schedule,
}

/// Initialize the bevy_tasks thread pool for multi-threaded system scheduling.
///
/// Must be called once before running schedules with the `multi_threaded` feature.
/// Subsequent calls are no-ops (the pool is a global singleton).
fn ensure_task_pool_initialized() {
    use bevy_tasks::ComputeTaskPool;
    ComputeTaskPool::get_or_init(|| {
        bevy_tasks::TaskPoolBuilder::new()
            .thread_name("arthropod-compute".to_string())
            .build()
    });
}

impl FrameworkContext {
    /// Create a new FrameworkContext with initialized ECS World and schedules
    pub fn new() -> Self {
        // Initialize the compute task pool for multi-threaded scheduling
        ensure_task_pool_initialized();

        let mut world = World::new();

        // Initialize resources
        world.insert_resource(Scene::new());
        world.insert_resource(A11yTree::new());
        world.insert_resource(FocusManager::new());
        world.insert_resource(ArthropodActionHandler::new());
        world.insert_resource(RenderCommands::default());
        world.insert_resource(ReactiveChangeBuffer::default());
        world.insert_resource(A11yBoundsBuffer::default());
        world.insert_resource(MousePosition::default());
        world.insert_resource(crate::components::IntegrationHeartbeat::default());
        world.insert_resource(TimeResource::default());

        // Create a shared runtime for reactive effects
        let runtime = flux_state::Runtime::new();
        world.insert_resource(RuntimeResource(runtime.clone()));

        // Create frame signal (incremented each frame for animation subscriptions)
        let frame_signal = flux_state::Signal::new(runtime, 0u64);
        let (read_handle, write_handle) = frame_signal.split();
        world.insert_resource(FrameSignalResource {
            write_handle,
            read_handle,
        });

        Self {
            world,
            frame_schedule: Self::build_frame_schedule(),
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

    /// Run the full frame schedule (reactive, layout, a11y, render collection).
    ///
    /// System execution order (enforced by explicit ordering constraints):
    /// 1. `gather_reactive_changes_system` — polls signals into buffer (NO Scene access)
    /// 2. `apply_reactive_changes_system` — writes buffered changes to Scene (`ResMut<Scene>`)
    /// 3. `layout_system` — computes flexbox layout (`ResMut<Scene>`)
    /// 4. `sync_accessible_nodes_system` + `collect_renderables_system` — run in parallel
    ///    (both use `Res<Scene>` with disjoint `ResMut` resources)
    pub fn update(&mut self) {
        // Run frame schedule
        self.frame_schedule.run(&mut self.world);
    }

    /// Run render collection and return render commands.
    ///
    /// Since Phase 3, render collection runs inside `update()` as part of the
    /// unified frame schedule. This method just extracts the collected instances.
    pub fn render(&mut self) -> Vec<PrimitiveInstance> {
        std::mem::take(&mut self.world.resource_mut::<RenderCommands>().0)
    }

    /// Build the unified frame schedule with explicit ordering.
    ///
    /// Pipeline (with `parallel-reactive` feature):
    /// ```text
    /// gather_reactive → apply_reactive → layout → [gather_a11y || render_collect] → apply_a11y
    /// ```
    ///
    /// Pipeline (without `parallel-reactive` feature):
    /// ```text
    /// update_all_reactive → layout → [gather_a11y || render_collect] → apply_a11y
    /// ```
    ///
    /// The gather-apply split (enabled by `parallel-reactive` feature) decouples signal
    /// polling from Scene mutation:
    /// - `gather_reactive_changes_system`: polls signals, NO Scene access
    /// - `apply_reactive_changes_system`: writes Scene, NO signal access
    ///
    /// Without the feature, uses simpler `update_all_reactive_system` which polls signals
    /// and writes Scene in a single pass (lower overhead for typical UI scales).
    ///
    /// After layout, systems can overlap:
    /// - `gather_a11y_bounds_system`: `Res<Scene>` + `ResMut<A11yBoundsBuffer>`
    /// - `collect_renderables_system`: `Res<Scene>` + `ResMut<RenderCommands>`
    ///
    /// Both use shared `Res<Scene>` with disjoint mutable resources.
    ///
    /// `apply_a11y_bounds_system` runs after gather to write to `ResMut<A11yTree>`.
    fn build_frame_schedule() -> Schedule {
        let mut schedule = Schedule::default();

        #[cfg(feature = "parallel-reactive")]
        {
            schedule.add_systems((
                update_frame_signal_system,
                timeline_system::<f32>.after(update_frame_signal_system),
                timeline_system::<Color>.after(update_frame_signal_system),
                gather_reactive_changes_system
                    .after(timeline_system::<f32>)
                    .after(timeline_system::<Color>),
                apply_reactive_changes_system.after(gather_reactive_changes_system),
                layout_system.after(apply_reactive_changes_system),
                // These two overlap — different ResMut, same Res<Scene>
                gather_a11y_bounds_system.after(layout_system),
                collect_renderables_system.after(layout_system),
                // Apply a11y bounds after gather completes
                apply_a11y_bounds_system.after(gather_a11y_bounds_system),
                // Run effects at the end of the frame to process any side effects
                run_reactive_effects_system,
            ));
        }

        #[cfg(not(feature = "parallel-reactive"))]
        {
            schedule.add_systems((
                update_frame_signal_system,
                timeline_system::<f32>.after(update_frame_signal_system),
                timeline_system::<Color>.after(update_frame_signal_system),
                update_interaction_state_system.after(update_frame_signal_system),
                // Signal readers must run AFTER timeline writers to avoid
                // rendering stale values from the previous frame.
                update_all_reactive_system
                    .after(timeline_system::<f32>)
                    .after(timeline_system::<Color>),
                update_progress_bar_direct_system.after(timeline_system::<f32>),
                update_widget_style_system.after(update_interaction_state_system),
                layout_system
                    .after(update_all_reactive_system)
                    .after(update_progress_bar_direct_system)
                    .after(update_widget_style_system),
                // These two overlap — different ResMut, same Res<Scene>
                gather_a11y_bounds_system.after(layout_system),
                collect_renderables_system.after(layout_system),
                // Apply a11y bounds after gather completes
                apply_a11y_bounds_system.after(gather_a11y_bounds_system),
                // Run effects at the end of the frame to process any side effects
                run_reactive_effects_system,
            ));
        }

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
        self.frame_schedule.add_systems(system);
    }
}

impl Default for FrameworkContext {
    fn default() -> Self {
        Self::new()
    }
}
