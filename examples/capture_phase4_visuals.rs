use std::path::PathBuf;

use arthropod_test::visual_test::save_image;
use plat_core::{EventLoop, Size, WindowConfig};
use render_engine::{
    Color,
    backend::{RenderBackend, WgpuBackend},
};

#[path = "phase4_visual_scenes.rs"]
mod phase4_visual_scenes;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

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

            // White outer border to make scale/tiling easier to identify.
            if x == 0 || y == 0 || x + 1 == width || y + 1 == height {
                color = [245, 245, 245, 255];
            }

            // Dark crosshair marker.
            if x.abs_diff(width / 2) <= 1 || y.abs_diff(height / 2) <= 1 {
                color = [22, 22, 22, 255];
            }

            // Bright center dot marker.
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

fn parse_out_dir() -> PathBuf {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--out"
            && let Some(path) = args.next()
        {
            return PathBuf::from(path);
        }
    }

    PathBuf::from("tests/visual/golden/phase4")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let out_dir = parse_out_dir();
    std::fs::create_dir_all(&out_dir)?;

    let event_loop = EventLoop::new()?;
    let window = event_loop.create_window(WindowConfig {
        title: "Phase4 Desktop Visual Capture".to_string(),
        size: Size::new(WIDTH, HEIGHT),
        visible: false,
        ..Default::default()
    })?;

    // SAFETY: backend is dropped before window in this function scope.
    let mut backend = unsafe { WgpuBackend::new(&window, WIDTH, HEIGHT, false) }?;
    backend.set_clear_color(Color::rgba(0.02, 0.05, 0.09, 1.0));
    backend.register_image_rgba8(
        phase4_visual_scenes::PHASE4_IMAGE_TEST_ID,
        64,
        48,
        phase4_image_bytes(64, 48),
    )?;

    let captures = [
        ("blur", phase4_visual_scenes::build_phase4_blur_scene()),
        ("blend", phase4_visual_scenes::build_phase4_blend_scene()),
        (
            "clipping",
            phase4_visual_scenes::build_phase4_clipping_scene(),
        ),
        ("mask", phase4_visual_scenes::build_phase4_mask_scene()),
        ("image", phase4_visual_scenes::build_phase4_image_scene()),
    ];

    for (name, scene) in captures {
        let rgba = backend.render_scene_to_rgba(&scene, WIDTH, HEIGHT)?;
        let out_path = out_dir.join(format!("{name}.png"));
        save_image(&out_path, &rgba, WIDTH, HEIGHT)?;
        println!("wrote {}", out_path.display());
    }

    Ok(())
}
