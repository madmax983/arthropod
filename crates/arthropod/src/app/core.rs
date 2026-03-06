use arthropod_ecs::{
    FrameworkContext,
    components::{
        BackgroundColor, Clickable, LayoutStyle, ReactiveColor, ReactiveComputedText, ReactiveText,
        SceneNodeRef,
    },
};
use bevy_ecs::{prelude::*, world::EntityWorldMut};
use flux_state::Runtime;
use plat_core::{EventLoop, Window, WindowConfig};
use render_engine::{
    NodeId,
    backend::{PrimitiveInstance, RenderBackend, WgpuBackend},
};
use std::sync::Arc;
use thiserror::Error;
use widget_core::WidgetContext;

use crate::app::AppContext;
use crate::app::widget::run_widget_app;
use widget_core::{Widget, WidgetBoxed};

/// Errors that can occur during app creation or execution
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to create window: {0}")]
    WindowCreation(String),

    #[error("Failed to create GPU backend: {0}")]
    BackendCreation(String),

    #[error("Failed to render: {0}")]
    RenderError(String),
}

/// Arthropod application
///
/// The central coordinator that manages the application lifecycle, including:
/// - **Window Management**: Handling window creation and events (if not headless).
/// - **ECS World**: Storing all resources (Scene, Runtime, WgpuBackend) and entities.
/// - **Reactive Runtime**: Coordinating signals and effects via `flux-state`.
/// - **Rendering**: Managing the render loop and GPU backend.
///
/// `App` acts as the container for the `bevy_ecs` World. All resources are stored in the
/// World and accessed via [`App::world()`] and [`App::world_mut()`].
///
/// # Integration
///
/// `App` provides the [`App::integrate_widgets()`] method to bridge the gap between
/// the high-level widget tree (defined in `widget-core`) and the low-level ECS runtime.
pub struct App {
    /// ECS framework context (contains Scene as Resource)
    ///
    /// NOTE: `context` MUST be declared before `window` to ensure correct drop order.
    /// `context` owns the `WgpuBackend` (as a resource), which owns the `wgpu::Surface`.
    /// The `Surface` holds a reference to `Window` but is `'static`.
    /// We must ensure the backend is dropped (and the surface destroyed) BEFORE the window is closed
    /// to avoid Undefined Behavior.
    context: FrameworkContext,

    /// Optional window (None in headless mode)
    window: Option<Window>,

    /// Reactive runtime (also stored in World as Resource for signals)
    #[allow(dead_code)]
    runtime: Arc<Runtime>,
}

impl App {
    /// Create a new App with a window and GPU backend.
    ///
    /// This is the standard way to create an application with a visible window.
    pub fn new_windowed(config: WindowConfig, event_loop: &EventLoop) -> Result<Self, AppError> {
        // Create window
        let window = event_loop
            .create_window(config)
            .map_err(|e| AppError::WindowCreation(e.to_string()))?;

        let size = window.inner_size();

        // Create GPU backend (standard mode - not using DirectComposition)
        // SAFETY: Safe because `App` struct guarantees correct drop order (context before window).
        let backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .map_err(|e| AppError::BackendCreation(e.to_string()))?;

        Ok(Self::new_with_backend(Some(window), Some(backend)))
    }

    /// Create a new headless App (no window, no GPU).
    ///
    /// Use this for testing, benchmarking, or server-side rendering.
    ///
    /// Returns `Ok(App)` on success. Since headless mode doesn't create a window or GPU context,
    /// it rarely fails, but is fallible to maintain API consistency.
    pub fn new_headless() -> Result<Self, AppError> {
        Ok(Self::new_with_backend(None, None))
    }

    /// Create a new App with optional backend
    ///
    /// Internal constructor used by new_windowed and new_headless.
    fn new_with_backend(window: Option<Window>, backend: Option<WgpuBackend>) -> Self {
        // Create reactive runtime (Runtime::new() already returns Arc<Runtime>)
        // Runtime uses Mutex (thread-safe), so we keep it in App and provide accessor methods
        let runtime = Runtime::new();

        // Create ECS context (Scene is already inserted as Resource)
        let mut context = FrameworkContext::new();

        // Insert WgpuBackend as Resource (if present)
        if let Some(backend) = backend {
            context.world_mut().insert_resource(backend);
        }

        #[cfg_attr(not(feature = "nova"), allow(unused_mut))]
        let mut app = Self {
            context, // Drops first
            window,  // Drops last
            runtime,
        };

        #[cfg(feature = "nova")]
        crate::experimental::xray::register_xray(&mut app);

        app
    }

    /// Access the ECS World (immutable)
    ///
    /// Use this to read resources like [`render_engine::Scene`] or [`Runtime`].
    ///
    /// # Example
    ///
    /// ```
    /// # use arthropod::prelude::*;
    /// # use render_engine::Scene;
    /// let app = App::new_headless().expect("Failed to create app");
    ///
    /// // Read the Scene resource
    /// let scene = app.world().resource::<Scene>();
    /// println!("Scene has {} nodes", scene.nodes().count());
    /// ```
    pub fn world(&self) -> &World {
        self.context.world()
    }

    /// Access the ECS World (mutable)
    ///
    /// Use this to modify resources, spawn entities, or run queries.
    ///
    /// # Example
    ///
    /// ```
    /// # use arthropod::prelude::*;
    /// # use render_engine::Scene;
    /// let mut app = App::new_headless().expect("Failed to create app");
    ///
    /// // Modify the Scene resource
    /// let mut scene = app.world_mut().resource_mut::<Scene>();
    /// let root = scene.root();
    /// // ... modify scene ...
    /// ```
    pub fn world_mut(&mut self) -> &mut World {
        self.context.world_mut()
    }

    /// Spawn a new entity linked to a scene node
    ///
    /// Creates a new ECS entity and adds a [`SceneNodeRef`] component pointing to the given `node_id`.
    /// This links the high-level Scene Graph to the ECS world, allowing you to attach components
    /// like `Renderable`, `Clickable`, or `ReactiveColor` to scene nodes.
    ///
    /// # Example
    ///
    /// ```
    /// # use arthropod::prelude::*;
    /// let mut app = App::new_headless().unwrap();
    /// let root = app.world().resource::<Scene>().root();
    ///
    /// // Spawn an entity linked to the root node and mark it as renderable
    /// app.spawn(root).insert(Renderable);
    /// ```
    pub fn spawn(&mut self, node_id: NodeId) -> EntityWorldMut<'_> {
        self.context.spawn(node_id)
    }

    /// Get a mutable reference to an entity by its NodeId
    ///
    /// Finds the entity that has a [`SceneNodeRef`] component matching the given `node_id`.
    /// Returns `None` if no such entity exists.
    ///
    /// This is useful for adding components to an existing widget's entity.
    ///
    /// # Example
    ///
    /// ```
    /// # use arthropod::prelude::*;
    /// let mut app = App::new_headless().unwrap();
    /// let root = app.world().resource::<Scene>().root();
    /// app.spawn(root); // Create the entity first
    ///
    /// // Later, retrieve it to add more components
    /// if let Some(mut entity) = app.get_entity_mut(root) {
    ///     entity.insert(Renderable);
    /// }
    /// ```
    pub fn get_entity_mut(&mut self, node_id: NodeId) -> Option<EntityWorldMut<'_>> {
        // Query for entity with matching SceneNodeRef
        let mut query = self.world_mut().query::<(Entity, &SceneNodeRef)>();

        // Find the entity with matching node_id
        let entity = query
            .iter(self.world())
            .find(|(_, scene_ref)| scene_ref.0 == node_id)
            .map(|(entity, _)| entity);

        // Get mutable access to the entity if found
        entity.and_then(|e| self.world_mut().get_entity_mut(e).ok())
    }

    /// Run all update systems (reactive signals, animations, etc.)
    ///
    /// Call this once per frame before rendering.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arthropod::prelude::*;
    /// # let mut app = App::new_headless().unwrap();
    /// app.update();
    /// ```
    pub fn update(&mut self) {
        self.context.update();
    }

    /// Add a system to the update loop
    ///
    /// Allows registering custom systems (e.g., from plugins or experimental modules)
    /// to run during the application update cycle.
    ///
    /// # Example
    ///
    /// ```
    /// # use arthropod::prelude::*;
    /// # use bevy_ecs::prelude::*;
    /// # let mut app = App::new_headless().unwrap();
    /// fn my_system() {
    ///     println!("Updating!");
    /// }
    ///
    /// app.add_update_system(my_system);
    /// ```
    pub fn add_update_system<M>(&mut self, system: impl IntoSystemConfigs<M>) {
        self.context.add_update_system(system);
    }

    /// Run render systems and collect GPU instances
    ///
    /// In headless mode, this returns the instances without rendering.
    /// With a backend, use `render_to_gpu()` instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arthropod::prelude::*;
    /// # let mut app = App::new_headless().unwrap();
    /// let instances = app.render();
    /// assert!(instances.len() >= 0);
    /// ```
    pub fn render(&mut self) -> Vec<PrimitiveInstance> {
        self.context.render()
    }

    /// Render to GPU (only available with backend)
    ///
    /// Call this after `update()` to render the frame.
    ///
    /// # Example
    ///
    /// ```
    /// # use arthropod::prelude::*;
    /// # let mut app = App::new_headless().unwrap();
    /// app.render_to_gpu().expect("Render failed");
    /// ```
    pub fn render_to_gpu(&mut self) -> Result<(), AppError> {
        // Collect instances from ECS
        let instances = self.context.render();

        // Render to GPU if backend is available
        if let Some(mut backend) = self.world_mut().get_resource_mut::<WgpuBackend>() {
            backend
                .render_instances(&instances)
                .map_err(|e| AppError::RenderError(e.to_string()))?;
        }

        Ok(())
    }

    /// Resize the GPU backend
    ///
    /// Call this when the window is resized.
    ///
    /// # Example
    ///
    /// ```
    /// # use arthropod::prelude::*;
    /// # let mut app = App::new_headless().unwrap();
    /// app.resize(1024, 768);
    /// ```
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(mut backend) = self.world_mut().get_resource_mut::<WgpuBackend>() {
            backend.resize(width, height);
        }
    }

    /// Get a reference to the window (if present)
    pub fn window(&self) -> Option<&Window> {
        self.window.as_ref()
    }

    /// Get a reference to the reactive runtime
    ///
    /// Use this to create signals and effects.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arthropod::prelude::*;
    /// # let app = App::new_headless().unwrap();
    /// let runtime = app.runtime();
    /// let signal = Signal::new(runtime.clone(), 42);
    /// ```
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    // =========================================================================
    // Widget Integration (App-Shell Layer)
    // =========================================================================

    /// Helper to transfer components from widget context to ECS world
    fn transfer_components<'a, T, C, F>(
        &mut self,
        source_data: impl Iterator<Item = (&'a NodeId, &'a T)>,
        component_factory: F,
    ) where
        T: 'a,
        C: Component,
        F: Fn(&T) -> C,
    {
        for (node_id, data) in source_data {
            // Assume widget_node_id == app_node_id
            if let Some(mut entity) = self.get_entity_mut(*node_id) {
                entity.insert(component_factory(data));
            }
        }
    }

    /// Integrate widgets from a WidgetContext into the ECS world.
    ///
    /// This method acts as the **ECS Bridge**, transferring static and reactive state from
    /// the ECS-agnostic `WidgetContext` to the `arthropod-ecs` World.
    ///
    /// It specifically handles:
    /// - **Layout**: Transfers [`layout_engine::FlexStyle`] to [`LayoutStyle`] components.
    /// - **Interactivity**: Transfers click handlers to [`Clickable`] components.
    /// - **Visuals**: Transfers background colors and reactive color states to [`BackgroundColor`]
    ///   and [`ReactiveColor`] components.
    ///
    /// # Architecture Note
    ///
    /// While this method handles *static* component registration, dynamic behavior (like
    /// text input handling, focus management, and form validation) is currently managed
    /// by the application controller loop (e.g., `WidgetApp`). This separation ensures
    /// that the ECS remains focused on data and systems, while the event loop handles
    /// immediate user interaction.
    ///
    /// # Example
    ///
    /// ```
    /// use arthropod::prelude::*;
    /// use render_engine::Scene;
    ///
    /// // 1. Create App (which has a Scene)
    /// let mut app = App::new_headless().unwrap();
    /// let mut scene = app.world_mut().remove_resource::<Scene>().unwrap();
    ///
    /// // 2. Build widgets directly into the scene
    /// let mut widget_ctx = WidgetContext::new(scene);
    /// let widget_root = widget_ctx.create_node(widget_ctx.root(), NodeContent::Empty);
    ///
    /// // 3. Return scene to App
    /// app.world_mut().insert_resource(widget_ctx.into_scene());
    /// app.spawn(widget_root); // Create ECS entity
    ///
    /// // 4. Transfer components
    /// // Note: In this new architecture, WidgetContext keys ARE App NodeIds
    /// // So we pass the context directly to integrate_widgets
    /// // (Hypothetically, as integrate_widgets is called internally by WidgetApp)
    /// // app.integrate_widgets(&widget_ctx);
    /// ```
    pub fn integrate_widgets(&mut self, widget_ctx: &WidgetContext) {
        // Transfer layout styles to LayoutStyle components
        self.transfer_components(
            widget_ctx.layout_styles().iter(),
            |style: &layout_engine::FlexStyle| LayoutStyle(style.clone()),
        );

        // Transfer clickables to Clickable components
        self.transfer_components(
            widget_ctx.clickables().iter(),
            |callback: &std::sync::Arc<dyn Fn() + Send + Sync>| Clickable {
                callback: callback.clone(),
            },
        );

        // Transfer background colors to BackgroundColor components
        self.transfer_components(
            widget_ctx.background_colors().iter(),
            |color: &render_engine::Vec4| BackgroundColor(*color),
        );

        // Transfer reactive colors to ReactiveColor components
        self.transfer_components(
            widget_ctx.reactive_color_states().iter(),
            |state: &widget_core::input_state::ReactiveColorState| {
                ReactiveColor::new(state.read_signal.clone())
            },
        );

        // Transfer reactive text to ReactiveText components
        self.transfer_components(
            widget_ctx.reactive_text_states().iter(),
            |state: &widget_core::input_state::ReactiveTextState| {
                ReactiveText::new(state.read_signal.clone())
            },
        );

        // Transfer computed text to ReactiveComputedText components
        self.transfer_components(
            widget_ctx.computed_text_states().iter(),
            |state: &widget_core::input_state::ComputedTextState| {
                ReactiveComputedText::new(state.computed.clone())
            },
        );

        // Note: Complex state (TextInput, Form) is handled by the higher-level
        // WidgetApp controller, not by direct ECS component transfer at this time.
    }

    /// Run a widget-based application.
    ///
    /// This is the simplest way to create an Arthropod app. It handles:
    /// - Window creation
    /// - Event loop
    /// - Event dispatch to widgets
    /// - Auto-layout
    /// - Rendering
    ///
    /// # Arguments
    ///
    /// * `title` - Window title
    /// * `width` - Window width in pixels
    /// * `height` - Window height in pixels
    /// * `build` - Builder function that receives `AppContext` and returns a widget
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arthropod::prelude::*;
    ///
    /// fn main() -> Result<(), AppError> {
    ///     App::run("Welcome App", 400, 300, |ctx| {
    ///         // 1. Create reactive state (a string)
    ///         let name_signal = ctx.signal(String::from("Arthropod"));
    ///         let (read_name, _write_name) = name_signal.split();
    ///
    ///         // 2. Build declarative UI tree using macros
    ///         Center::new(
    ///             col!(
    ///                 [
    ///                     txt!("Welcome to", size: 24.0),
    ///                     // Use @ syntax to pass the reactive signal
    ///                     txt!(@read_name, size: 48.0),
    ///                 ],
    ///                 gap: 20.0,
    ///                 padding: 40.0
    ///             )
    ///         )
    ///     })
    /// }
    /// ```
    pub fn run<W, F>(title: &str, width: u32, height: u32, build: F) -> Result<(), AppError>
    where
        W: Widget + 'static,
        F: FnOnce(&mut AppContext) -> W + 'static,
    {
        run_widget_app(title, width, height, |ctx| {
            let widget = build(ctx);
            Box::new(widget) as Box<dyn WidgetBoxed>
        })
    }
}
