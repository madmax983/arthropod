//! Arthropod application builder and runtime
//!
//! Provides a high-level API for creating and managing Arthropod applications.
//! All resources (Scene, Runtime, WgpuBackend) are managed automatically.

use arthropod_ecs::FrameworkContext;
use bevy_ecs::{prelude::*, world::EntityWorldMut};
use flux_state::Runtime;
use plat_core::{EventLoop, Window, WindowConfig};
use render_engine::{
    NodeId,
    backend::{RectInstance, RenderBackend, WgpuBackend},
};
use std::sync::Arc;
use thiserror::Error;

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

        // Create GPU backend
        let backend = WgpuBackend::new(&window, size.width, size.height)
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
    /// Optional window (None in headless mode)
    window: Option<Window>,

    /// ECS framework context (contains Scene as Resource)
    context: FrameworkContext,

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
            window,
            context,
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
}
