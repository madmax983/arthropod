//! Arthropod demo application.

use plat_core::{Application, ControlFlow, Event, EventLoop, WindowConfig, WindowId};

struct DemoApp {
    _window: plat_core::Window,
}

impl Application for DemoApp {
    fn new(event_loop: &EventLoop) -> Self {
        let window = event_loop
            .create_window(WindowConfig {
                title: "Arthropod - Phase 1 Demo".into(),
                ..Default::default()
            })
            .expect("Failed to create window");

        Self { _window: window }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window { event, .. } => match event {
                plat_core::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        // Rendering will be implemented later
    }
}

fn main() {
    env_logger::init();
    plat_core::run::<DemoApp>().expect("Failed to run application");
}
