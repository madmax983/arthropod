use super::core::{App, AppError};
use super::integration::integrate_widget_scene;
use crate::event_dispatcher::{DispatchResult, EventDispatcher};
// use crate::layout::auto_layout; // Removed in favor of ECS system
use arthropod_ecs::components::LayoutConstraintsResource;
use flux_state::{Runtime, Signal};
use layout_engine::LayoutConstraints;
use plat_core::{
    Application, ControlFlow, Event, EventLoop, Size, WindowConfig, WindowEvent, WindowId,
};
use render_engine::{
    Color, NodeContent, NodeId, Scene,
    backend::{RenderBackend, WgpuBackend},
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use widget_core::{Widget, WidgetContext};

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
pub(crate) struct WidgetAppConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub builder: WidgetBuilder,
}

/// Extension trait for boxed widgets.
pub trait WidgetExt {
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
    pub(crate) runtime: Arc<Runtime>,
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

/// Internal function to run the widget app.
/// Delegates from `App::run`.
pub fn run_widget_app<F>(title: &str, width: u32, height: u32, build: F) -> Result<(), AppError>
where
    F: FnOnce(&mut AppContext) -> Box<dyn WidgetExt> + 'static,
{
    // Store configuration in thread-local for WidgetApp::new() to retrieve
    WIDGET_APP_CONFIG.with(|cell| {
        let config = WidgetAppConfig {
            title: title.to_string(),
            width,
            height,
            builder: Box::new(build),
        };
        *cell.borrow_mut() = Some(config);
    });

    // Run the app
    plat_core::run::<WidgetApp>().map_err(|e| AppError::WindowCreation(e.to_string()))
}

/// Internal widget application that implements plat_core::Application.
struct WidgetApp {
    app: App,
    widget_ctx: WidgetContext,
    dispatcher: EventDispatcher,
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
        let mut app = App::new_windowed(window_config, event_loop).expect("Failed to create app");

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
        let (_app_root, node_id_map) = integrate_widget_scene(&mut app, &widget_ctx, widget_root);

        // Integrate widget components (LayoutStyle, Clickable, etc.) into ECS
        app.integrate_widgets(&widget_ctx, &node_id_map);

        // Set initial layout constraints
        app.world_mut()
            .insert_resource(LayoutConstraintsResource(LayoutConstraints {
                max_width: Some(config.width as f32),
                max_height: Some(config.height as f32),
                min_width: None,
                min_height: None,
            }));

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

                // Update layout constraints resource
                if let Some(mut constraints) = self
                    .app
                    .world_mut()
                    .get_resource_mut::<LayoutConstraintsResource>()
                {
                    constraints.0 = LayoutConstraints {
                        max_width: Some(size.width as f32),
                        max_height: Some(size.height as f32),
                        min_width: None,
                        min_height: None,
                    };
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
            DispatchResult::Handled => {
                // Button clicks and other handled events may trigger reactive state changes
                // Request a redraw so the reactive system can update the UI
                if let Some(window) = self.app.window() {
                    window.request_redraw();
                }
            }
            DispatchResult::Ignored => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        // Run update systems (layout, reactive, etc.)
        self.app.update();

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
            if !self.widget_ctx.is_text_input(widget_node) {
                continue;
            }

            let Some(value) = self.widget_ctx.get_text_input_value(widget_node) else {
                continue;
            };

            let mut scene = self.app.world_mut().resource_mut::<Scene>();

            // Extract text_child ID first to drop immutable borrow of scene
            let text_child = scene
                .get_node(app_node)
                .and_then(|n| n.children.first().copied());

            let Some(text_child) = text_child else {
                continue;
            };

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

    /// Sync focus visual state.
    fn sync_focus_visuals(&mut self) {
        let focused = self.widget_ctx.focused_node();
        let mut scene = self.app.world_mut().resource_mut::<Scene>();

        // Reset all text input colors to white
        for (&widget_node, &app_node) in &self.node_id_map {
            if !self.widget_ctx.is_text_input(widget_node) {
                continue;
            }

            if let Some(color) =
                scene
                    .get_node_mut(app_node)
                    .and_then(|node| match &mut node.content {
                        NodeContent::Rect { color } => Some(color),
                        _ => None,
                    })
            {
                *color = Color::WHITE;
            }
            if let Some(node) = scene.get_node_mut(app_node) {
                #[allow(clippy::collapsible_if)]
                if let NodeContent::Rect { color } = &mut node.content {
                    *color = Color::WHITE;
                }
            }
        }

        // Highlight focused field
        let Some(focused_widget) = focused else {
            return;
        };

        let Some(&app_node) = self.node_id_map.get(&focused_widget) else {
            return;
        };

        if let Some(color) = scene
            .get_node_mut(app_node)
            .and_then(|node| match &mut node.content {
                NodeContent::Rect { color } => Some(color),
                _ => None,
            })
        {
            *color = Color::rgba(0.7, 0.85, 1.0, 1.0);
        }
        if let Some(node) = scene.get_node_mut(app_node) {
            #[allow(clippy::collapsible_if)]
            if let NodeContent::Rect { color } = &mut node.content {
                *color = Color::rgba(0.7, 0.85, 1.0, 1.0);
            }
        }
    }
}
