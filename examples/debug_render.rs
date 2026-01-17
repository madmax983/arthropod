//! Debug rendering with RenderDoc capture and frame inspection

use arthropod_test::{RenderDocCapture, init_test_tracing};
use plat_core::{
    Application, ControlFlow, Event, EventLoop, Rect, Size, Window, WindowConfig, WindowEvent,
};
use render_engine::{
    Color, NodeContent, Scene, SceneNode, Transform2D,
    backend::{RenderBackend, WgpuBackend},
};

struct DebugApp {
    #[allow(dead_code)]
    window: Window,
    backend: WgpuBackend,
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

        let window = event_loop
            .create_window(config)
            .expect("Failed to create window");
        let backend = WgpuBackend::new(&window, 800, 600).expect("Failed to create backend");

        println!("\nTest configuration:");
        println!("  Window size: 800x600");
        println!("  Rectangle: (100, 100) size 200x200");
        println!("  Color: RED (1.0, 0.0, 0.0, 1.0)");
        println!("\nExpected: Red rectangle in upper-left quadrant");
        println!("Actual: Check RenderDoc capture to see GPU output\n");

        Self {
            window,
            backend,
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

    fn on_redraw(&mut self, _window_id: plat_core::WindowId) {
        // Create test scene
        let mut scene = Scene::new();
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
            visible: true,
            opacity: 1.0,
        };

        scene.add_node(root, test_rect);

        if let Err(e) = self.backend.render(&scene) {
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
