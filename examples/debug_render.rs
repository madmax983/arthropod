//! Debug rendering with RenderDoc capture and frame inspection

use arthropod::prelude::*;
use arthropod_test::{RenderDocCapture, init_test_tracing};

struct DebugApp {
    app: arthropod::App,
    #[allow(dead_code)]
    node_id: NodeId,
    renderdoc: RenderDocCapture,
    frame_count: u32,
    captured: bool,
}

impl Application for DebugApp {
    fn new(event_loop: &EventLoop) -> Self {
        init_test_tracing();

        let mut renderdoc = RenderDocCapture::new();

        // Start capture before creating GPU resources
        if renderdoc.is_available() {
            renderdoc.start_capture().expect("Failed to start capture");
            println!("✓ RenderDoc capture started - will capture first frame");
        } else {
            println!("⚠ RenderDoc not available - install RenderDoc and run from its UI");
        }

        let config = WindowConfig {
            title: "Debug Render - RenderDoc Capture".to_string(),
            size: Size {
                width: 800,
                height: 600,
            },
            resizable: false,
            decorations: true,
            visible: true,
            ..Default::default()
        };

        // Create app with App builder
        let mut app = arthropod::AppBuilder::new()
            .with_window_config(config)
            .build(event_loop)
            .expect("Failed to create app");

        // Set up test scene once (Scene is now a Resource in ECS)
        let node_id = {
            let mut scene = app.world_mut().resource_mut::<Scene>();
            let root = scene.root();

            let test_rect = SceneNode {
                content: NodeContent::Rect { color: Color::RED },
                transform: Transform2D::identity(),
                bounds: Rect {
                    x: 100.0,
                    y: 100.0,
                    width: 200.0,
                    height: 200.0,
                },
                children: vec![],
                parent: None,
                visible: true,
                opacity: 1.0,
            };

            scene.add_node(root, test_rect)
        };

        // Spawn entity with Renderable component
        app.spawn(node_id).insert(Renderable);

        println!("\nTest configuration:");
        println!("  Window size: 800x600");
        println!("  Rectangle: (100, 100) size 200x200");
        println!("  Color: RED (1.0, 0.0, 0.0, 1.0)");
        println!("\nExpected: Red rectangle in upper-left quadrant");
        println!("Actual: Check RenderDoc capture to see GPU output\n");

        Self {
            app,
            node_id,
            renderdoc,
            frame_count: 0,
            captured: false,
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        if let Event::Window {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        // Update ECS systems
        self.app.update();

        // Render to GPU
        if let Err(e) = self.app.render_to_gpu() {
            eprintln!("Render error: {}", e);
        }

        self.frame_count += 1;

        // Capture only the first frame
        if self.frame_count == 1 && !self.captured && self.renderdoc.is_available() {
            self.renderdoc.end_capture().expect("Failed to end capture");
            self.captured = true;
            println!("✓ Frame captured! Check RenderDoc UI for the capture.");
            println!("  Look for:");
            println!("    - Pipeline state");
            println!("    - Uniform buffer contents");
            println!("    - Vertex shader outputs");
            println!("    - Fragment shader outputs");
            println!("    - Final framebuffer\n");
        }
    }
}

fn main() {
    println!("=== Arthropod Render Debug Tool ===\n");
    println!("This tool captures the first frame with RenderDoc.");
    println!("Make sure RenderDoc is installed and launch this from RenderDoc UI.\n");

    plat_core::run::<DebugApp>().expect("Failed to run");
}
