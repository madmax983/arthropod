//! Arthropod application builder and runtime
//!
//! Provides a high-level API for creating and managing Arthropod applications.
//! All resources (Scene, Runtime, WgpuBackend) are managed automatically.
//!
//! # High-Level API
//!
//! For widget-based applications, use [`App::run()`]:
//!
//! ```no_run
//! use arthropod::prelude::*;
//! use widget_core::{Form, TextInput};
//!
//! fn main() -> Result<(), AppError> {
//!     App::run("My Form", 400, 300, |ctx| {
//!         let name = ctx.signal(String::new());
//!         Form::new((
//!             ("name", TextInput::new(name)),
//!         ))
//!     })
//! }
//! ```

// Allow collapsible_if since nested if-let chains are more readable in this context
#![allow(clippy::collapsible_if)]

use crate::event_dispatcher::{DispatchResult, EventDispatcher};
use crate::layout::auto_layout;
use arthropod_ecs::{
    FrameworkContext, Renderable,
    components::{BackgroundColor, Clickable, LayoutStyle, SceneNodeRef},
};
use bevy_ecs::{prelude::*, world::EntityWorldMut};
use flux_state::{Runtime, Signal};
use plat_core::{
    Application, ControlFlow, Event, EventLoop, Size, Window, WindowConfig, WindowEvent, WindowId,
};
use render_engine::{
    Color, NodeContent, NodeId, Scene, SceneNode,
    backend::{RectInstance, RenderBackend, WgpuBackend},
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use widget_core::{Widget, WidgetContext};

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

/// Arthropod application builder
///
/// Use this to configure and create an `App` instance.
///
/// # Example
///
/// ```
/// use arthropod::prelude::*;
///
/// let config = WindowConfig {
///     title: "My App".to_string(),
///     size: Size { width: 800, height: 600 },
///     ..Default::default()
/// };
///
/// // For testing/headless mode
/// let app = AppBuilder::new()
///     .with_window_config(config)
///     .build_headless()
///     .expect("Failed to create app");
/// ```
pub struct AppBuilder {
    window_config: Option<WindowConfig>,
}

impl AppBuilder {
    /// Create a new AppBuilder
    pub fn new() -> Self {
        Self {
            window_config: None,
        }
    }

    /// Set the window configuration
    pub fn with_window_config(mut self, config: WindowConfig) -> Self {
        self.window_config = Some(config);
        self
    }

    /// Build the application with a real window and GPU backend
    ///
    /// Requires an EventLoop to create the window.
    /// Use this for real applications.
    pub fn build(self, event_loop: &EventLoop) -> Result<App, AppError> {
        let config = self.window_config.unwrap_or_default();

        // Create window
        let window = event_loop
            .create_window(config)
            .map_err(|e| AppError::WindowCreation(e.to_string()))?;

        let size = window.inner_size();

        // Create GPU backend (standard mode - not using DirectComposition)
        // SAFETY: Safe because `App` struct guarantees correct drop order (context before window).
        let backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .map_err(|e| AppError::BackendCreation(e.to_string()))?;

        // Create app with backend
        Ok(App::new_with_backend(Some(window), Some(backend)))
    }

    /// Build the application in headless mode (no window, no GPU)
    ///
    /// Use this for testing and benchmarking.
    /// render() will collect instances but not actually render to GPU.
    pub fn build_headless(self) -> Result<App, AppError> {
        Ok(App::new_with_backend(None, None))
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Arthropod application
///
/// Manages the entire application lifecycle including:
/// - Window and GPU backend (if not headless)
/// - ECS World with all resources (Scene, Runtime, WgpuBackend)
/// - Reactive runtime
/// - Frame updates and rendering
///
/// All resources are stored in the ECS World and accessed via `world()` / `world_mut()`.
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
    /// Create a new App with optional backend
    ///
    /// Internal constructor used by AppBuilder.
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

        Self {
            context, // Drops first
            window,  // Drops last
            runtime,
        }
    }

    /// Access the ECS World (immutable)
    ///
    /// Use this to read resources like Scene, Runtime, etc.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arthropod::prelude::*;
    /// # let app = AppBuilder::new().build_headless().unwrap();
    /// let scene = app.world().resource::<Scene>();
    /// let root = scene.root();
    /// ```
    pub fn world(&self) -> &World {
        self.context.world()
    }

    /// Access the ECS World (mutable)
    ///
    /// Use this to modify resources or spawn entities.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arthropod::prelude::*;
    /// # let mut app = AppBuilder::new().build_headless().unwrap();
    /// let mut scene = app.world_mut().resource_mut::<Scene>();
    /// let root = scene.root();
    /// ```
    pub fn world_mut(&mut self) -> &mut World {
        self.context.world_mut()
    }

    /// Spawn a new entity linked to a scene node
    ///
    /// This is a convenience method that wraps `FrameworkContext::spawn()`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arthropod::prelude::*;
    /// # let mut app = AppBuilder::new().build_headless().unwrap();
    /// # let node_id = app.world().resource::<Scene>().root();
    /// app.spawn(node_id).insert(Renderable);
    /// ```
    pub fn spawn(&mut self, node_id: NodeId) -> EntityWorldMut<'_> {
        self.context.spawn(node_id)
    }

    /// Get a mutable reference to an entity by its NodeId
    ///
    /// Finds the entity with a SceneNodeRef component matching the given NodeId.
    /// Returns None if no entity is linked to this node.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arthropod::prelude::*;
    /// # let mut app = AppBuilder::new().build_headless().unwrap();
    /// # let node_id = app.world().resource::<Scene>().root();
    /// # app.spawn(node_id).insert(Renderable);
    /// if let Some(mut entity) = app.get_entity_mut(node_id) {
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
    /// # let mut app = AppBuilder::new().build_headless().unwrap();
    /// app.update();
    /// ```
    pub fn update(&mut self) {
        self.context.update();
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
    /// # let mut app = AppBuilder::new().build_headless().unwrap();
    /// let instances = app.render();
    /// assert!(instances.len() >= 0);
    /// ```
    pub fn render(&mut self) -> Vec<RectInstance> {
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
    /// # let mut app = AppBuilder::new()
    /// #     .with_window_config(WindowConfig::default())
    /// #     .build_headless()
    /// #     .unwrap();
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
    /// # let mut app = AppBuilder::new()
    /// #     .with_window_config(WindowConfig::default())
    /// #     .build_headless()
    /// #     .unwrap();
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
    /// # let app = AppBuilder::new().build_headless().unwrap();
    /// let runtime = app.runtime();
    /// let signal = Signal::new(runtime.clone(), 42);
    /// ```
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    // =========================================================================
    // Widget Integration (App-Shell Layer)
    // =========================================================================

    /// Integrate widgets from a WidgetContext into the ECS world
    ///
    /// This is the bridge between the widget-core layer (ECS-agnostic) and
    /// the ECS runtime. It transfers all accumulated widget state (layout styles,
    /// clickables, validators, etc.) to ECS components.
    ///
    /// The `node_id_map` maps widget NodeIds to app NodeIds, since widget scenes
    /// are typically copied into the app scene with new NodeIds.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arthropod::prelude::*;
    /// use widget_core::WidgetContext;
    /// use std::collections::HashMap;
    ///
    /// let mut widget_ctx = WidgetContext::new_test();
    /// // ... build widgets ...
    ///
    /// let mut app = AppBuilder::new()
    ///     .with_window_config(WindowConfig::default())
    ///     .build_headless()
    ///     .unwrap();
    ///
    /// // Copy widget scene nodes to app scene, building up node_id_map
    /// let node_id_map: HashMap<NodeId, NodeId> = HashMap::new();
    /// // ... copy nodes ...
    ///
    /// // Transfer widget state to ECS components
    /// app.integrate_widgets(&widget_ctx, &node_id_map);
    /// ```
    pub fn integrate_widgets(
        &mut self,
        widget_ctx: &WidgetContext,
        node_id_map: &HashMap<NodeId, NodeId>,
    ) {
        // Transfer layout styles to LayoutStyle components
        for (widget_node_id, style) in widget_ctx.layout_styles() {
            if let Some(&app_node_id) = node_id_map.get(widget_node_id) {
                if let Some(mut entity) = self.get_entity_mut(app_node_id) {
                    entity.insert(LayoutStyle(style.clone()));
                }
            }
        }

        // Transfer clickables to Clickable components
        for (widget_node_id, callback) in widget_ctx.clickables() {
            if let Some(&app_node_id) = node_id_map.get(widget_node_id) {
                if let Some(mut entity) = self.get_entity_mut(app_node_id) {
                    entity.insert(Clickable {
                        callback: callback.clone(),
                    });
                }
            }
        }

        // Transfer background colors to BackgroundColor components
        for (widget_node_id, color) in widget_ctx.background_colors() {
            if let Some(&app_node_id) = node_id_map.get(widget_node_id) {
                if let Some(mut entity) = self.get_entity_mut(app_node_id) {
                    entity.insert(BackgroundColor(*color));
                }
            }
        }

        // Note: TextInputState, Validator, and FormState components require
        // additional implementation in arthropod-ecs. For now, we handle the
        // core components (layout, clickable, background color).
        // TODO: Add text_input_states, validators, form_states when needed
    }

    // =========================================================================
    // High-Level Widget App API
    // =========================================================================

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
    /// use widget_core::{Form, TextInput};
    ///
    /// fn main() -> Result<(), AppError> {
    ///     App::run("Registration", 400, 300, |ctx| {
    ///         let name = ctx.signal(String::new());
    ///         Form::new((
    ///             ("name", TextInput::new(name).placeholder("Name")),
    ///         ))
    ///     })
    /// }
    /// ```
    pub fn run<W, F>(title: &str, width: u32, height: u32, build: F) -> Result<(), AppError>
    where
        W: Widget + 'static,
        F: FnOnce(&mut AppContext) -> W + 'static,
    {
        // Store configuration in thread-local for WidgetApp::new() to retrieve
        WIDGET_APP_CONFIG.with(|cell| {
            let config = WidgetAppConfig {
                title: title.to_string(),
                width,
                height,
                builder: Box::new(move |ctx: &mut AppContext| {
                    let widget = build(ctx);
                    Box::new(widget) as Box<dyn WidgetExt>
                }),
            };
            *cell.borrow_mut() = Some(config);
        });

        // Run the app
        plat_core::run::<WidgetApp>().map_err(|e| AppError::WindowCreation(e.to_string()))
    }
}

// =============================================================================
// High-Level Widget App Infrastructure
// =============================================================================

// Thread-local storage for widget app configuration.
// Used to pass config from App::run() to WidgetApp::new().
thread_local! {
    static WIDGET_APP_CONFIG: RefCell<Option<WidgetAppConfig>> = const { RefCell::new(None) };
}

/// Type alias for widget builder function to reduce type complexity.
type WidgetBuilder = Box<dyn FnOnce(&mut AppContext) -> Box<dyn WidgetExt>>;

/// Configuration passed to WidgetApp via thread-local.
struct WidgetAppConfig {
    title: String,
    width: u32,
    height: u32,
    builder: WidgetBuilder,
}

/// Extension trait for boxed widgets.
trait WidgetExt {
    fn build_boxed(&self, ctx: &mut WidgetContext) -> NodeId;
}

impl<W: Widget + 'static> WidgetExt for W {
    fn build_boxed(&self, ctx: &mut WidgetContext) -> NodeId {
        self.build(ctx)
    }
}

/// Context provided to widget builders in App::run().
///
/// Provides convenient access to create signals for reactive state.
pub struct AppContext {
    runtime: Arc<Runtime>,
}

impl AppContext {
    /// Create a new signal with the given initial value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arthropod::prelude::*;
    /// # use widget_core::Text;
    /// App::run("Test", 400, 300, |ctx| {
    ///     let _counter = ctx.signal(0);
    ///     let _name = ctx.signal(String::new());
    ///
    ///     // Return a widget...
    ///     Text::new("Placeholder")
    /// });
    /// ```
    pub fn signal<T: Clone + Send + Sync + 'static>(&self, initial: T) -> Signal<T> {
        Signal::new(self.runtime.clone(), initial)
    }

    /// Get the reactive runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }
}

/// Internal widget application that implements plat_core::Application.
struct WidgetApp {
    app: App,
    widget_ctx: WidgetContext,
    dispatcher: EventDispatcher,
    app_root: NodeId,
    node_id_map: HashMap<NodeId, NodeId>,
    viewport: (u32, u32),
}

impl Application for WidgetApp {
    fn new(event_loop: &EventLoop) -> Self {
        // Retrieve config from thread-local
        let config = WIDGET_APP_CONFIG.with(|cell| {
            cell.borrow_mut()
                .take()
                .expect("WidgetAppConfig not set - use App::run()")
        });

        // Initialize logging (using env_logger for simplicity)
        let _ = env_logger::try_init();

        // Create window config
        let window_config = WindowConfig {
            title: config.title,
            size: Size {
                width: config.width,
                height: config.height,
            },
            resizable: true,
            decorations: true,
            visible: true,
            ..Default::default()
        };

        // Create app with window
        let mut app = AppBuilder::new()
            .with_window_config(window_config)
            .build(event_loop)
            .expect("Failed to create app");

        let runtime = app.runtime().clone();

        // Create app context for builder
        let mut app_ctx = AppContext { runtime };

        // Build widget tree
        let widget = (config.builder)(&mut app_ctx);
        let mut widget_ctx = WidgetContext::new_test();
        let widget_root = widget.build_boxed(&mut widget_ctx);

        // Detect form node (for Enter submission)
        let form_node = widget_ctx.form_states().keys().next().copied();

        // Integrate widget scene into app scene
        let (app_root, node_id_map) = integrate_widget_scene(&mut app, &widget_ctx, widget_root);

        // Perform initial layout
        let layout_styles = widget_ctx.layout_styles().clone();
        {
            let mut scene = app.world_mut().resource_mut::<Scene>();
            auto_layout(
                &mut scene,
                app_root,
                config.width as f32,
                config.height as f32,
                &layout_styles,
            );
        }

        // Create event dispatcher
        let dispatcher = EventDispatcher::new(node_id_map.clone(), form_node);

        println!("=== Widget App Started ===");
        println!("Interactions:");
        println!("  - Click on field to focus");
        println!("  - Type to enter text");
        println!("  - Tab to move to next field");
        println!("  - Enter to submit form");

        Self {
            app,
            widget_ctx,
            dispatcher,
            app_root,
            node_id_map,
            viewport: (config.width, config.height),
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match &event {
            Event::Window {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
                return;
            }
            Event::Window {
                event: WindowEvent::Resized(size),
                ..
            } => {
                self.viewport = (size.width, size.height);
                self.app.resize(size.width, size.height);

                // Re-layout
                let layout_styles = self.widget_ctx.layout_styles().clone();
                {
                    let mut scene = self.app.world_mut().resource_mut::<Scene>();
                    auto_layout(
                        &mut scene,
                        self.app_root,
                        size.width as f32,
                        size.height as f32,
                        &layout_styles,
                    );
                }

                if let Some(window) = self.app.window() {
                    window.request_redraw();
                }
                return;
            }
            _ => {}
        }

        // Dispatch event to widgets
        // We need to do hit testing separately to avoid borrow conflicts
        let hit_result = {
            let scene = self.app.world().resource::<Scene>();
            self.dispatcher
                .dispatch(&event, &mut self.widget_ctx, scene)
        };

        match hit_result {
            DispatchResult::TextChanged => {
                self.sync_text_nodes();
                if let Some(window) = self.app.window() {
                    window.request_redraw();
                }
            }
            DispatchResult::FocusChanged => {
                self.sync_focus_visuals();
                if let Some(window) = self.app.window() {
                    window.request_redraw();
                }
            }
            DispatchResult::FormSubmitted => {
                // Check for validation errors
                if let Some(form_node) = self.dispatcher.form_node() {
                    if self.widget_ctx.has_submit_error(form_node) {
                        println!(
                            "Submit error: {}",
                            self.widget_ctx
                                .get_submit_error(form_node)
                                .unwrap_or_default()
                        );
                    } else if !self.widget_ctx.is_form_valid(form_node) {
                        println!("Cannot submit: Form has validation errors");
                        for (field, error) in self.widget_ctx.get_form_field_errors(form_node) {
                            println!("  {}: {}", field, error);
                        }
                    }
                }
            }
            DispatchResult::Handled | DispatchResult::Ignored => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        // Take backend to avoid borrow conflicts
        let mut backend = self.app.world_mut().remove_resource::<WgpuBackend>();

        if let Some(ref mut backend) = backend {
            let scene = self.app.world().resource::<Scene>();
            if let Err(e) = backend.render(scene) {
                eprintln!("Render error: {}", e);
            }
        }

        // Put backend back
        if let Some(backend) = backend {
            self.app.world_mut().insert_resource(backend);
        }
    }
}

impl WidgetApp {
    /// Sync text nodes in app scene with widget context values.
    fn sync_text_nodes(&mut self) {
        const FONT_SIZE: f32 = 16.0;
        const TEXT_COLOR: Color = Color::rgba(0.2, 0.2, 0.2, 1.0);
        const PLACEHOLDER_COLOR: Color = Color::rgba(0.5, 0.5, 0.5, 1.0);

        for (&widget_node, &app_node) in &self.node_id_map {
            if self.widget_ctx.is_text_input(widget_node) {
                if let Some(value) = self.widget_ctx.get_text_input_value(widget_node) {
                    let mut scene = self.app.world_mut().resource_mut::<Scene>();
                    if let Some(input_node) = scene.get_node(app_node) {
                        if let Some(&text_child) = input_node.children.first() {
                            if let Some(text_node) = scene.get_node_mut(text_child) {
                                let color = if value.is_empty() {
                                    PLACEHOLDER_COLOR
                                } else {
                                    TEXT_COLOR
                                };
                                text_node.content = NodeContent::Text {
                                    text: value.clone(),
                                    font_size: FONT_SIZE,
                                    color,
                                };
                            }
                        }
                    }
                }
            }
        }
    }

    /// Sync focus visual state.
    fn sync_focus_visuals(&mut self) {
        let focused = self.widget_ctx.focused_node();
        let mut scene = self.app.world_mut().resource_mut::<Scene>();

        // Reset all text input colors to white
        for (&widget_node, &app_node) in &self.node_id_map {
            if self.widget_ctx.is_text_input(widget_node) {
                if let Some(node) = scene.get_node_mut(app_node) {
                    if matches!(node.content, NodeContent::Rect { .. }) {
                        node.content = NodeContent::Rect {
                            color: Color::WHITE,
                        };
                    }
                }
            }
        }

        // Highlight focused field
        if let Some(focused_widget) = focused {
            if let Some(&app_node) = self.node_id_map.get(&focused_widget) {
                if let Some(node) = scene.get_node_mut(app_node) {
                    node.content = NodeContent::Rect {
                        color: Color::rgba(0.7, 0.85, 1.0, 1.0),
                    };
                }
            }
        }
    }
}

/// Integrate widget scene into app scene.
fn integrate_widget_scene(
    app: &mut App,
    widget_ctx: &WidgetContext,
    widget_root: NodeId,
) -> (NodeId, HashMap<NodeId, NodeId>) {
    let widget_scene = widget_ctx.scene();
    let mut node_id_map = HashMap::new();

    fn copy_recursive(
        src: &Scene,
        dst: &mut Scene,
        src_id: NodeId,
        dst_parent: NodeId,
        map: &mut HashMap<NodeId, NodeId>,
    ) -> NodeId {
        let src_node = src.get_node(src_id).unwrap();

        let mut new_node = SceneNode::new(src_node.content.clone());
        new_node.bounds = src_node.bounds;
        new_node.visible = src_node.visible;
        new_node.opacity = src_node.opacity;

        let dst_id = dst.add_node(dst_parent, new_node);
        map.insert(src_id, dst_id);

        let children = src_node.children.clone();
        for child_id in children {
            copy_recursive(src, dst, child_id, dst_id, map);
        }

        dst_id
    }

    let app_root = {
        let mut scene = app.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        copy_recursive(
            widget_scene,
            &mut scene,
            widget_root,
            root,
            &mut node_id_map,
        )
    };

    // Spawn Renderable entities
    for &app_node in node_id_map.values() {
        app.spawn(app_node).insert(Renderable);
    }

    (app_root, node_id_map)
}
