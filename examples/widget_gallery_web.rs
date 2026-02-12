//! Web entrypoint for Widget Gallery.
//!
//! Build:
//! `trunk build --example widget_gallery_web --features web`
//!
//! Run:
//! `trunk serve --example widget_gallery_web --features web`

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("widget_gallery_web is intended for wasm32 targets.");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
mod wasm_app {
    use std::cell::RefCell;
    use std::rc::Rc;

    use plat_core::{
        Application, ControlFlow, ElementState, Event, EventLoop, Key, Rect, Size, Window,
        WindowConfig, WindowEvent, WindowId,
    };
    use render_engine::{
        Color, NodeContent, Scene, SceneNode, TextContent, VisualStyle,
        backend::{RenderBackend, WgpuBackend},
    };
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::spawn_local;
    use web_sys::console;

    const APP_TITLE: &str = "Arthropod Widget Gallery Live";
    const INTER_REGULAR_FONT: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");

    struct WebGalleryApp {
        backend: Option<WgpuBackend>,
        backend_init_result: Rc<RefCell<Option<Result<WgpuBackend, String>>>>,
        backend_init_failed: bool,
        _window: Rc<Window>,
        scene: Scene,
        pending_size: Option<Size<u32>>,
    }

    fn log_startup_failure(stage: &str, error: &str) {
        console::error_1(
            &format!("widget_gallery_web startup failed during {stage}: {error}").into(),
        );
    }

    impl WebGalleryApp {
        fn build_scene() -> Scene {
            let mut scene = Scene::new();
            let root = scene.root();

            let mut panel = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .solid_fill(Color::rgba(0.10, 0.14, 0.20, 1.0).as_vec4())
                        .corner_radius(20.0),
                ),
            });
            panel.bounds = Rect::new(96.0, 72.0, 1088.0, 576.0);
            scene.add_node(root, panel);

            let mut title = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .solid_fill(Color::WHITE.as_vec4())
                        .text(TextContent::new(APP_TITLE, 40.0)),
                ),
            });
            title.bounds = Rect::new(140.0, 126.0, 980.0, 72.0);
            scene.add_node(root, title);

            let mut subtitle = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .solid_fill(Color::rgba(0.75, 0.82, 0.95, 1.0).as_vec4())
                        .text(TextContent::new(
                            "Phase 4/5 pipeline running in browser",
                            22.0,
                        )),
                ),
            });
            subtitle.bounds = Rect::new(144.0, 190.0, 940.0, 40.0);
            scene.add_node(root, subtitle);

            let swatches = [
                (170.0, 300.0, Color::rgba(0.20, 0.68, 0.95, 1.0)),
                (430.0, 300.0, Color::rgba(0.14, 0.82, 0.62, 1.0)),
                (690.0, 300.0, Color::rgba(0.99, 0.73, 0.24, 1.0)),
                (950.0, 300.0, Color::rgba(0.96, 0.38, 0.51, 1.0)),
            ];

            for (x, y, color) in swatches {
                let mut node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(
                        VisualStyle::new()
                            .solid_fill(color.as_vec4())
                            .corner_radius(14.0),
                    ),
                });
                node.bounds = Rect::new(x, y, 180.0, 180.0);
                scene.add_node(root, node);
            }

            scene
        }

        fn create_window_or_panic(event_loop: &EventLoop) -> Window {
            let config = WindowConfig {
                title: APP_TITLE.to_string(),
                size: Size::new(1280, 720),
                ..Default::default()
            };

            match event_loop.create_window(config) {
                Ok(window) => window,
                Err(err) => {
                    log_startup_failure("window creation", &err.to_string());
                    panic!("failed to create web window: {err}");
                }
            }
        }

        fn begin_backend_init(
            window: Rc<Window>,
            backend_init_result: Rc<RefCell<Option<Result<WgpuBackend, String>>>>,
        ) {
            spawn_local(async move {
                let size = window.inner_size();
                let init = unsafe {
                    WgpuBackend::new_async(
                        window.as_ref(),
                        size.width.max(1),
                        size.height.max(1),
                        false,
                    )
                    .await
                }
                .map_err(|err| err.to_string())
                .map(|mut backend| {
                    backend.set_clear_color(Color::rgba(0.03, 0.06, 0.10, 1.0));
                    backend
                });

                *backend_init_result.borrow_mut() = Some(init);
            });
        }

        fn poll_backend_init(&mut self) {
            if self.backend.is_some() || self.backend_init_failed {
                return;
            }

            let Some(init_result) = self.backend_init_result.borrow_mut().take() else {
                return;
            };

            match init_result {
                Ok(mut backend) => {
                    let loaded_faces = backend.register_font_bytes(INTER_REGULAR_FONT.to_vec());
                    if loaded_faces == 0 {
                        console::warn_1(
                            &"widget_gallery_web font registration loaded zero faces".into(),
                        );
                    } else {
                        console::log_1(
                            &format!(
                                "widget_gallery_web font registration loaded {loaded_faces} face(s)"
                            )
                            .into(),
                        );
                    }

                    if let Some(size) = self.pending_size.take() {
                        backend.resize(size.width.max(1), size.height.max(1));
                    }
                    console::log_1(&"widget_gallery_web backend initialized".into());
                    self.backend = Some(backend);
                }
                Err(err) => {
                    self.backend_init_failed = true;
                    log_startup_failure("WGPU backend init", &err);
                }
            }
        }
    }

    impl Application for WebGalleryApp {
        fn new(event_loop: &EventLoop) -> Self {
            let window = Rc::new(Self::create_window_or_panic(event_loop));
            let backend_init_result = Rc::new(RefCell::new(None));
            Self::begin_backend_init(Rc::clone(&window), Rc::clone(&backend_init_result));

            Self {
                backend: None,
                backend_init_result,
                backend_init_failed: false,
                _window: window,
                scene: Self::build_scene(),
                pending_size: None,
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
                    if let Some(backend) = self.backend.as_mut() {
                        backend.resize(size.width.max(1), size.height.max(1));
                    } else {
                        self.pending_size = Some(size);
                    }
                }
                Event::Window {
                    event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                    ..
                } => {
                    if let Some(backend) = self.backend.as_mut() {
                        backend.resize(new_inner_size.width.max(1), new_inner_size.height.max(1));
                    } else {
                        self.pending_size = Some(new_inner_size);
                    }
                }
                Event::Window {
                    event: WindowEvent::KeyboardInput(input),
                    ..
                } if input.state == ElementState::Pressed && input.key == Key::Escape => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        }

        fn on_redraw(&mut self, _window_id: WindowId) {
            self.poll_backend_init();

            let Some(backend) = self.backend.as_mut() else {
                return;
            };

            if let Err(err) = backend.render(&self.scene) {
                web_sys::console::error_1(
                    &format!("widget_gallery_web render failed: {err}").into(),
                );
            }
        }
    }

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        plat_core::run::<WebGalleryApp>().map_err(|e| {
            let message = format!("widget_gallery_web run failed: {e}");
            console::error_1(&message.clone().into());
            JsValue::from_str(&message)
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_app::start;
