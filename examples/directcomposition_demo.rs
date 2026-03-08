//! Example: DirectComposition with Selective Transparency
//!
//! Demonstrates per-region backdrop materials using DirectComposition.
//! Shows a Mica sidebar with solid content area.
//!
//! ## How It Works
//!
//! - **DirectComposition**: Creates a visual tree with backdrop materials
//! - **Composition Mode**: wgpu renders with premultiplied alpha for proper blending
//! - **Layer-based API**: Explicit layers with materials for different regions
//!
//! ## What You'll See
//!
//! - Semi-transparent sidebar region (Mica effect on Windows 11)
//! - Solid opaque content area
//! - Desktop wallpaper visible through the sidebar only
//!
//! Run with: cargo run --example directcomposition_demo
//!
//! Note: Requires Windows 10 1803+ for DirectComposition.
//! Full Mica effect requires Windows 11.

use plat_core::{
    Application, BackdropMaterial, Compositor, ControlFlow, Event, EventLoop, Rect, Size, Window,
    WindowConfig, WindowEvent, WindowId,
};
use render_engine::{Color, NodeContent, Scene, SceneNode, backend::WgpuBackend};

const SIDEBAR_WIDTH: f32 = 250.0;

/// Application state for the DirectComposition demo
struct CompositionApp {
    // Backend must be dropped before window to avoid use-after-free
    backend: WgpuBackend,
    #[allow(dead_code)]
    window: Window,
    scene: Scene,
    #[allow(dead_code)]
    compositor: Option<Compositor>,
}

impl Application for CompositionApp {
    fn new(event_loop: &EventLoop) -> Self {
        println!("=== DirectComposition Demo ===");
        println!("Selective transparency: Mica sidebar + solid content");
        println!();

        // Window config for wgpu-managed DirectComposition
        let config = WindowConfig {
            title: "DirectComposition Demo - wgpu Built-in Transparency".to_string(),
            size: Size::new(1000, 700),
            composition_mode: false, // FALSE: wgpu handles DirectComposition via DxgiFromVisual
            transparent: true,       // TRUE: enables transparent window
            visible: true,
            ..Default::default()
        };

        let window = event_loop
            .create_window(config)
            .expect("Failed to create window");

        // Apply Mica backdrop to window (this is the base layer)
        window.set_backdrop_material(BackdropMaterial::Mica);

        // Create wgpu backend in composition mode for proper alpha blending
        let size = window.inner_size();
        // SAFETY: Safe because backend is dropped before window (struct field order)
        let mut backend = unsafe { WgpuBackend::new(&window, size.width, size.height, true) }
            .expect("Failed to create backend");

        // Use transparent clear color so backdrop shows through
        backend.set_clear_color(Color::rgba(0.0, 0.0, 0.0, 0.0));

        // NOTE: wgpu 28.0's DxgiFromVisual handles DirectComposition automatically.
        // The Mica backdrop is applied via window.set_backdrop_material() above.
        // Manual Compositor would conflict with wgpu's built-in DirectComposition,
        // causing DCOMPOSITION_ERROR_WINDOW_ALREADY_COMPOSED.
        //
        // Instead, we demonstrate transparency by rendering:
        // - Semi-transparent rectangles in sidebar (shows Mica through)
        // - Opaque rectangles in content area (blocks Mica)
        let compositor = None;

        println!("Using wgpu's built-in DirectComposition (DxgiFromVisual)");
        println!("Window backdrop: Mica (shows through transparent areas)");

        // Create demo scene
        let scene = create_demo_scene(size.width, size.height);

        println!();
        println!("You should see:");
        println!("  - LEFT: Semi-transparent sidebar with Mica effect");
        println!("  - RIGHT: Solid opaque content area");
        println!("  - Desktop wallpaper visible through sidebar only");
        println!();

        Self {
            backend,
            window,
            scene,
            compositor,
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::Window {
                event: WindowEvent::Resized(size),
                ..
            } => {
                self.backend.resize(size.width, size.height);
                self.scene = create_demo_scene(size.width, size.height);
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

/// Creates the demo scene with sidebar and content areas
fn create_demo_scene(width: u32, height: u32) -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    // Sidebar region - semi-transparent to show Mica
    let sidebar = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            render_engine::VisualStyle::new().solid_fill(Color::rgba(1.0, 1.0, 1.0, 0.1).as_vec4()),
        ), // Very transparent white
    });
    let sidebar_id = scene.add_node(root, sidebar);
    if let Some(node) = scene.get_node_mut(sidebar_id) {
        node.bounds = Rect::new(0.0, 0.0, SIDEBAR_WIDTH, height as f32);
    }

    // Sidebar items (semi-transparent cards)
    let items = ["Dashboard", "Documents", "Settings", "Help"];
    for (i, label) in items.iter().enumerate() {
        let item = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                render_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(1.0, 1.0, 1.0, 0.15).as_vec4()),
            ), // Semi-transparent card
        });
        let item_id = scene.add_node(sidebar_id, item);
        if let Some(node) = scene.get_node_mut(item_id) {
            node.bounds = Rect::new(15.0, 60.0 + (i as f32 * 50.0), SIDEBAR_WIDTH - 30.0, 40.0);
        }
        let _ = label; // Label would be rendered with text system
    }

    // Content region - solid opaque background
    let content = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            render_engine::VisualStyle::new()
                .solid_fill(Color::rgba(0.98, 0.98, 0.98, 1.0).as_vec4()),
        ), // Solid light gray
    });
    let content_id = scene.add_node(root, content);
    if let Some(node) = scene.get_node_mut(content_id) {
        node.bounds = Rect::new(
            SIDEBAR_WIDTH,
            0.0,
            width as f32 - SIDEBAR_WIDTH,
            height as f32,
        );
    }

    // Content cards (solid, demonstrating opaque rendering)
    for i in 0..3 {
        let card = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                render_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(1.0, 1.0, 1.0, 1.0).as_vec4()),
            ), // Solid white
        });
        let card_id = scene.add_node(content_id, card);
        if let Some(node) = scene.get_node_mut(card_id) {
            node.bounds = Rect::new(
                SIDEBAR_WIDTH + 30.0,
                30.0 + (i as f32 * 150.0),
                width as f32 - SIDEBAR_WIDTH - 60.0,
                120.0,
            );
        }
    }

    scene
}

fn main() {
    env_logger::init();
    plat_core::run::<CompositionApp>().expect("Failed to run application");
}
