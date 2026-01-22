//! Example: Native Materials (Mica/Acrylic)
//!
//! Demonstrates platform-native backdrop materials on Windows 11.
//!
//! ## What Works
//!
//! - **Title bar Mica**: The window title bar shows the Mica effect, blending
//!   with your desktop wallpaper colors. This is the standard Windows 11 behavior.
//!
//! ## Current Limitations
//!
//! - **Client area Mica**: Full client-area transparency with Mica requires
//!   DirectComposition swap chains, which wgpu doesn't currently expose.
//!   The GPU-rendered content appears opaque even with transparent clear color.
//!
//! This is a known limitation of wgpu + DWM integration. Native Windows apps
//! using WinUI 3 or UWP achieve full Mica through special compositor APIs.
//!
//! Run with: cargo run --example native_materials
//!
//! Note: Mica requires Windows 11. On Windows 10, it falls back gracefully.

use plat_core::{
    Application, BackdropMaterial, ControlFlow, Event, EventLoop, HasBackdropMaterial, Rect, Size,
    Window, WindowConfig, WindowEvent, WindowId,
};
use render_engine::{
    Color, NodeContent, Scene, SceneNode,
    backend::{RenderBackend, WgpuBackend},
};
use theme_engine::{DesignTokens, SystemTheme};

/// Application state for the native materials demo
struct MaterialApp {
    window: Window,
    backend: WgpuBackend,
    scene: Scene,
    #[allow(dead_code)]
    tokens: DesignTokens,
}

impl Application for MaterialApp {
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

        println!("=== Native Materials Demo ===");
        println!("System theme:");
        println!("  - Dark mode: {}", theme.is_dark_mode);
        println!("  - Supports transparency: {}", theme.supports_transparency);
        println!(
            "  - Accent color: ({:.2}, {:.2}, {:.2})",
            theme.accent_color.x, theme.accent_color.y, theme.accent_color.z
        );
        println!("  - Available materials: {:?}", theme.available_materials);
        println!();

        // Create window with transparent config for backdrop effect
        let config = WindowConfig {
            title: "Native Materials Demo - Mica Backdrop".to_string(),
            size: Size::new(800, 600),
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

        // Create GPU backend
        let size = window.inner_size();
        let mut backend =
            WgpuBackend::new(&window, size.width, size.height).expect("Failed to create backend");

        // Note: We set a transparent clear color, but full client-area Mica
        // requires DirectComposition swap chains. With standard wgpu swap chains,
        // the alpha channel isn't composited with DWM's backdrop.
        // Title bar Mica still works via DwmSetWindowAttribute.
        backend.set_clear_color(Color::rgba(0.0, 0.0, 0.0, 1.0)); // Solid black for now

        // Create scene with semi-transparent cards to demonstrate Mica effect
        let scene = create_demo_scene(&tokens);

        println!();
        println!("You should see:");
        println!("  - Title bar with Mica effect (blends with desktop wallpaper)");
        println!("  - Theme-aware colored cards using DesignTokens");
        println!("  - Accent-colored card using system accent color");
        println!();
        println!("Note: Full client-area Mica requires DirectComposition APIs");
        println!("      which wgpu doesn't currently expose. The title bar Mica");
        println!("      demonstrates the DWM integration is working.");

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

/// Create a demo scene with semi-transparent cards
fn create_demo_scene(tokens: &DesignTokens) -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    // Card 1: Elevated surface with semi-transparency (top-left)
    // Uses theme's elevated surface color with reduced alpha
    let elevated_color = tokens.surface_elevated.as_color();
    let card1 = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(elevated_color.x, elevated_color.y, elevated_color.z, 0.85),
    });
    let card1_id = scene.add_node(root, card1);
    if let Some(node) = scene.get_node_mut(card1_id) {
        node.bounds = Rect::new(50.0, 50.0, 300.0, 200.0);
    }

    // Card 2: Accent colored card (top-right)
    // Uses system accent color with slight transparency
    let accent = tokens.accent;
    let card2 = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(accent.x, accent.y, accent.z, 0.9),
    });
    let card2_id = scene.add_node(root, card2);
    if let Some(node) = scene.get_node_mut(card2_id) {
        node.bounds = Rect::new(400.0, 50.0, 300.0, 200.0);
    }

    // Card 3: Secondary surface (bottom, spanning width)
    // More transparent to show off the Mica effect
    let secondary = tokens.surface_secondary.as_color();
    let card3 = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(secondary.x, secondary.y, secondary.z, 0.75),
    });
    let card3_id = scene.add_node(root, card3);
    if let Some(node) = scene.get_node_mut(card3_id) {
        node.bounds = Rect::new(50.0, 300.0, 650.0, 250.0);
    }

    // Card 4: Very transparent card to really show the backdrop
    let card4 = SceneNode::new(NodeContent::Rect {
        color: Color::rgba(1.0, 1.0, 1.0, 0.3),
    });
    let card4_id = scene.add_node(root, card4);
    if let Some(node) = scene.get_node_mut(card4_id) {
        node.bounds = Rect::new(250.0, 150.0, 300.0, 200.0);
    }

    scene
}

fn main() {
    env_logger::init();
    plat_core::run::<MaterialApp>().expect("Failed to run application");
}
