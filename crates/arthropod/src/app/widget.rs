use super::core::{App, AppError};
use super::integration::integrate_widget_scene;
use crate::event_dispatcher::{DispatchResult, EventDispatcher};
// use crate::layout::auto_layout; // Removed in favor of ECS system
use arthropod_ecs::components::{FrameSignalResource, LayoutConstraintsResource};
use flux_state::{ReadSignal, Runtime, Signal};
use layout_engine::LayoutConstraints;
use plat_core::{
    Application, ControlFlow, Event, EventLoop, Size, WindowConfig, WindowEvent, WindowId,
};
use render_engine::{Color, NodeContent, Scene, backend::WgpuBackend};
use std::cell::RefCell;
use std::sync::Arc;
use theme_engine::DesignTokens;
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
type WidgetBuilder = Box<dyn FnOnce(&mut AppContext) -> Box<dyn Widget>>;

/// Configuration passed to WidgetApp via thread-local.
pub(crate) struct WidgetAppConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub builder: WidgetBuilder,
}

/// Context provided to widget builders in App::run().
///
/// Provides convenient access to create signals for reactive state.
pub struct AppContext {
    pub(crate) runtime: Arc<Runtime>,
    pub(crate) widget_ctx: WidgetContext,
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

    /// Set design tokens for theming.
    pub fn set_design_tokens(&mut self, tokens: DesignTokens) {
        self.widget_ctx.set_design_tokens(tokens);
    }

    /// Get a signal that updates every frame.
    ///
    /// Useful for driving animations and time-based reactive logic.
    pub fn frame_signal(&self) -> ReadSignal<u64> {
        self.widget_ctx.frame_signal()
    }

    /// Store an effect to keep it alive.
    pub fn store_effect(&mut self, effect: flux_state::Effect) {
        self.widget_ctx.store_effect(effect);
    }

    /// Get the reactive runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    /// Attach a `Timeline<f32>` that drives a signal each frame.
    ///
    /// The timeline is ticked by the ECS `timeline_system` and writes
    /// its sampled value to `target` every frame.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arthropod::prelude::*;
    /// # use std::time::Duration;
    /// App::run("Test", 400, 300, |ctx| {
    ///     let sig = ctx.signal(0.0_f32);
    ///     let (read, write) = sig.split();
    ///     ctx.add_timeline_f32(
    ///         anim_graph::timeline::Timeline::tween(0.0, 1.0, Duration::from_secs(2)).loop_forever(),
    ///         write,
    ///     );
    ///     widget_core::Text::new("Placeholder")
    /// });
    /// ```
    pub fn add_timeline_f32(
        &mut self,
        timeline: anim_graph::timeline::Timeline<f32>,
        target: flux_state::WriteSignal<f32>,
    ) {
        // Use a sentinel NodeId — the timeline isn't attached to a scene node,
        // it drives a signal that widgets read from.
        let count = self.widget_ctx.timeline_f32_states().len() as u64;
        let id = render_engine::NodeId(u64::MAX.wrapping_sub(count));
        self.widget_ctx.add_timeline_f32(id, timeline, target);
    }

    /// Attach a `Timeline<Color>` that drives a color signal each frame.
    pub fn add_timeline_color(
        &mut self,
        timeline: anim_graph::timeline::Timeline<render_engine::Color>,
        target: flux_state::WriteSignal<render_engine::Color>,
    ) {
        let count = self.widget_ctx.timeline_color_states().len() as u64;
        let id = render_engine::NodeId(u64::MAX.wrapping_sub(count + 10000));
        self.widget_ctx.add_timeline_color(id, timeline, target);
    }
}

/// Internal function to run the widget app.
/// Delegates from `App::run`.
pub fn run_widget_app<F>(title: &str, width: u32, height: u32, build: F) -> Result<(), AppError>
where
    F: FnOnce(&mut AppContext) -> Box<dyn Widget> + 'static,
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
    viewport: (u32, u32),
    /// True when there are active timeline drivers that need continuous ticking.
    /// Checked in on_event to decide between ControlFlow::Poll and ControlFlow::Wait.
    has_active_animations: bool,
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

        // Initialize frame signal resource early so builders can use it
        let frame_signal = Signal::new(runtime.clone(), 0u64);
        let (read_frame, write_frame) = frame_signal.split();
        app.world_mut().insert_resource(FrameSignalResource {
            write_handle: write_frame,
            read_handle: read_frame.clone(),
        });

        // Insert runtime resource for reactive effects runner
        app.world_mut()
            .insert_resource(arthropod_ecs::systems::RuntimeResource(runtime.clone()));

        // Take scene from app to build widgets directly into it
        let scene = app
            .world_mut()
            .remove_resource::<Scene>()
            .expect("Scene resource missing");

        // Build widget tree
        let mut widget_ctx = WidgetContext::new(scene);
        widget_ctx.set_frame_signal(read_frame);

        let mut app_ctx = AppContext {
            runtime: runtime.clone(),
            widget_ctx,
        };

        let widget = (config.builder)(&mut app_ctx);

        // Take the context back from AppContext
        let mut widget_ctx = app_ctx.widget_ctx;
        let widget_root = widget.build(&mut widget_ctx);

        // Detect form node (for Enter submission)
        let form_node = widget_ctx.form_states().keys().next().copied();

        // Return scene to app
        let scene = widget_ctx.take_scene();
        app.world_mut().insert_resource(scene);

        // 1. Integrate widget scene into app ECS (spawn entities)
        integrate_widget_scene(&mut app, widget_root);

        // 2. Integrate widget components (LayoutStyle, Clickable, etc.) into ECS
        // This must happen AFTER spawn so entities exist to receive components
        app.integrate_widgets(&widget_ctx);

        // 3. Transfer timelines and effects (must move, not clone)
        app.integrate_timelines(&mut widget_ctx);
        app.integrate_effects(&mut widget_ctx);

        // Set initial layout constraints
        app.world_mut()
            .insert_resource(LayoutConstraintsResource(LayoutConstraints {
                max_width: Some(config.width as f32),
                max_height: Some(config.height as f32),
                min_width: None,
                min_height: None,
            }));

        // Create event dispatcher
        let dispatcher = EventDispatcher::new(form_node);

        println!("=== Widget App Started ===");
        println!("Interactions:");
        println!("  - Click on field to focus");
        println!("  - Type to enter text");
        println!("  - Tab to move to next field");
        println!("  - Enter to submit form");

        // Check if any timelines were spawned into the ECS
        let has_active_animations = {
            let f32_count = app
                .world_mut()
                .query::<&anim_graph::ecs::TimelineDriver<f32>>()
                .iter(app.world())
                .count();
            let color_count = app
                .world_mut()
                .query::<&anim_graph::ecs::TimelineDriver<Color>>()
                .iter(app.world())
                .count();
            f32_count + color_count > 0
        };

        Self {
            app,
            widget_ctx,
            dispatcher,
            viewport: (config.width, config.height),
            has_active_animations,
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        // Only poll continuously when animations are active.
        // Static UIs use Wait to avoid wasting CPU/battery.
        *control_flow = if self.has_active_animations {
            ControlFlow::Poll
        } else {
            ControlFlow::Wait
        };

        #[cfg(feature = "nova")]
        crate::experimental::ghost_replay::record_event(self.app.world_mut(), &event);

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
            Event::Window {
                event: WindowEvent::CursorMoved { position },
                ..
            } => {
                let mut mouse_pos = self
                    .app
                    .world_mut()
                    .resource_mut::<arthropod_ecs::components::MousePosition>();
                mouse_pos.0 = render_engine::Vec2::new(position.x as f32, position.y as f32);

                if let Some(window) = self.app.window() {
                    window.request_redraw();
                }
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

    fn on_update(&mut self, delta: std::time::Duration) {
        // Update time resource for animation system
        if let Some(mut time) = self
            .app
            .world_mut()
            .get_resource_mut::<anim_graph::ecs::TimeResource>()
        {
            time.set_delta(delta);
        }

        // Run update systems (layout, reactive, etc.)
        self.app.update();

        // Refresh animation activity flag — when the last timeline completes
        // and is removed, switch back to Wait mode next frame.
        self.has_active_animations = {
            let f32_count = self
                .app
                .world_mut()
                .query::<&anim_graph::ecs::TimelineDriver<f32>>()
                .iter(self.app.world())
                .count();
            let color_count = self
                .app
                .world_mut()
                .query::<&anim_graph::ecs::TimelineDriver<Color>>()
                .iter(self.app.world())
                .count();
            f32_count + color_count > 0
        };

        // Request redraw so on_redraw renders the new frame
        if let Some(window) = self.app.window() {
            window.request_redraw();
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        // Do NOT call self.app.update() here — on_update() already ran the
        // frame schedule with the correct delta. A second update would tick
        // animations twice per frame with stale/zero delta.

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

        // Don't request_redraw unconditionally — ControlFlow::Poll already
        // drives continuous updates. Unconditional redraws waste CPU/GPU
        // for static UIs with no active animations.
    }
}

impl WidgetApp {
    /// Sync text nodes in app scene with widget context values.
    fn sync_text_nodes(&mut self) {
        const TEXT_COLOR: Color = Color::rgba(0.2, 0.2, 0.2, 1.0);
        const PLACEHOLDER_COLOR: Color = Color::rgba(0.5, 0.5, 0.5, 1.0);

        // Iterate over text input states directly
        for &node_id in self.widget_ctx.text_input_states().keys() {
            // Since we share the scene, widget_node_id == app_node_id
            let app_node = node_id;

            let Some(value) = self.widget_ctx.get_text_input_value(node_id) else {
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
                if let NodeContent::Styled { ref mut style } = text_node.content {
                    if let Some(ref mut text_content) = style.text {
                        text_content.text = value.clone();
                    }
                    if !style.fills.is_empty() {
                        style.fills[0] = render_engine::Paint::Solid(color.as_vec4());
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
        for &node_id in self.widget_ctx.text_input_states().keys() {
            let app_node = node_id;

            if let Some(fill) = scene
                .get_node_mut(app_node)
                .and_then(|n| {
                    if let NodeContent::Styled { style } = &mut n.content {
                        Some(style)
                    } else {
                        None
                    }
                })
                .and_then(|style| style.fills.first_mut())
            {
                *fill = render_engine::Paint::Solid(Color::WHITE.as_vec4());
            }
        }

        // Highlight focused field
        let Some(focused_widget) = focused else {
            return;
        };

        // widget node id == app node id
        let app_node = focused_widget;

        if let Some(fill) = scene
            .get_node_mut(app_node)
            .and_then(|n| {
                if let NodeContent::Styled { style } = &mut n.content {
                    Some(style)
                } else {
                    None
                }
            })
            .and_then(|style| style.fills.first_mut())
        {
            *fill = render_engine::Paint::Solid(Color::rgba(0.7, 0.85, 1.0, 1.0).as_vec4());
        }
    }
}
