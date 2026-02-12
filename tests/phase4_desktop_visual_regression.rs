#![cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use arthropod_test::visual_test::{compare_images_with_tolerance, load_image, save_image};
use plat_core::{EventLoop, Size, WindowConfig};
use render_engine::{
    Color,
    backend::{RenderBackend, WgpuBackend},
};

#[path = "../examples/phase4_visual_scenes.rs"]
mod phase4_visual_scenes;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const DEFAULT_CHANNEL_TOLERANCE: u8 = 2;
const DEFAULT_MAX_DIFFERENCE_RATIO: f32 = 0.02;
static VISUAL_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn acquire_visual_test_lock() -> std::sync::MutexGuard<'static, ()> {
    VISUAL_TEST_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
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

            let in_tl =
                x >= margin_x && x < margin_x + block_w && y >= margin_y && y < margin_y + block_h;
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

#[test]
fn phase4_desktop_visual_regression_matches_golden() {
    let _guard = acquire_visual_test_lock();
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let window = event_loop
        .create_window(WindowConfig {
            title: "Phase4 Desktop Visual Regression".to_string(),
            size: Size::new(WIDTH, HEIGHT),
            visible: false,
            ..Default::default()
        })
        .expect("failed to create window");
    // SAFETY: backend is dropped before window due to declaration order in this scope.
    let mut backend =
        unsafe { WgpuBackend::new(&window, WIDTH, HEIGHT, false) }.expect("backend init failed");
    backend.set_clear_color(Color::rgba(0.02, 0.05, 0.09, 1.0));
    backend
        .register_image_rgba8(
            phase4_visual_scenes::PHASE4_IMAGE_TEST_ID,
            64,
            48,
            phase4_image_bytes(64, 48),
        )
        .expect("failed to register phase4 image asset");

    let artifact_dir = PathBuf::from("tests/visual/artifacts/phase4");
    std::fs::create_dir_all(&artifact_dir).expect("failed to create artifact dir");
    let channel_tolerance = env_u8(
        "ARTHROPOD_VISUAL_CHANNEL_TOLERANCE",
        DEFAULT_CHANNEL_TOLERANCE,
    );
    let max_difference_ratio = env_f32(
        "ARTHROPOD_VISUAL_MAX_DIFF_RATIO",
        DEFAULT_MAX_DIFFERENCE_RATIO,
    );

    let cases = [
        ("blur", phase4_visual_scenes::build_phase4_blur_scene()),
        ("blend", phase4_visual_scenes::build_phase4_blend_scene()),
        (
            "clipping",
            phase4_visual_scenes::build_phase4_clipping_scene(),
        ),
        ("mask", phase4_visual_scenes::build_phase4_mask_scene()),
        ("image", phase4_visual_scenes::build_phase4_image_scene()),
    ];

    for (name, scene) in cases {
        let rendered = backend
            .render_scene_to_rgba(&scene, WIDTH, HEIGHT)
            .expect("failed to render offscreen capture");

        let artifact_path = artifact_dir.join(format!("{name}.png"));
        save_image(&artifact_path, &rendered, WIDTH, HEIGHT)
            .expect("failed to save rendered artifact");

        let golden_path = Path::new("tests/visual/golden/phase4").join(format!("{name}.png"));
        assert!(
            golden_path.exists(),
            "missing golden image at {}. generate with: cargo run --example capture_phase4_visuals",
            golden_path.display()
        );

        let (golden, golden_w, golden_h) =
            load_image(&golden_path).expect("failed to load golden image");
        assert_eq!(
            (golden_w, golden_h),
            (WIDTH, HEIGHT),
            "golden dimensions must match capture dimensions"
        );

        let diff_ratio =
            compare_images_with_tolerance(&rendered, &golden, WIDTH, HEIGHT, channel_tolerance)
                .expect("diff computation failed");

        assert!(
            diff_ratio <= max_difference_ratio,
            "visual regression for case '{name}' exceeded threshold: diff_ratio={diff_ratio:.4}, threshold={max_difference_ratio:.4}, channel_tolerance={channel_tolerance}. artifacts: {}",
            artifact_path.display()
        );
    }
}

#[test]
fn phase4_blend_modes_render_distinct_outputs() {
    let _guard = acquire_visual_test_lock();
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let window = event_loop
        .create_window(WindowConfig {
            title: "Phase4 Blend Distinctness".to_string(),
            size: Size::new(WIDTH, HEIGHT),
            visible: false,
            ..Default::default()
        })
        .expect("failed to create window");
    // SAFETY: backend is dropped before window due to declaration order in this scope.
    let mut backend =
        unsafe { WgpuBackend::new(&window, WIDTH, HEIGHT, false) }.expect("backend init failed");
    backend.set_clear_color(Color::rgba(0.02, 0.05, 0.09, 1.0));

    let scene = phase4_visual_scenes::build_phase4_blend_scene();
    let rendered = backend
        .render_scene_to_rgba(&scene, WIDTH, HEIGHT)
        .expect("failed to render blend scene");

    // Sample overlap region for first two blend columns:
    // idx0 Multiply at x=140, idx1 Screen at x=285.
    let px_multiply = sample_rgba(&rendered, WIDTH, 140 + 80, 210 + 70);
    let px_screen = sample_rgba(&rendered, WIDTH, 285 + 80, 210 + 70);

    let diff = channel_abs_sum(px_multiply, px_screen);
    assert!(
        diff > 20,
        "blend columns should render distinct pixels, got diff={diff} multiply={px_multiply:?} screen={px_screen:?}"
    );

    // Screen should be brighter than multiply for this color pair.
    let lum_multiply = luma(px_multiply);
    let lum_screen = luma(px_screen);
    assert!(
        lum_screen > lum_multiply,
        "expected Screen to be brighter than Multiply, got multiply_luma={lum_multiply}, screen_luma={lum_screen}, multiply={px_multiply:?}, screen={px_screen:?}"
    );
}

fn env_u8(name: &str, default: u8) -> u8 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(default)
}

fn env_f32(name: &str, default: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(default)
}

fn sample_rgba(rgba: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * width + x) * 4) as usize;
    [rgba[idx], rgba[idx + 1], rgba[idx + 2], rgba[idx + 3]]
}

fn channel_abs_sum(a: [u8; 4], b: [u8; 4]) -> u32 {
    a.into_iter()
        .zip(b)
        .map(|(x, y)| x.abs_diff(y) as u32)
        .sum()
}

fn luma(px: [u8; 4]) -> u32 {
    // ITU-R BT.709 integer approximation.
    (2126u32 * px[0] as u32 + 7152u32 * px[1] as u32 + 722u32 * px[2] as u32) / 10_000
}
