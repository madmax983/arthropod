//! Riot Waves generated Figma example.
//!
//! Regenerate the embedded module with:
//! `cargo run --bin figma_codegen -- --input riot_waves.json --output examples/generated/riot_waves_generated_module.rs --module-name riot_waves_generated --document-fn document --runtime-fn runtime`

use std::time::Instant;

use arthropod::figma_runtime::FigmaRuntime;
use arthropod::prototype_runtime::PrototypeRuntimeEvent;
use plat_core::{
    Application, ControlFlow, ElementState, Event, EventLoop, MouseButton, Size, Window,
    WindowConfig, WindowEvent, WindowId,
};
use render_engine::{
    NodeId,
    backend::{RenderBackend, WgpuBackend},
};

#[allow(dead_code)]
#[path = "generated/riot_waves_generated_module.rs"]
mod riot_waves_generated_module;

const DEFAULT_WIDTH: u32 = 1280;
const DEFAULT_HEIGHT: u32 = 720;

struct RiotWavesApp {
    backend: WgpuBackend,
    window: Window,
    runtime: FigmaRuntime,
    hovered_node: Option<NodeId>,
    last_frame: Instant,
}

impl Application for RiotWavesApp {
    fn new(event_loop: &EventLoop) -> Self {
        let config = WindowConfig {
            title: "Riot Waves (Generated from Figma JSON)".to_string(),
            size: Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT),
            resizable: true,
            visible: true,
            ..Default::default()
        };

        let window = event_loop
            .create_window(config)
            .expect("failed to create window");
        let size = window.inner_size();

        // SAFETY: backend is dropped before window based on struct field order.
        let backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .expect("failed to create backend");

        let mut runtime = riot_waves_generated_module::riot_waves_generated::runtime()
            .expect("failed to initialize generated riot_waves runtime");
        runtime.apply_layout(size.width as f32, size.height as f32);

        window.request_redraw();

        Self {
            backend,
            window,
            runtime,
            hovered_node: None,
            last_frame: Instant::now(),
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
                self.runtime
                    .apply_layout(size.width as f32, size.height as f32);
                self.window.request_redraw();
            }
            Event::Window {
                event: WindowEvent::CursorMoved { position },
                ..
            } => {
                let hovered = self
                    .runtime
                    .scene()
                    .hit_test(position.x as f32, position.y as f32);
                if hovered != self.hovered_node {
                    self.hovered_node = hovered;
                    if let Some(node) = hovered {
                        let _ = self.runtime.dispatch(PrototypeRuntimeEvent::Hover { node });
                        self.window.request_redraw();
                    }
                }
            }
            Event::Window {
                event: WindowEvent::MouseInput(input),
                ..
            } => {
                if input.button == MouseButton::Left && input.state == ElementState::Pressed {
                    let target = self
                        .runtime
                        .scene()
                        .hit_test(input.position.x as f32, input.position.y as f32);
                    if let Some(node) = target {
                        self.hovered_node = Some(node);
                        let _ = self.runtime.dispatch(PrototypeRuntimeEvent::Press { node });
                        let _ = self.runtime.dispatch(PrototypeRuntimeEvent::Click { node });
                        self.window.request_redraw();
                    }
                }
            }
            Event::Window {
                event: WindowEvent::KeyboardInput(input),
                ..
            } => {
                if input.state == ElementState::Pressed {
                    let target = self.hovered_node.or_else(|| self.runtime.current_screen());
                    if let Some(node) = target {
                        let _ = self
                            .runtime
                            .dispatch(PrototypeRuntimeEvent::KeyDown { node });
                        self.window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        let now = Instant::now();
        let elapsed_ms_u128 = now.duration_since(self.last_frame).as_millis();
        self.last_frame = now;
        let elapsed_ms = u32::try_from(elapsed_ms_u128).unwrap_or(u32::MAX);

        if elapsed_ms > 0 {
            let _ = self
                .runtime
                .dispatch(PrototypeRuntimeEvent::Tick { elapsed_ms });
        }

        if let Err(err) = self.backend.render(self.runtime.scene()) {
            eprintln!("render error: {err}");
        }

        self.window.request_redraw();
    }
}

fn main() {
    env_logger::init();
    plat_core::run::<RiotWavesApp>().expect("failed to run riot waves example");
}
