//! Minimal test: Single hardcoded rectangle
//! Purpose: Verify the rendering pipeline works at all

use arthropod::prelude::*;

struct TestApp {
    app: arthropod::App,
    #[allow(dead_code)]
    node_id: NodeId,
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

        // Create app with App builder
        let mut app = arthropod::AppBuilder::new()
            .with_window_config(config)
            .build(event_loop)
            .expect("Failed to create app");

        // Set up scene once (Scene is now a Resource in ECS)
        let node_id = {
            let mut scene = app.world_mut().resource_mut::<Scene>();
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
                parent: None,
                visible: true,
                opacity: 1.0,
            };

            scene.add_node(root, red_rect)
        };

        // Spawn entity with Renderable component
        app.spawn(node_id).insert(Renderable);

        println!("Window created. You should see a RED rectangle at (100, 100) size 200x200");

        Self { app, node_id }
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
    }
}

fn main() {
    plat_core::run::<TestApp>().expect("Failed to run");
}
