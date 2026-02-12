//! Web visual regression fixture for Phase 4 effects.
//!
//! Build:
//! `trunk build --example phase4_visual_web --features web`
//!
//! Run:
//! `trunk serve --example phase4_visual_web --features web`
//!
//! Cases:
//! - `/?case=blur`
//! - `/?case=blend`
//! - `/?case=clipping`

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("phase4_visual_web is intended for wasm32 targets.");
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
        BlendMode, Color, ColorStop, Effect, NodeContent, Paint, Scene, SceneNode, StrokeStyle,
        VisualStyle,
        backend::{RenderBackend, WgpuBackend},
    };
    use style_engine::{BackgroundBlur, LayerBlur, LinearGradient, StrokeAlign};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::spawn_local;
    use web_sys::console;

    const APP_TITLE: &str = "Arthropod Phase4 Visual Fixture";
    const INTER_REGULAR_FONT: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum VisualCase {
        Blur,
        Blend,
        Clipping,
    }

    impl VisualCase {
        fn from_query() -> Self {
            let search = web_sys::window()
                .and_then(|window| window.location().search().ok())
                .unwrap_or_default();

            for pair in search.trim_start_matches('?').split('&') {
                if pair.is_empty() {
                    continue;
                }

                let mut parts = pair.splitn(2, '=');
                let key = parts.next().unwrap_or_default();
                let value = parts.next().unwrap_or_default();

                if key == "case" {
                    return match value {
                        "blend" => Self::Blend,
                        "clipping" => Self::Clipping,
                        _ => Self::Blur,
                    };
                }
            }

            Self::Blur
        }

        fn as_str(self) -> &'static str {
            match self {
                Self::Blur => "blur",
                Self::Blend => "blend",
                Self::Clipping => "clipping",
            }
        }
    }

    struct Phase4VisualApp {
        case: VisualCase,
        backend: Option<WgpuBackend>,
        backend_init_result: Rc<RefCell<Option<Result<WgpuBackend, String>>>>,
        backend_init_failed: bool,
        _window: Rc<Window>,
        scene: Scene,
        pending_size: Option<Size<u32>>,
        ready_marker_set: bool,
    }

    fn log_startup_failure(stage: &str, error: &str) {
        console::error_1(
            &format!("phase4_visual_web startup failed during {stage}: {error}").into(),
        );
    }

    fn set_ready_marker(case: VisualCase) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(body) = document.body() else {
            return;
        };

        let _ = body.set_attribute("data-arthropod-ready", "1");
        let _ = body.set_attribute("data-arthropod-case", case.as_str());
    }

    impl Phase4VisualApp {
        fn build_blur_scene() -> Scene {
            let mut scene = Scene::new();
            let root = scene.root();

            let mut background = SceneNode::new(NodeContent::Styled {
                style: Box::new(VisualStyle::new().fill(Paint::Linear(LinearGradient {
                    start: glam::Vec2::new(0.0, 0.0),
                    end: glam::Vec2::new(1.0, 1.0),
                    stops: vec![
                        ColorStop::new(0.0, Color::rgba(0.05, 0.10, 0.18, 1.0).as_vec4()),
                        ColorStop::new(1.0, Color::rgba(0.12, 0.20, 0.34, 1.0).as_vec4()),
                    ],
                }))),
            });
            background.bounds = Rect::new(0.0, 0.0, 1280.0, 720.0);
            scene.add_node(root, background);

            let mut color_bars = SceneNode::new(NodeContent::Styled {
                style: Box::new(VisualStyle::new().fill(Paint::Linear(LinearGradient {
                    start: glam::Vec2::new(0.0, 0.5),
                    end: glam::Vec2::new(1.0, 0.5),
                    stops: vec![
                        ColorStop::new(0.00, Color::rgba(0.93, 0.29, 0.47, 1.0).as_vec4()),
                        ColorStop::new(0.33, Color::rgba(0.96, 0.72, 0.24, 1.0).as_vec4()),
                        ColorStop::new(0.66, Color::rgba(0.19, 0.81, 0.63, 1.0).as_vec4()),
                        ColorStop::new(1.00, Color::rgba(0.25, 0.68, 0.94, 1.0).as_vec4()),
                    ],
                }))),
            });
            color_bars.bounds = Rect::new(120.0, 190.0, 980.0, 260.0);
            scene.add_node(root, color_bars);

            let mut layer_blur_card = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .solid_fill(Color::rgba(1.0, 1.0, 1.0, 0.08).as_vec4())
                        .stroke(StrokeStyle::solid(
                            Paint::Solid(Color::rgba(1.0, 1.0, 1.0, 0.30).as_vec4()),
                            1.0,
                            StrokeAlign::Inside,
                        ))
                        .corner_radius(22.0)
                        .effect(Effect::LayerBlur(LayerBlur {
                            radius: 18.0,
                            visible: true,
                        })),
                ),
            });
            layer_blur_card.bounds = Rect::new(180.0, 120.0, 460.0, 320.0);
            scene.add_node(root, layer_blur_card);

            let mut backdrop_blur_card = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .solid_fill(Color::rgba(0.95, 0.97, 1.0, 0.10).as_vec4())
                        .stroke(StrokeStyle::solid(
                            Paint::Solid(Color::rgba(0.85, 0.92, 1.0, 0.32).as_vec4()),
                            1.0,
                            StrokeAlign::Inside,
                        ))
                        .corner_radius(22.0)
                        .effect(Effect::BackgroundBlur(BackgroundBlur {
                            radius: 16.0,
                            visible: true,
                        })),
                ),
            });
            backdrop_blur_card.bounds = Rect::new(620.0, 220.0, 460.0, 320.0);
            scene.add_node(root, backdrop_blur_card);

            scene
        }

        fn build_blend_scene() -> Scene {
            let mut scene = Scene::new();
            let root = scene.root();

            let mut background = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .solid_fill(Color::rgba(0.07, 0.09, 0.13, 1.0).as_vec4())
                        .corner_radius(20.0),
                ),
            });
            background.bounds = Rect::new(90.0, 90.0, 1100.0, 540.0);
            scene.add_node(root, background);

            let blend_modes = [
                BlendMode::Multiply,
                BlendMode::Screen,
                BlendMode::Overlay,
                BlendMode::Darken,
                BlendMode::Lighten,
                BlendMode::Difference,
                BlendMode::Exclusion,
            ];

            for (idx, mode) in blend_modes.iter().enumerate() {
                let x = 140.0 + idx as f32 * 145.0;
                let mut base = SceneNode::new(NodeContent::Styled {
                    style: Box::new(
                        VisualStyle::new()
                            .solid_fill(Color::rgba(0.23, 0.46, 0.94, 0.85).as_vec4())
                            .corner_radius(16.0),
                    ),
                });
                base.bounds = Rect::new(x, 210.0, 120.0, 230.0);
                scene.add_node(root, base);

                let mut top = SceneNode::new(NodeContent::Styled {
                    style: Box::new(
                        VisualStyle::new()
                            .solid_fill(Color::rgba(0.95, 0.35, 0.30, 0.78).as_vec4())
                            .blend_mode(*mode)
                            .corner_radius(16.0),
                    ),
                });
                top.bounds = Rect::new(x + 28.0, 170.0, 120.0, 230.0);
                scene.add_node(root, top);
            }

            scene
        }

        fn build_clipping_scene() -> Scene {
            let mut scene = Scene::new();
            let root = scene.root();

            let mut background = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new().solid_fill(Color::rgba(0.04, 0.08, 0.12, 1.0).as_vec4()),
                ),
            });
            background.bounds = Rect::new(0.0, 0.0, 1280.0, 720.0);
            scene.add_node(root, background);

            let mut outer_clip = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .solid_fill(Color::rgba(0.14, 0.17, 0.23, 1.0).as_vec4())
                        .corner_radius(18.0)
                        .clips_content(true),
                ),
            });
            outer_clip.bounds = Rect::new(160.0, 110.0, 960.0, 500.0);
            let outer_id = scene.add_node(root, outer_clip);

            let mut inner_clip = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .solid_fill(Color::rgba(0.20, 0.24, 0.33, 1.0).as_vec4())
                        .corner_radius(14.0)
                        .clips_content(true),
                ),
            });
            inner_clip.bounds = Rect::new(90.0, 70.0, 780.0, 360.0);
            let inner_id = scene.add_node(outer_id, inner_clip);

            let mut diagonal_band = SceneNode::new(NodeContent::Styled {
                style: Box::new(VisualStyle::new().fill(Paint::Linear(LinearGradient {
                    start: glam::Vec2::new(0.0, 0.0),
                    end: glam::Vec2::new(1.0, 1.0),
                    stops: vec![
                        ColorStop::new(0.0, Color::rgba(0.97, 0.41, 0.55, 1.0).as_vec4()),
                        ColorStop::new(1.0, Color::rgba(0.20, 0.72, 0.98, 1.0).as_vec4()),
                    ],
                }))),
            });
            diagonal_band.bounds = Rect::new(-140.0, -40.0, 1040.0, 420.0);
            scene.add_node(inner_id, diagonal_band);

            let mut escape_rect = SceneNode::new(NodeContent::Styled {
                style: Box::new(
                    VisualStyle::new()
                        .solid_fill(Color::rgba(0.97, 0.78, 0.22, 0.95).as_vec4())
                        .corner_radius(22.0),
                ),
            });
            escape_rect.bounds = Rect::new(680.0, 240.0, 300.0, 220.0);
            scene.add_node(inner_id, escape_rect);

            scene
        }

        fn build_scene(case: VisualCase) -> Scene {
            match case {
                VisualCase::Blur => Self::build_blur_scene(),
                VisualCase::Blend => Self::build_blend_scene(),
                VisualCase::Clipping => Self::build_clipping_scene(),
            }
        }

        fn create_window_or_panic(event_loop: &EventLoop) -> Window {
            let case = VisualCase::from_query();
            let title = format!("{APP_TITLE} [{}]", case.as_str());
            let config = WindowConfig {
                title,
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
                    backend.set_clear_color(Color::rgba(0.02, 0.05, 0.09, 1.0));
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
                    let _ = backend.register_font_bytes(INTER_REGULAR_FONT.to_vec());
                    if let Some(size) = self.pending_size.take() {
                        backend.resize(size.width.max(1), size.height.max(1));
                    }
                    console::log_1(
                        &format!(
                            "phase4_visual_web backend initialized [{}]",
                            self.case.as_str()
                        )
                        .into(),
                    );
                    self.backend = Some(backend);
                }
                Err(err) => {
                    self.backend_init_failed = true;
                    log_startup_failure("WGPU backend init", &err);
                }
            }
        }
    }

    impl Application for Phase4VisualApp {
        fn new(event_loop: &EventLoop) -> Self {
            let case = VisualCase::from_query();
            let scene = Self::build_scene(case);
            let window = Rc::new(Self::create_window_or_panic(event_loop));
            let backend_init_result = Rc::new(RefCell::new(None));
            Self::begin_backend_init(Rc::clone(&window), Rc::clone(&backend_init_result));

            Self {
                case,
                backend: None,
                backend_init_result,
                backend_init_failed: false,
                _window: window,
                scene,
                pending_size: None,
                ready_marker_set: false,
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
                    &format!("phase4_visual_web render failed: {err}").into(),
                );
                return;
            }

            if !self.ready_marker_set {
                set_ready_marker(self.case);
                self.ready_marker_set = true;
                console::log_1(&format!("phase4_visual_web ready [{}]", self.case.as_str()).into());
            }
        }
    }

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        plat_core::run::<Phase4VisualApp>().map_err(|e| {
            let message = format!("phase4_visual_web run failed: {e}");
            console::error_1(&message.clone().into());
            JsValue::from_str(&message)
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_app::start;
