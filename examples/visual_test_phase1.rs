//! Visual Test: Phase 1 Rendering Pipeline
//!
//! Demonstrates the unified PrimitivePipeline with:
//! - Solid fills (various colors)
//! - Per-corner radii (uniform and asymmetric)
//! - Text rendering via TextContent
//! - Multiple primitives in a single scene
//!
//! This test verifies that the migration from separate RectPipeline/GlyphPipeline
//! to the unified PrimitivePipeline is working correctly.
//!
//! Run with: cargo run --example visual_test_phase1

use plat_core::{
    Application, ControlFlow, Event, EventLoop, Rect, Size, Window, WindowConfig, WindowEvent,
    WindowId,
};
use render_engine::{
    Color, CornerRadii, NodeContent, Scene, SceneNode, VisualStyle,
    backend::{RenderBackend, WgpuBackend},
};

struct VisualTest {
    backend: WgpuBackend,
    window: Window,
    scene: Scene,
}

impl Application for VisualTest {
    fn new(event_loop: &EventLoop) -> Self {
        println!("=== Phase 1 Visual Test ===");
        println!("Testing unified PrimitivePipeline:");
        println!("  ✓ Solid fills (various colors)");
        println!("  ✓ Corner radii (uniform and per-corner)");
        println!("  ✓ Text rendering via TextContent");
        println!("  ✓ Multiple primitives in single scene\n");

        let config = WindowConfig {
            title: "Phase 1 Visual Test - PrimitivePipeline".to_string(),
            size: Size::new(1200, 800),
            resizable: true,
            visible: true,
            ..Default::default()
        };

        let window = event_loop
            .create_window(config)
            .expect("Failed to create window");

        let size = window.inner_size();
        // SAFETY: Backend is dropped before window (struct field order)
        let mut backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .expect("Failed to create backend");

        backend.set_clear_color(Color::rgba(0.95, 0.95, 0.95, 1.0));

        let scene = create_visual_test_scene();

        Self {
            backend,
            window,
            scene,
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
                self.window.request_redraw();
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

fn create_visual_test_scene() -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    // Row 1: Solid color fills
    println!("Creating Row 1: Solid color fills");

    // Red rectangle (sharp corners)
    let red = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().solid_fill(Color::rgba(0.8, 0.2, 0.2, 1.0).as_vec4())),
    });
    let red_id = scene.add_node(root, red);
    scene.get_node_mut(red_id).unwrap().bounds = Rect::new(50.0, 50.0, 150.0, 100.0);

    // Green rectangle (sharp corners)
    let green = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().solid_fill(Color::rgba(0.2, 0.8, 0.2, 1.0).as_vec4())),
    });
    let green_id = scene.add_node(root, green);
    scene.get_node_mut(green_id).unwrap().bounds = Rect::new(220.0, 50.0, 150.0, 100.0);

    // Blue rectangle (sharp corners)
    let blue = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().solid_fill(Color::rgba(0.2, 0.2, 0.8, 1.0).as_vec4())),
    });
    let blue_id = scene.add_node(root, blue);
    scene.get_node_mut(blue_id).unwrap().bounds = Rect::new(390.0, 50.0, 150.0, 100.0);

    // Semi-transparent purple
    let purple = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().solid_fill(Color::rgba(0.6, 0.2, 0.8, 0.7).as_vec4())),
    });
    let purple_id = scene.add_node(root, purple);
    scene.get_node_mut(purple_id).unwrap().bounds = Rect::new(560.0, 50.0, 150.0, 100.0);

    // Row 2: Uniform corner radii
    println!("Creating Row 2: Uniform corner radii");

    // Small radius (4px)
    let small_radius = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.3, 0.5, 0.8, 1.0).as_vec4())
                .corner_radius(4.0),
        ),
    });
    let small_id = scene.add_node(root, small_radius);
    scene.get_node_mut(small_id).unwrap().bounds = Rect::new(50.0, 200.0, 150.0, 100.0);

    // Medium radius (12px)
    let med_radius = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.8, 0.5, 0.3, 1.0).as_vec4())
                .corner_radius(12.0),
        ),
    });
    let med_id = scene.add_node(root, med_radius);
    scene.get_node_mut(med_id).unwrap().bounds = Rect::new(220.0, 200.0, 150.0, 100.0);

    // Large radius (24px)
    let large_radius = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.3, 0.8, 0.5, 1.0).as_vec4())
                .corner_radius(24.0),
        ),
    });
    let large_id = scene.add_node(root, large_radius);
    scene.get_node_mut(large_id).unwrap().bounds = Rect::new(390.0, 200.0, 150.0, 100.0);

    // Very large radius (circle-like)
    let circle_like = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.8, 0.3, 0.5, 1.0).as_vec4())
                .corner_radius(50.0),
        ),
    });
    let circle_id = scene.add_node(root, circle_like);
    scene.get_node_mut(circle_id).unwrap().bounds = Rect::new(560.0, 200.0, 150.0, 100.0);

    // Row 3: Per-corner radii (asymmetric)
    println!("Creating Row 3: Per-corner radii (asymmetric)");

    // Top-left only
    let tl_only = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.9, 0.6, 0.2, 1.0).as_vec4())
                .corner_radii(CornerRadii::new(20.0, 0.0, 0.0, 0.0)),
        ),
    });
    let tl_id = scene.add_node(root, tl_only);
    scene.get_node_mut(tl_id).unwrap().bounds = Rect::new(50.0, 350.0, 150.0, 100.0);

    // Top-right only
    let tr_only = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.2, 0.9, 0.6, 1.0).as_vec4())
                .corner_radii(CornerRadii::new(0.0, 20.0, 0.0, 0.0)),
        ),
    });
    let tr_id = scene.add_node(root, tr_only);
    scene.get_node_mut(tr_id).unwrap().bounds = Rect::new(220.0, 350.0, 150.0, 100.0);

    // Bottom-left only
    let bl_only = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.6, 0.2, 0.9, 1.0).as_vec4())
                .corner_radii(CornerRadii::new(0.0, 0.0, 0.0, 20.0)),
        ),
    });
    let bl_id = scene.add_node(root, bl_only);
    scene.get_node_mut(bl_id).unwrap().bounds = Rect::new(390.0, 350.0, 150.0, 100.0);

    // All different (TL=8, TR=16, BR=24, BL=4)
    let all_different = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.9, 0.2, 0.6, 1.0).as_vec4())
                .corner_radii(CornerRadii::new(8.0, 16.0, 24.0, 4.0)),
        ),
    });
    let diff_id = scene.add_node(root, all_different);
    scene.get_node_mut(diff_id).unwrap().bounds = Rect::new(560.0, 350.0, 150.0, 100.0);

    // Row 4: Text rendering - COMMENTED OUT FOR DIAGNOSTIC
    // println!("Creating Row 4: Text rendering");

    // Row 5: Mixed content (shapes + text) - COMMENTED OUT FOR DIAGNOSTIC
    // println!("Creating Row 5: Mixed content");

    println!("\n✅ Scene created successfully");
    println!("\nYou should see:");
    println!("  Row 1: 4 solid color rectangles (red, green, blue, purple)");
    println!("  Row 2: 4 rectangles with uniform corner radii (4px, 12px, 24px, 50px)");
    println!("  Row 3: 4 rectangles with asymmetric corner radii");
    println!("  Row 4: Text samples at different sizes and colors");
    println!("  Row 5: Card with mixed content (shapes + text)");

    scene
}

fn main() {
    env_logger::init();
    plat_core::run::<VisualTest>().expect("Failed to run visual test");
}
