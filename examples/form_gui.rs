//! Form GUI Example - Complete form with real window, keyboard, and mouse input
//!
//! Demonstrates:
//! - Full GUI window with wgpu rendering
//! - Keyboard input reaching text fields
//! - Mouse click for focus management
//! - Form validation with visual feedback
//! - Tab navigation between fields
//! - Enter to submit
//!
//! Run with: cargo run --example form_gui

use arthropod::prelude::*;
use arthropod_mcp::{connect_to_mcp_server, AppConnection};
use flux_state::Signal;
use plat_core::{ElementState, Key, MouseButton};
use std::sync::Arc;
use widget_core::{Form, TextInput, Widget, WidgetContext};

// ============================================================================
// Input Utilities (Application Layer - glue between plat-core and widget-core)
// ============================================================================

/// Convert plat-core Key to character
fn key_to_char(key: &Key, shift: bool) -> Option<char> {
    match key {
        // Letters
        Key::A => Some(if shift { 'A' } else { 'a' }),
        Key::B => Some(if shift { 'B' } else { 'b' }),
        Key::C => Some(if shift { 'C' } else { 'c' }),
        Key::D => Some(if shift { 'D' } else { 'd' }),
        Key::E => Some(if shift { 'E' } else { 'e' }),
        Key::F => Some(if shift { 'F' } else { 'f' }),
        Key::G => Some(if shift { 'G' } else { 'g' }),
        Key::H => Some(if shift { 'H' } else { 'h' }),
        Key::I => Some(if shift { 'I' } else { 'i' }),
        Key::J => Some(if shift { 'J' } else { 'j' }),
        Key::K => Some(if shift { 'K' } else { 'k' }),
        Key::L => Some(if shift { 'L' } else { 'l' }),
        Key::M => Some(if shift { 'M' } else { 'm' }),
        Key::N => Some(if shift { 'N' } else { 'n' }),
        Key::O => Some(if shift { 'O' } else { 'o' }),
        Key::P => Some(if shift { 'P' } else { 'p' }),
        Key::Q => Some(if shift { 'Q' } else { 'q' }),
        Key::R => Some(if shift { 'R' } else { 'r' }),
        Key::S => Some(if shift { 'S' } else { 's' }),
        Key::T => Some(if shift { 'T' } else { 't' }),
        Key::U => Some(if shift { 'U' } else { 'u' }),
        Key::V => Some(if shift { 'V' } else { 'v' }),
        Key::W => Some(if shift { 'W' } else { 'w' }),
        Key::X => Some(if shift { 'X' } else { 'x' }),
        Key::Y => Some(if shift { 'Y' } else { 'y' }),
        Key::Z => Some(if shift { 'Z' } else { 'z' }),
        // Numbers (with shift symbols)
        Key::Key0 => Some(if shift { ')' } else { '0' }),
        Key::Key1 => Some(if shift { '!' } else { '1' }),
        Key::Key2 => Some(if shift { '@' } else { '2' }),
        Key::Key3 => Some(if shift { '#' } else { '3' }),
        Key::Key4 => Some(if shift { '$' } else { '4' }),
        Key::Key5 => Some(if shift { '%' } else { '5' }),
        Key::Key6 => Some(if shift { '^' } else { '6' }),
        Key::Key7 => Some(if shift { '&' } else { '7' }),
        Key::Key8 => Some(if shift { '*' } else { '8' }),
        Key::Key9 => Some(if shift { '(' } else { '9' }),
        // Space
        Key::Space => Some(' '),
        // Punctuation
        Key::Period => Some(if shift { '>' } else { '.' }),
        Key::Comma => Some(if shift { '<' } else { ',' }),
        Key::Minus => Some(if shift { '_' } else { '-' }),
        Key::Equal => Some(if shift { '+' } else { '=' }),
        Key::Semicolon => Some(if shift { ':' } else { ';' }),
        Key::Quote => Some(if shift { '"' } else { '\'' }),
        Key::Slash => Some(if shift { '?' } else { '/' }),
        Key::Backslash => Some(if shift { '|' } else { '\\' }),
        Key::BracketLeft => Some(if shift { '{' } else { '[' }),
        Key::BracketRight => Some(if shift { '}' } else { ']' }),
        Key::Backtick => Some(if shift { '~' } else { '`' }),
        // Non-character keys
        _ => None,
    }
}

/// Hit test: find topmost visible node at position
fn hit_test(scene: &Scene, x: f32, y: f32) -> Option<NodeId> {
    let mut result = None;

    // Iterate through all nodes, keeping track of the last one that contains the point
    for (id, node) in scene.nodes() {
        if node.visible && bounds_contains(&node.bounds, x, y) {
            result = Some(id);
        }
    }

    result
}

/// Check if a rectangle contains a point
fn bounds_contains(bounds: &Rect, x: f32, y: f32) -> bool {
    x >= bounds.x
        && x < bounds.x + bounds.width
        && y >= bounds.y
        && y < bounds.y + bounds.height
}

// ============================================================================
// Main Application
// ============================================================================

struct FormGuiApp {
    // App encapsulates Window, WgpuBackend, Scene, Runtime
    app: arthropod::App,

    // Form tracking
    form_node: NodeId,
    app_form_node: NodeId, // NodeId in app's scene

    // NodeId mapping: widget_ctx NodeId -> app Scene NodeId
    node_id_map: std::collections::HashMap<NodeId, NodeId>,

    // Input state
    mouse_pos: (f32, f32),
    shift_held: bool,

    // Widget context (for input handling and widget state)
    widget_ctx: WidgetContext,

    // MCP connection for remote debugging (optional)
    mcp_connection: Option<AppConnection>,

    // Frame counter for periodic MCP updates
    mcp_update_counter: u32,
}

impl Application for FormGuiApp {
    fn new(event_loop: &EventLoop) -> Self {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::INFO.into()),
            )
            .init();

        let config = WindowConfig {
            title: "Arthropod Form GUI Demo".to_string(),
            size: Size {
                width: 500,
                height: 400,
            },
            resizable: true,
            decorations: true,
            visible: true,
            ..Default::default()
        };

        // Create app with AppBuilder - handles everything automatically
        let mut app = arthropod::AppBuilder::new()
            .with_window_config(config)
            .build(event_loop)
            .expect("Failed to create app");

        // Get runtime from app
        let runtime = app.runtime().clone();

        // Build form once (no rebuilds - use reactive updates!)
        let (form_node, widget_ctx) = Self::build_form(&runtime);

        // Integrate widget scene into app's ECS Scene and get NodeId mapping
        let (app_form_node, node_id_map) =
            Self::integrate_scene_into_app(&mut app, &widget_ctx, form_node);

        println!("✓ Integrated {} nodes into app scene", node_id_map.len());
        println!("✓ Form root node ID in app: {:?}", app_form_node);

        println!("=== Form GUI Started ===");
        println!("Interactions:");
        println!("  - Click on field to focus (field will turn light blue)");
        println!("  - Type to enter text (shown in console below)");
        println!("  - Tab to move to next field");
        println!("  - Backspace/Delete to remove characters");
        println!("  - Arrow keys to move cursor");
        println!("  - Enter to submit form (when valid)");
        println!("\nValidation:");
        println!("  - Name: Required");
        println!("  - Email: Required, must contain @ and .");
        println!("  - Password: Required, minimum 8 characters");
        println!("\nNOTE: Text glyphs render but atlas not yet rasterized - typed text shown in console\n");

        // Try to connect to MCP server (optional - don't fail if server isn't running)
        let mcp_connection = connect_to_mcp_server("form_gui").ok();
        if mcp_connection.is_some() {
            println!("✓ Connected to MCP server for remote debugging");
        } else {
            println!("⚠ MCP server not available (run 'cargo run --bin arthropod-mcp' to enable remote debugging)");
        }

        Self {
            app,
            form_node,
            app_form_node,
            node_id_map,
            mouse_pos: (0.0, 0.0),
            shift_held: false,
            widget_ctx,
            mcp_connection,
            mcp_update_counter: 0,
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        println!("EVENT: {:?}", event);
        match event {
            Event::Window {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::Window {
                event: WindowEvent::KeyboardInput(keyboard_input),
                ..
            } => {
                if keyboard_input.state == ElementState::Pressed {
                    self.handle_key_press(&keyboard_input.key);
                    // Request redraw to update visuals
                    if let Some(window) = self.app.window() {
                        window.request_redraw();
                    }
                } else if keyboard_input.state == ElementState::Released
                    && matches!(keyboard_input.key, Key::Shift)
                {
                    self.shift_held = false;
                }
            }
            Event::Window {
                event: WindowEvent::CursorMoved { position },
                ..
            } => {
                self.mouse_pos = (position.x as f32, position.y as f32);
            }
            Event::Window {
                event: WindowEvent::MouseInput(mouse_input),
                ..
            } => {
                if mouse_input.button == MouseButton::Left
                    && mouse_input.state == ElementState::Pressed
                {
                    println!("🖱️  Mouse click at ({}, {})", mouse_input.position.x, mouse_input.position.y);
                    self.handle_click();
                    // Request redraw to update visuals
                    if let Some(window) = self.app.window() {
                        window.request_redraw();
                    }
                }
            }
            Event::Window {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Backend resize handled automatically by app
                println!("Window resized to {}x{}", size.width, size.height);
                // Request redraw
                if let Some(window) = self.app.window() {
                    window.request_redraw();
                }
                // TODO: Re-layout form with new window size
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        println!("🔄 on_redraw called");

        // Render scene directly (not through ECS, since our widgets are pure Scene nodes)
        // Take ownership of backend temporarily to avoid borrow conflicts
        let mut backend = self.app.world_mut().remove_resource::<WgpuBackend>();

        if let Some(ref mut backend) = backend {
            // Now we can borrow scene immutably
            let scene = self.app.world().resource::<Scene>();

            println!("  Calling backend.render(scene)...");
            if let Err(e) = backend.render(&scene) {
                eprintln!("  ❌ Render error: {}", e);
            } else {
                println!("  ✅ Render complete");
            }
        }

        // Put backend back
        if let Some(backend) = backend {
            self.app.world_mut().insert_resource(backend);
        }

        // Send scene updates to MCP server periodically
        // Using fast updates (every frame) for debugging
        self.mcp_update_counter += 1;
        if self.mcp_update_counter >= 1 {
            self.mcp_update_counter = 0;
            if let Some(connection) = &mut self.mcp_connection {
                let scene = self.app.world().resource::<Scene>();
                if let Ok(scene_json) = scene.serialize_to_json() {
                    let _ = connection.send_scene_update(scene_json);
                }
            }
        }
    }
}

impl FormGuiApp {
    /// Build the form widgets (called once - no rebuild!)
    fn build_form(runtime: &Arc<flux_state::Runtime>) -> (NodeId, WidgetContext) {
        // Create signals for form fields
        let name = Signal::new(runtime.clone(), String::new());
        let email = Signal::new(runtime.clone(), String::new());
        let password = Signal::new(runtime.clone(), String::new());

        // Create form with validators
        let form = Form::new((
            ("name", TextInput::new(name).placeholder("Name")),
            (
                "email",
                TextInput::new(email)
                    .placeholder("email@example.com")
                    .validator(|s| {
                        if s.contains('@') && s.contains('.') {
                            Ok(())
                        } else {
                            Err("Invalid email".into())
                        }
                    }),
            ),
            (
                "password",
                TextInput::new(password)
                    .placeholder("Password (8+ chars)")
                    .validator(|s| {
                        if s.len() >= 8 {
                            Ok(())
                        } else {
                            Err("Password too short".into())
                        }
                    }),
            ),
        ))
        .gap(16.0)
        .padding(20.0)
        .on_submit(|data| {
            println!("\n✅ Form submitted successfully!");
            println!("Data:");
            for (field, value) in data {
                println!("  {}: {}", field, value);
            }
            Ok(())
        });

        // Build form using WidgetContext
        let mut ctx = WidgetContext::new_test();
        let form_node = form.build(&mut ctx);

        // Add submit button to form
        use widget_core::Button;
        let submit_button = Button::new("Submit")
            .primary()
            .padding(16.0)
            .on_click({
                move || {
                    println!("Submit button clicked!");
                }
            });

        let button_node = submit_button.build(&mut ctx);
        // Re-parent button from root to form
        ctx.reparent_node(button_node, ctx.root(), form_node);

        // Assign bounds to form fields for hit testing (hardcoded layout for MVP)
        Self::layout_form(&mut ctx, form_node);

        (form_node, ctx)
    }

    /// Integrate widget scene into app's ECS Scene and spawn Renderable entities
    /// Returns (app_form_node, node_id_mapping)
    fn integrate_scene_into_app(
        app: &mut arthropod::App,
        widget_ctx: &WidgetContext,
        form_node: NodeId,
    ) -> (NodeId, std::collections::HashMap<NodeId, NodeId>) {
        use std::collections::HashMap;

        let widget_scene = widget_ctx.scene();
        let mut node_id_map = HashMap::new();

        // Recursively copy a node and all its descendants
        fn copy_node_recursive(
            widget_scene: &Scene,
            app_scene: &mut Scene,
            widget_node_id: NodeId,
            app_parent_id: NodeId,
            node_id_map: &mut HashMap<NodeId, NodeId>,
        ) -> NodeId {
            // Get widget node data
            let widget_node = widget_scene.get_node(widget_node_id).unwrap();

            // Create new node in app scene
            let mut new_node = SceneNode::new(widget_node.content.clone());
            new_node.bounds = widget_node.bounds;
            new_node.visible = widget_node.visible;
            new_node.opacity = widget_node.opacity;
            let app_node_id = app_scene.add_node(app_parent_id, new_node);

            // Track mapping
            node_id_map.insert(widget_node_id, app_node_id);

            // Recursively copy children
            let children = widget_node.children.clone();
            for child_id in children {
                copy_node_recursive(widget_scene, app_scene, child_id, app_node_id, node_id_map);
            }

            app_node_id
        }

        // Copy the entire form subtree
        let mut scene = app.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        let app_form_node = copy_node_recursive(
            widget_scene,
            &mut scene,
            form_node,
            root,
            &mut node_id_map,
        );

        // Now spawn Renderable entities for all copied nodes
        drop(scene);
        for &app_node_id in node_id_map.values() {
            app.spawn(app_node_id).insert(Renderable);
        }

        (app_form_node, node_id_map)
    }

    /// Hardcoded layout for MVP (Phase 3 will have proper layout engine)
    fn layout_form(ctx: &mut WidgetContext, form_node: NodeId) {
        let scene = ctx.scene_mut();

        // Get form's children (text inputs + submit button)
        if let Some(form) = scene.get_node(form_node) {
            let children: Vec<NodeId> = form.children.clone();

            // Layout each field vertically with gap
            let mut y = 50.0;
            let x = 50.0;
            let width = 400.0;
            let field_height = 40.0;
            let button_height = 50.0;
            let gap = 16.0;
            let padding = 8.0;

            for (i, child_id) in children.iter().enumerate() {
                // Last child is submit button - give it more height and spacing
                let is_button = i == children.len() - 1;
                let height = if is_button { button_height } else { field_height };

                // Add extra gap before button
                if is_button {
                    y += 8.0;
                }

                // Set bounds on container
                if let Some(child) = scene.get_node_mut(*child_id) {
                    child.bounds = Rect {
                        x,
                        y,
                        width,
                        height,
                    };

                    // Also set bounds on text nodes inside container
                    let text_children: Vec<NodeId> = child.children.clone();
                    let _ = child; // Drop borrow before next get_node_mut

                    for text_id in text_children {
                        if let Some(text_node) = scene.get_node_mut(text_id) {
                            // Position text inside container with padding
                            text_node.bounds = Rect {
                                x: x + padding,
                                y: y + padding,
                                width: width - 2.0 * padding,
                                height: height - 2.0 * padding,
                            };
                        }
                    }
                }
                y += height + gap;
            }
        }
    }

    fn handle_key_press(&mut self, key: &Key) {
        match key {
            Key::Shift => self.shift_held = true,
            Key::Backspace => {
                self.widget_ctx.send_backspace();
                self.update_text_nodes();
            }
            Key::Delete => {
                self.widget_ctx.send_delete();
                self.update_text_nodes();
            }
            Key::Left => self.widget_ctx.send_key_left(),
            Key::Right => self.widget_ctx.send_key_right(),
            Key::Enter => {
                // Submit form
                println!("\nAttempting form submission...");
                self.widget_ctx.trigger_submit(self.form_node);

                if self.widget_ctx.has_submit_error(self.form_node) {
                    println!(
                        "Submit error: {}",
                        self.widget_ctx
                            .get_submit_error(self.form_node)
                            .unwrap()
                    );
                } else if !self.widget_ctx.is_form_valid(self.form_node) {
                    println!("Cannot submit: Form has validation errors");
                    let errors = self.widget_ctx.get_form_field_errors(self.form_node);
                    for (field, error) in errors {
                        println!("  {}: {}", field, error);
                    }
                }
            }
            Key::Tab => self.focus_next(),
            _ => {
                // Try to convert to character
                if let Some(c) = key_to_char(key, self.shift_held) {
                    self.widget_ctx.send_char(c);
                    self.update_text_nodes();

                    if let Some(focused) = self.widget_ctx.focused_node() {
                        if let Some(value) = self.widget_ctx.get_text_input_value(focused) {
                            println!("Input updated: '{}'", value);
                        }
                    }
                }
            }
        }
    }

    /// Update text nodes in app scene to match widget context values
    /// This is the reactive update - no rebuild needed!
    fn update_text_nodes(&mut self) {
        use render_engine::{ShapedGlyphData, ShapedTextData};

        const FONT_SIZE: f32 = 16.0;
        const TEXT_COLOR: Color = Color::rgba(0.2, 0.2, 0.2, 1.0); // Dark gray text
        const PLACEHOLDER_COLOR: Color = Color::rgba(0.5, 0.5, 0.5, 1.0); // Gray placeholder

        // For each text input, update its text CHILD node (not the input container!)
        for (&widget_input_node, &app_input_node) in &self.node_id_map {
            if self.widget_ctx.is_text_input(widget_input_node) {
                // Get the current text value from widget context
                if let Some(value) = self.widget_ctx.get_text_input_value(widget_input_node) {
                    // Find the text child node in the app scene
                    let mut app_scene = self.app.world_mut().resource_mut::<Scene>();

                    // The text input structure is: Rect container -> Text child
                    // We need to update the text child, not the container
                    if let Some(input_node) = app_scene.get_node(app_input_node) {
                        // Get the first child (should be the text node)
                        if let Some(&text_child_id) = input_node.children.first() {
                            // Store raw text - it will be shaped during rendering by the backend's TextRenderer
                            // This avoids FontSystem/CacheKey mismatch issues between different TextEngine instances
                            if let Some(text_node) = app_scene.get_node_mut(text_child_id) {
                                let color = if value.is_empty() { PLACEHOLDER_COLOR } else { TEXT_COLOR };
                                text_node.content = NodeContent::RawText {
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

    /// Print current form state for debugging (since we don't have text rendering yet)
    fn print_form_state(&self) {
        println!("\n┌─ Current Form State ─────────────────┐");

        let field_nodes: Vec<NodeId> = {
            let scene = self.widget_ctx.scene();
            if let Some(form) = scene.get_node(self.form_node) {
                form.children.clone()
            } else {
                vec![]
            }
        };

        let field_names = ["Name", "Email", "Password"];
        for (i, &field_node) in field_nodes.iter().enumerate() {
            let value = self
                .widget_ctx
                .get_text_input_value(field_node)
                .unwrap_or_default();
            let is_focused = self.widget_ctx.is_focused(field_node);
            let focus_marker = if is_focused { "→ " } else { "  " };

            println!(
                "│ {}{:<10} │ {:<20} │",
                focus_marker,
                field_names.get(i).unwrap_or(&"Unknown"),
                if value.is_empty() {
                    "(empty)".to_string()
                } else {
                    value
                }
            );
        }

        println!("└──────────────────────────────────────┘\n");
    }

    fn handle_click(&mut self) {
        let (x, y) = self.mouse_pos;
        println!("Click at ({}, {}) - testing against app scene", x, y);

        // Hit test against APP scene (not widget scene) since that's what's rendered
        let app_scene = self.app.world().resource::<Scene>();
        if let Some(mut app_node_id) = hit_test(&app_scene, x, y) {
            println!("  Hit app node: {:?}", app_node_id);

            // Check if we hit a text node - if so, get its parent (the input container)
            if let Some(hit_node) = app_scene.get_node(app_node_id) {
                if matches!(hit_node.content, NodeContent::Text { .. }) {
                    // We clicked on text - get the parent container
                    if let Some(parent_id) = app_scene.find_parent(app_node_id) {
                        println!("  Hit text node, using parent: {:?}", parent_id);
                        app_node_id = parent_id;
                    }
                }
            }

            // Find the corresponding widget node ID
            let widget_node_id = self
                .node_id_map
                .iter()
                .find(|&(_, &app_id)| app_id == app_node_id)
                .map(|(&widget_id, _)| widget_id);

            if let Some(widget_node_id) = widget_node_id {
                println!("  Mapped to widget node: {:?}", widget_node_id);

                if self.widget_ctx.is_text_input(widget_node_id) {
                    // Clear previous focus visual
                    self.update_focus_visual(None);

                    // Set new focus
                    self.widget_ctx.focus_node(widget_node_id);
                    println!("  Focused text input");

                    // Update visual for new focus
                    self.update_focus_visual(Some(widget_node_id));
                } else if self.widget_ctx.is_clickable(widget_node_id) {
                    self.widget_ctx.trigger_click(widget_node_id);
                    println!("  Triggered clickable");
                }
            } else {
                println!("  No widget mapping found for app node");
            }
        } else {
            println!("  No node hit at this position");
        }
    }

    /// Update visual feedback for focused field (change color in app scene)
    fn update_focus_visual(&mut self, focused_widget_node: Option<NodeId>) {
        let mut scene = self.app.world_mut().resource_mut::<Scene>();

        // Reset all TEXT INPUT field colors to white (not buttons!)
        for (&widget_node_id, &app_node_id) in &self.node_id_map {
            // Only update text inputs, not buttons
            if self.widget_ctx.is_text_input(widget_node_id) {
                if let Some(node) = scene.get_node_mut(app_node_id) {
                    if matches!(node.content, NodeContent::Rect { .. }) {
                        node.content = NodeContent::Rect {
                            color: Color::WHITE,
                        };
                    }
                }
            }
        }

        // Highlight focused field in light blue
        if let Some(widget_node_id) = focused_widget_node {
            if let Some(&app_node_id) = self.node_id_map.get(&widget_node_id) {
                if let Some(node) = scene.get_node_mut(app_node_id) {
                    node.content = NodeContent::Rect {
                        color: Color::rgba(0.7, 0.85, 1.0, 1.0), // Light blue
                    };
                }
            }
        }
    }

    fn focus_next(&mut self) {
        // Get form children (field nodes)
        let field_nodes: Vec<NodeId> = {
            let scene = self.widget_ctx.scene();
            if let Some(form) = scene.get_node(self.form_node) {
                form.children.clone()
            } else {
                return;
            }
        };

        // Find current focused field index
        let current_focused = self.widget_ctx.focused_node();
        let current_index =
            current_focused.and_then(|focused| field_nodes.iter().position(|&id| id == focused));

        // Move to next field (wrap around)
        let next_index = match current_index {
            Some(idx) => (idx + 1) % field_nodes.len(),
            None => 0,
        };

        if next_index < field_nodes.len() {
            // Clear previous focus visual
            self.update_focus_visual(None);

            // Set new focus
            self.widget_ctx.focus_node(field_nodes[next_index]);
            println!("Focused field {}", next_index);

            // Update visual for new focus
            self.update_focus_visual(Some(field_nodes[next_index]));

            // Print form state
            self.print_form_state();
        }
    }
}

fn main() {
    plat_core::run::<FormGuiApp>().expect("Failed to run");
}
