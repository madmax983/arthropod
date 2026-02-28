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
//! - `/?case=mask`
//! - `/?case=image`

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("phase4_visual_web is intended for wasm32 targets.");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[path = "phase4_visual_scenes.rs"]
mod phase4_visual_scenes;

#[cfg(target_arch = "wasm32")]
mod wasm_app {
    use std::cell::RefCell;
    use std::rc::Rc;

    use plat_core::{
        Application, ControlFlow, ElementState, Event, EventLoop, Key, Size, Window, WindowConfig,
        WindowEvent, WindowId,
    };
    use render_engine::{
        Color, Scene,
        backend::{RenderBackend, WgpuBackend},
    };
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::spawn_local;
    use web_sys::console;

    use super::phase4_visual_scenes;

    const APP_TITLE: &str = "Arthropod Phase4 Visual Fixture";
    const INTER_REGULAR_FONT: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
    const IMAGE_FIXTURE_WIDTH: u32 = 192;
    const IMAGE_FIXTURE_HEIGHT: u32 = 144;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum VisualCase {
        Blur,
        Blend,
        Clipping,
        Mask,
        Image,
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
                        "mask" => Self::Mask,
                        "image" => Self::Image,
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
                Self::Mask => "mask",
                Self::Image => "image",
            }
        }

        fn needs_image_fixture(self) -> bool {
            matches!(self, Self::Image)
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

    fn sanitize_error_marker_value(message: &str) -> String {
        let mut output = String::with_capacity(message.len().min(1024));
        for ch in message.chars() {
            if ch == '\0' {
                continue;
            }

            if ch.is_control() && !matches!(ch, '\n' | '\r' | '\t') {
                output.push(' ');
            } else {
                output.push(ch);
            }

            if output.len() >= 1024 {
                break;
            }
        }

        if output.is_empty() {
            "phase4_visual_web startup failed: unavailable error details".to_string()
        } else {
            output
        }
    }

    fn phase4_image_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        let margin_x = 6u32;
        let margin_y = 6u32;
        let gap_x = 4u32;
        let gap_y = 4u32;
        let block_w = (width.saturating_sub(margin_x * 2 + gap_x)) / 2;
        let block_h = (height.saturating_sub(margin_y * 2 + gap_y)) / 2;
        let cx = (width / 2) as i32;
        let cy = (height / 2) as i32;
        let r2 = 7_i32 * 7_i32;

        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                let mut color = [28u8, 36u8, 52u8, 255u8];

                let in_tl = x >= margin_x
                    && x < margin_x + block_w
                    && y >= margin_y
                    && y < margin_y + block_h;
                let in_tr = x >= margin_x + block_w + gap_x
                    && x < margin_x + block_w + gap_x + block_w
                    && y >= margin_y
                    && y < margin_y + block_h;
                let in_bl = x >= margin_x
                    && x < margin_x + block_w
                    && y >= margin_y + block_h + gap_y
                    && y < margin_y + block_h + gap_y + block_h;
                let in_br = x >= margin_x + block_w + gap_x
                    && x < margin_x + block_w + gap_x + block_w
                    && y >= margin_y + block_h + gap_y
                    && y < margin_y + block_h + gap_y + block_h;

                if in_tl {
                    color = [224, 86, 86, 255];
                } else if in_tr {
                    color = [83, 188, 236, 255];
                } else if in_bl {
                    color = [237, 196, 85, 255];
                } else if in_br {
                    color = [163, 116, 229, 255];
                }

                if x == 0 || y == 0 || x + 1 == width || y + 1 == height {
                    color = [245, 245, 245, 255];
                }

                if x.abs_diff(width / 2) <= 1 || y.abs_diff(height / 2) <= 1 {
                    color = [22, 22, 22, 255];
                }

                let dx = x as i32 - cx;
                let dy = y as i32 - cy;
                if dx * dx + dy * dy <= r2 {
                    color = [245, 245, 245, 255];
                }

                rgba[idx] = color[0];
                rgba[idx + 1] = color[1];
                rgba[idx + 2] = color[2];
                rgba[idx + 3] = 255;
            }
        }

        rgba
    }

    fn log_startup_failure(stage: &str, error: &str) {
        let message = format!("phase4_visual_web startup failed during {stage}: {error}");
        console::error_1(&message.clone().into());
        set_error_marker(&message);
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

    fn set_error_marker(message: &str) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(body) = document.body() else {
            return;
        };
        let _ = body.set_attribute("data-arthropod-ready", "0");
        let sanitized = sanitize_error_marker_value(message);
        if body
            .set_attribute("data-arthropod-error", sanitized.as_str())
            .is_err()
        {
            let _ = body.set_attribute(
                "data-arthropod-error",
                "phase4_visual_web startup failed: unavailable error details",
            );
        }
    }

    fn install_panic_marker_hook() {
        console_error_panic_hook::set_once();
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let message = format!("phase4_visual_web panic: {panic_info}");
            console::error_1(&message.clone().into());
            set_error_marker(&message);
            previous_hook(panic_info);
        }));
    }

    impl Phase4VisualApp {
        fn build_scene(case: VisualCase) -> Scene {
            match case {
                VisualCase::Blur => phase4_visual_scenes::build_phase4_blur_scene(),
                VisualCase::Blend => phase4_visual_scenes::build_phase4_blend_scene(),
                VisualCase::Clipping => phase4_visual_scenes::build_phase4_clipping_scene(),
                VisualCase::Mask => phase4_visual_scenes::build_phase4_mask_scene(),
                VisualCase::Image => phase4_visual_scenes::build_phase4_image_scene(),
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

                    if self.case.needs_image_fixture() {
                        let image_bytes =
                            phase4_image_bytes(IMAGE_FIXTURE_WIDTH, IMAGE_FIXTURE_HEIGHT);
                        if let Err(err) = backend.register_image_rgba8(
                            phase4_visual_scenes::PHASE4_IMAGE_TEST_ID,
                            IMAGE_FIXTURE_WIDTH,
                            IMAGE_FIXTURE_HEIGHT,
                            image_bytes,
                        ) {
                            self.backend_init_failed = true;
                            log_startup_failure("image fixture registration", &err.to_string());
                            return;
                        }
                    }

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
                self.backend_init_failed = true;
                log_startup_failure("render", &err.to_string());
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
        install_panic_marker_hook();
        plat_core::run::<Phase4VisualApp>().map_err(|e| {
            let message = format!("phase4_visual_web run failed: {e}");
            console::error_1(&message.clone().into());
            JsValue::from_str(&message)
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_app::start;
