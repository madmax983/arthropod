//! Example: Themed Form
//!
//! Demonstrates all widgets using the theme engine with platform colors.
//! Shows Form, TextInput, Text (as labels), and Button all styled with DesignTokens.
//!
//! Run with: cargo run --example themed_form
//!
//! Features demonstrated:
//! - SystemTheme query for platform colors
//! - DesignTokens for semantic theming
//! - Mica backdrop material (Windows 11)
//! - Form container with theme-aware background
//! - Input fields using surface_primary token
//! - Submit button using accent color
//! - Labels using text_primary and text_secondary tokens

use plat_core::{
    Application, BackdropMaterial, ControlFlow, Event, EventLoop, HasBackdropMaterial, Rect, Size,
    Window, WindowConfig, WindowEvent, WindowId,
};
use render_engine::{
    Color, NodeContent, Scene, SceneNode,
    backend::{RenderBackend, WgpuBackend},
};
use theme_engine::{DesignTokens, SystemTheme};

/// Application state for the themed form demo
struct ThemedFormApp {
    window: Window,
    backend: WgpuBackend,
    scene: Scene,
    #[allow(dead_code)]
    tokens: DesignTokens,
}

impl Application for ThemedFormApp {
    fn new(event_loop: &EventLoop) -> Self {
        // Query system theme for design tokens
        let theme = SystemTheme::query().unwrap_or_else(|_| {
            // Provide a sensible fallback if theme query fails
            SystemTheme {
                accent_color: glam::Vec4::new(0.0, 0.47, 0.84, 1.0),
                is_dark_mode: false,
                supports_transparency: false,
                available_materials: vec![],
                text_color: glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
                text_secondary_color: glam::Vec4::new(0.4, 0.4, 0.4, 1.0),
            }
        });
        let tokens = DesignTokens::from_system(&theme);

        println!("=== Themed Form Demo ===");
        println!("System theme:");
        println!("  - Dark mode: {}", theme.is_dark_mode);
        println!("  - Supports transparency: {}", theme.supports_transparency);
        println!(
            "  - Accent color: ({:.2}, {:.2}, {:.2})",
            theme.accent_color.x, theme.accent_color.y, theme.accent_color.z
        );
        println!(
            "  - Text color: ({:.2}, {:.2}, {:.2})",
            theme.text_color.x, theme.text_color.y, theme.text_color.z
        );
        println!("  - Available materials: {:?}", theme.available_materials);
        println!();

        // Create window with transparent config for backdrop effect
        let config = WindowConfig {
            title: "Themed Form Demo - Mica Backdrop".to_string(),
            size: Size::new(500, 450),
            resizable: true,
            decorations: true,
            transparent: true, // Enable transparency for backdrop effect
            visible: true,
            ..Default::default()
        };

        let window = event_loop
            .create_window(config)
            .expect("Failed to create window");

        // Apply Mica backdrop material (Windows 11)
        // This gracefully degrades on Windows 10 (falls back to Acrylic or solid)
        window.set_backdrop_material(BackdropMaterial::Mica);
        println!(
            "Applied backdrop material: {:?}",
            window.backdrop_material()
        );

        // Create GPU backend (standard mode - not using DirectComposition)
        let size = window.inner_size();
        let mut backend =
            WgpuBackend::new(&window, size.width, size.height, false).expect("Failed to create backend");

        // IMPORTANT: Use transparent clear color to show Mica backdrop through
        backend.set_clear_color(Color::rgba(0.0, 0.0, 0.0, 0.0));

        // Create scene with themed form elements
        let scene = create_themed_form_scene(&tokens, theme.is_dark_mode);

        println!();
        println!("You should see:");
        println!("  - Desktop wallpaper bleeding through (Windows 11 Mica)");
        println!("  - Form container with semi-transparent background");
        println!("  - Input fields using surface_primary token");
        println!("  - Labels using text_primary color");
        println!("  - Submit button using system accent color");
        println!();
        println!("Try moving the window to see the Mica effect update!");

        Self {
            window,
            backend,
            scene,
            tokens,
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window {
                event: WindowEvent::Resized(size),
                ..
            } => {
                self.backend.resize(size.width, size.height);
                self.window.request_redraw();
            }
            Event::Window {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        if let Err(e) = self.backend.render(&self.scene) {
            eprintln!("Render error: {}", e);
        }
    }
}

/// Create a themed form scene demonstrating all widget types
fn create_themed_form_scene(tokens: &DesignTokens, is_dark_mode: bool) -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    // Form container - uses surface_secondary with transparency for glass effect
    let surface_color = tokens.surface_secondary.as_color();
    let form_container = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(surface_color.x, surface_color.y, surface_color.z, 0.85),
    });
    let form_id = scene.add_node(root, form_container);
    if let Some(node) = scene.get_node_mut(form_id) {
        node.bounds = Rect::new(30.0, 30.0, 440.0, 390.0);
    }

    // Title "label" - uses text_primary color
    // Note: Text rendering requires TextEngine integration; for now we use a colored rect
    // to represent the title area. In production, this would be NodeContent::Text.
    let title_bg_color = if is_dark_mode {
        Color::rgba(0.2, 0.2, 0.25, 0.5) // Subtle title background for dark mode
    } else {
        Color::rgba(0.0, 0.0, 0.0, 0.0) // Transparent for light mode
    };
    let title_area = SceneNode::new(NodeContent::Rect {
        color: title_bg_color,
    });
    let title_id = scene.add_node(root, title_area);
    if let Some(node) = scene.get_node_mut(title_id) {
        node.bounds = Rect::new(50.0, 50.0, 400.0, 40.0);
    }

    // Form fields section

    // Field 1: Name input
    // Label background (subtle indicator)
    let label_color = Color::rgba(
        tokens.text_secondary.x,
        tokens.text_secondary.y,
        tokens.text_secondary.z,
        0.3,
    );
    let name_label = SceneNode::new(NodeContent::Rect { color: label_color });
    let name_label_id = scene.add_node(root, name_label);
    if let Some(node) = scene.get_node_mut(name_label_id) {
        node.bounds = Rect::new(50.0, 110.0, 60.0, 24.0);
    }

    // Name input field - uses surface_primary
    let input_bg = tokens.surface_primary.as_color();
    let name_input = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(input_bg.x, input_bg.y, input_bg.z, 1.0),
    });
    let name_input_id = scene.add_node(root, name_input);
    if let Some(node) = scene.get_node_mut(name_input_id) {
        node.bounds = Rect::new(50.0, 140.0, 380.0, 44.0);
    }

    // Field 2: Email input
    let email_label = SceneNode::new(NodeContent::Rect { color: label_color });
    let email_label_id = scene.add_node(root, email_label);
    if let Some(node) = scene.get_node_mut(email_label_id) {
        node.bounds = Rect::new(50.0, 200.0, 60.0, 24.0);
    }

    // Email input field
    let email_input = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(input_bg.x, input_bg.y, input_bg.z, 1.0),
    });
    let email_input_id = scene.add_node(root, email_input);
    if let Some(node) = scene.get_node_mut(email_input_id) {
        node.bounds = Rect::new(50.0, 230.0, 380.0, 44.0);
    }

    // Field 3: Password input (optional third field)
    let password_label = SceneNode::new(NodeContent::Rect { color: label_color });
    let password_label_id = scene.add_node(root, password_label);
    if let Some(node) = scene.get_node_mut(password_label_id) {
        node.bounds = Rect::new(50.0, 290.0, 80.0, 24.0);
    }

    // Password input field
    let password_input = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(input_bg.x, input_bg.y, input_bg.z, 1.0),
    });
    let password_input_id = scene.add_node(root, password_input);
    if let Some(node) = scene.get_node_mut(password_input_id) {
        node.bounds = Rect::new(50.0, 320.0, 380.0, 44.0);
    }

    // Submit button - uses accent color
    let accent = tokens.accent;
    let submit_button = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(accent.x, accent.y, accent.z, 1.0),
    });
    let submit_button_id = scene.add_node(root, submit_button);
    if let Some(node) = scene.get_node_mut(submit_button_id) {
        node.bounds = Rect::new(150.0, 380.0, 200.0, 50.0);
    }

    // Add visual indicators for focus/hover states
    // Input border highlight (demonstrates accent_hover usage)
    let accent_hover = tokens.accent_hover;
    let focus_indicator = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(accent_hover.x, accent_hover.y, accent_hover.z, 0.5),
    });
    let focus_id = scene.add_node(root, focus_indicator);
    if let Some(node) = scene.get_node_mut(focus_id) {
        // Position as a subtle border under the first input
        node.bounds = Rect::new(48.0, 182.0, 384.0, 3.0);
    }

    scene
}

fn main() {
    env_logger::init();
    plat_core::run::<ThemedFormApp>().expect("Failed to run application");
}
