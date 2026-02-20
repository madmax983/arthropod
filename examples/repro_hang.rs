use plat_core::{Application, ControlFlow, Event, EventLoop, Window, WindowConfig, WindowEvent};
use std::time::{Duration, Instant};

struct App {
    window: Option<Window>,
    start_time: Instant,
}

impl Application for App {
    fn new(event_loop: &EventLoop) -> Self {
        let window = event_loop
            .create_window(WindowConfig {
                title: "Hang Repro".to_string(),
                size: plat_core::Size::new(800, 600),
                ..Default::default()
            })
            .unwrap();

        println!("Window created");
        window.request_redraw();

        Self {
            window: Some(window),
            start_time: Instant::now(),
        }
    }

    fn on_event(&mut self, event: Event, _control_flow: &mut ControlFlow) {
        if let Event::Window {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            println!("Close requested");
            self.window = None;
        }
    }

    fn on_redraw(&mut self, _window_id: plat_core::WindowId) {
        // Auto-close after 1 second
        if self.start_time.elapsed() > Duration::from_secs(1) {
            if self.window.is_some() {
                println!("Closing window...");
                self.window = None; // Drop the window
            }
        } else if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    // Set a timeout for the process
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(5));
        println!("TIMEOUT: App failed to exit!");
        std::process::exit(1);
    });

    println!("Starting app...");
    plat_core::run::<App>().unwrap();
    println!("App exited successfully!");
}
