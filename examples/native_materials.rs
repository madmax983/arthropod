//! Example: Native Materials (Mica/Acrylic)
//!
//! Demonstrates platform-native backdrop materials on Windows 11 using the
//! window-vibrancy crate for full-window effects.
//!
//! ## How It Works
//!
//! - **window-vibrancy**: Applies Mica/Acrylic to the entire window via DWM APIs
//! - **Semi-transparent rendering**: wgpu content with alpha < 1.0 shows backdrop through
//! - **Uniform effect**: The backdrop applies to the whole window (not selective areas)
//!
//! ## What You'll See
//!
//! - Desktop wallpaper blurred/tinted behind the window
//! - Semi-transparent cards showing Mica/Acrylic effect through them
//! - Theme-aware colors from DesignTokens
//!
//! ## Future: DirectComposition (Option 3)
//!
//! For selective transparency (Mica in some areas, solid in others), we'll need
//! DirectComposition integration to composite wgpu output onto a compositor visual tree.
//! This is planned as the next enhancement.
//!
//! Run with: cargo run --example native_materials
//!
//! Note: Mica requires Windows 11. On Windows 10, falls back to Acrylic blur.

use plat_core::{
    Application, BackdropMaterial, ControlFlow, Event, EventLoop, Rect, Size, Window, WindowConfig,
    WindowEvent, WindowId,
};
use render_engine::{Color, NodeContent, Scene, SceneNode, backend::WgpuBackend};
use theme_engine::{DesignTokens, SystemTheme};

/// Application state for the native materials demo
struct MaterialApp {
    // Backend must be dropped before window
    backend: WgpuBackend,
    window: Window,
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

        // Create GPU backend (standard mode - not using DirectComposition)
        let size = window.inner_size();
        // SAFETY: Safe because backend is dropped before window (struct field order)
        let mut backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .expect("Failed to create backend");

        // IMPORTANT: Use semi-transparent clear color to show Mica backdrop through.
        // With window-vibrancy applying the effect to the entire window,
        // any transparent pixels in our rendered content will show the backdrop.
        // Alpha 0.5 = 50% transparent, letting Mica blend through nicely.
        backend.set_clear_color(Color::rgba(0.0, 0.0, 0.0, 0.5));

        // Create scene with semi-transparent cards to demonstrate Mica effect
        let scene = create_demo_scene(&tokens);

        println!();
        println!("You should see:");
        println!("  - Desktop wallpaper blurred/tinted behind the window (Mica)");
        println!("  - Semi-transparent cards showing backdrop effect through them");
        println!("  - Theme-aware colors using DesignTokens");
        println!("  - Accent-colored card using system accent color");
        println!();
        println!("Implementation: window-vibrancy applies effect to entire window,");
        println!("               semi-transparent wgpu rendering shows it through.");
        println!();
        println!("Try moving the window over different desktop areas!");

        Self {
            backend,
            window,
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
    let card1 = SceneNode::new(NodeContent::Styled {
        style: Box::new(render_engine::VisualStyle::new().solid_fill(
            Color::rgba(elevated_color.x, elevated_color.y, elevated_color.z, 0.85).as_vec4(),
        )),
    });
    let card1_id = scene.add_node(root, card1);
    if let Some(node) = scene.get_node_mut(card1_id) {
        node.bounds = Rect::new(50.0, 50.0, 300.0, 200.0);
    }

    // Card 2: Accent colored card (top-right)
    // Uses system accent color with slight transparency
    let accent = tokens.accent;
    let card2 = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            render_engine::VisualStyle::new()
                .solid_fill(Color::rgba(accent.x, accent.y, accent.z, 0.9).as_vec4()),
        ),
    });
    let card2_id = scene.add_node(root, card2);
    if let Some(node) = scene.get_node_mut(card2_id) {
        node.bounds = Rect::new(400.0, 50.0, 300.0, 200.0);
    }

    // Card 3: Secondary surface (bottom, spanning width)
    // More transparent to show off the Mica effect
    let secondary = tokens.surface_secondary.as_color();
    let card3 = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            render_engine::VisualStyle::new()
                .solid_fill(Color::rgba(secondary.x, secondary.y, secondary.z, 0.75).as_vec4()),
        ),
    });
    let card3_id = scene.add_node(root, card3);
    if let Some(node) = scene.get_node_mut(card3_id) {
        node.bounds = Rect::new(50.0, 300.0, 650.0, 250.0);
    }

    // Card 4: Very transparent card to really show the backdrop
    let card4 = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            render_engine::VisualStyle::new().solid_fill(Color::rgba(1.0, 1.0, 1.0, 0.3).as_vec4()),
        ),
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
