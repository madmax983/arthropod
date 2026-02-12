use plat_core::{
    Application, ControlFlow, Event, EventLoop, Size, Window, WindowConfig, WindowEvent, WindowId,
};
use render_engine::{
    Color, Scene,
    backend::{RenderBackend, WgpuBackend},
};

#[path = "phase4_visual_scenes.rs"]
mod phase4_visual_scenes;

struct Phase4BlendVisualTest {
    backend: WgpuBackend,
    window: Window,
    scene: Scene,
}

impl Application for Phase4BlendVisualTest {
    fn new(event_loop: &EventLoop) -> Self {
        println!("=== Phase 4 Blend Visual Test (Desktop) ===");
        println!("Rendering live native scene for blend mode compositing.");

        let config = WindowConfig {
            title: "Phase 4 Visual Test - Blend".to_string(),
            size: Size::new(1280, 720),
            resizable: true,
            visible: true,
            ..Default::default()
        };

        let window = event_loop
            .create_window(config)
            .expect("Failed to create window");
        let size = window.inner_size();

        // SAFETY: Backend is dropped before window due to struct field order.
        let mut backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .expect("Failed to create backend");
        backend.set_clear_color(Color::rgba(0.02, 0.05, 0.09, 1.0));

        let scene = phase4_visual_scenes::build_phase4_blend_scene();

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
            } => *control_flow = ControlFlow::Exit,
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
        if let Err(err) = self.backend.render(&self.scene) {
            eprintln!("Render error: {err}");
        }
    }
}

fn main() {
    env_logger::init();
    plat_core::run::<Phase4BlendVisualTest>().expect("Failed to run Phase 4 blend visual test");
}
