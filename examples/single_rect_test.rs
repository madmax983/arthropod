//! Minimal test: Single hardcoded rectangle
//! Purpose: Verify the rendering pipeline works at all

use plat_core::{
    Application, ControlFlow, Event, EventLoop, Size, Window, WindowConfig, WindowEvent,
};
use render_engine::backend::{RenderBackend, WgpuBackend};

struct TestApp {
    #[allow(dead_code)]
    window: Window,
    backend: WgpuBackend,
}

impl Application for TestApp {
    fn new(event_loop: &EventLoop) -> Self {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::DEBUG.into()),
            )
            .init();

        let config = WindowConfig {
            title: "Single Rectangle Test".to_string(),
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

        println!("Window created. You should see a RED rectangle at (100, 100) size 200x200");

        Self { window, backend }
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
        // Create a minimal scene with just one rectangle
        use plat_core::Rect;
        use render_engine::{Color, NodeContent, Scene, SceneNode, Transform2D};

        let mut scene = Scene::new();
        let root = scene.root();

        let red_rect = SceneNode {
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

        scene.add_node(root, red_rect);

        if let Err(e) = self.backend.render(&scene) {
            eprintln!("Render error: {}", e);
        }
    }
}

fn main() {
    plat_core::run::<TestApp>().expect("Failed to run");
}
