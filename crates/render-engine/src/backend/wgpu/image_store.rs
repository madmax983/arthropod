//! CPU-side image store for `Paint::Image` sampling in path rendering.

use glam::{Vec2, Vec4};
use std::sync::{OnceLock, RwLock};
use style_engine::{ImageFill, ImageId, ImageScaleMode};

#[derive(Debug, Clone)]
struct CpuImage {
    width: u32,
    height: u32,
    rgba8: Vec<u8>,
}

fn image_store() -> &'static RwLock<hashbrown::HashMap<ImageId, CpuImage>> {
    static STORE: OnceLock<RwLock<hashbrown::HashMap<ImageId, CpuImage>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(hashbrown::HashMap::new()))
}

/// Register/update an RGBA8 image that can be sampled by `Paint::Image`.
pub fn register_image_rgba8(
    image_id: ImageId,
    width: u32,
    height: u32,
    rgba8: Vec<u8>,
) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err("image dimensions must be non-zero".to_string());
    }

    let expected_len = (width as usize)
        .checked_mul(height as usize)
        .and_then(|area| area.checked_mul(4))
        .ok_or_else(|| "image dimensions too large, causing overflow".to_string())?;

    if rgba8.len() != expected_len {
        return Err(format!(
            "invalid RGBA8 byte length: got {}, expected {} ({}x{}x4)",
            rgba8.len(),
            expected_len,
            width,
            height
        ));
    }

    let mut store = image_store()
        .write()
        .map_err(|_| "image store lock poisoned".to_string())?;
    store.insert(
        image_id,
        CpuImage {
            width,
            height,
            rgba8,
        },
    );
    Ok(())
}

/// Remove a previously-registered image.
pub fn unregister_image(image_id: ImageId) {
    if let Ok(mut store) = image_store().write() {
        store.remove(&image_id);
    }
}

/// Sample a `Paint::Image` at normalized local UV.
///
/// Returns `None` when the image is unavailable.
#[must_use]
pub fn sample_image_fill(fill: &ImageFill, uv: Vec2, target_size: Vec2) -> Option<Vec4> {
    let store = image_store().read().ok()?;
    let image = store.get(&fill.image_id)?;

    let mut uv = uv;
    if let Some(transform) = fill.transform {
        uv = apply_transform(uv, transform);
    }

    let mapped =
        map_uv_for_scale_mode(uv, fill.scale_mode, target_size, image.width, image.height)?;
    Some(sample_bilinear(image, mapped))
}

fn apply_transform(uv: Vec2, m: [f32; 9]) -> Vec2 {
    let x = m[0] * uv.x + m[1] * uv.y + m[2];
    let y = m[3] * uv.x + m[4] * uv.y + m[5];
    let w = m[6] * uv.x + m[7] * uv.y + m[8];
    if w.abs() > 1.0e-6 {
        Vec2::new(x / w, y / w)
    } else {
        Vec2::new(x, y)
    }
}

fn wrap01(v: f32) -> f32 {
    let f = v.fract();
    if f < 0.0 { f + 1.0 } else { f }
}

fn map_uv_for_scale_mode(
    uv: Vec2,
    scale_mode: ImageScaleMode,
    target_size: Vec2,
    image_width: u32,
    image_height: u32,
) -> Option<Vec2> {
    let target_w = target_size.x.max(1.0);
    let target_h = target_size.y.max(1.0);
    let target_aspect = target_w / target_h;
    let image_aspect = image_width as f32 / image_height as f32;

    match scale_mode {
        ImageScaleMode::Tile => Some(Vec2::new(wrap01(uv.x), wrap01(uv.y))),
        ImageScaleMode::Stretch => Some(uv),
        ImageScaleMode::Fit => {
            if image_aspect > target_aspect {
                let visible_h = target_aspect / image_aspect;
                let y0 = (1.0 - visible_h) * 0.5;
                let y1 = y0 + visible_h;
                if uv.y < y0 || uv.y > y1 {
                    return Some(Vec2::new(-1.0, -1.0));
                }
                Some(Vec2::new(uv.x, (uv.y - y0) / visible_h))
            } else {
                let visible_w = image_aspect / target_aspect;
                let x0 = (1.0 - visible_w) * 0.5;
                let x1 = x0 + visible_w;
                if uv.x < x0 || uv.x > x1 {
                    return Some(Vec2::new(-1.0, -1.0));
                }
                Some(Vec2::new((uv.x - x0) / visible_w, uv.y))
            }
        }
        ImageScaleMode::Fill | ImageScaleMode::Crop => {
            if image_aspect > target_aspect {
                let used_w = target_aspect / image_aspect;
                let x0 = (1.0 - used_w) * 0.5;
                Some(Vec2::new(x0 + uv.x * used_w, uv.y))
            } else {
                let used_h = image_aspect / target_aspect;
                let y0 = (1.0 - used_h) * 0.5;
                Some(Vec2::new(uv.x, y0 + uv.y * used_h))
            }
        }
    }
}

fn sample_texel(image: &CpuImage, x: u32, y: u32) -> Vec4 {
    let idx = ((y * image.width + x) * 4) as usize;
    // Decode stored sRGB texels to linear space before shading.
    let r = srgb_to_linear(image.rgba8[idx] as f32 / 255.0);
    let g = srgb_to_linear(image.rgba8[idx + 1] as f32 / 255.0);
    let b = srgb_to_linear(image.rgba8[idx + 2] as f32 / 255.0);
    let a = image.rgba8[idx + 3] as f32 / 255.0;
    Vec4::new(r, g, b, a)
}

fn srgb_to_linear(channel: f32) -> f32 {
    let c = channel.clamp(0.0, 1.0);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn sample_bilinear(image: &CpuImage, uv: Vec2) -> Vec4 {
    if uv.x < 0.0 || uv.y < 0.0 {
        return Vec4::ZERO;
    }

    let u = uv.x.clamp(0.0, 1.0);
    let v = uv.y.clamp(0.0, 1.0);
    let max_x = image.width.saturating_sub(1);
    let max_y = image.height.saturating_sub(1);
    let fx = u * max_x as f32;
    let fy = v * max_y as f32;

    let x0 = fx.floor() as u32;
    let y0 = fy.floor() as u32;
    let x1 = (x0 + 1).min(max_x);
    let y1 = (y0 + 1).min(max_y);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;

    let c00 = sample_texel(image, x0, y0);
    let c10 = sample_texel(image, x1, y0);
    let c01 = sample_texel(image, x0, y1);
    let c11 = sample_texel(image, x1, y1);

    let top = c00.lerp(c10, tx);
    let bottom = c01.lerp(c11, tx);
    top.lerp(bottom, ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_rejects_invalid_byte_len() {
        let result = register_image_rgba8(ImageId(10), 2, 2, vec![0; 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_rejects_overflowing_dimensions() {
        // Use dimensions that would overflow usize when multiplied by 4
        // For a 32-bit architecture, usize::MAX / 2 * 4 will definitely overflow
        // For a 64-bit architecture, u32::MAX * u32::MAX * 4 > usize::MAX
        let width = u32::MAX;
        let height = u32::MAX;
        let result = register_image_rgba8(ImageId(99), width, height, vec![]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "image dimensions too large, causing overflow"
        );
    }

    #[test]
    fn test_sample_image_fill_reads_registered_pixel() {
        register_image_rgba8(
            ImageId(11),
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, // row 0
                0, 0, 255, 255, 255, 255, 255, 255, // row 1
            ],
        )
        .expect("register image");
        let fill = ImageFill {
            image_id: ImageId(11),
            scale_mode: ImageScaleMode::Fill,
            transform: None,
        };
        let c = sample_image_fill(&fill, Vec2::new(0.05, 0.05), Vec2::new(100.0, 100.0))
            .expect("sample should exist");
        assert!(c.x > 0.9 && c.y < 0.1 && c.z < 0.1);
    }

    #[test]
    fn test_sample_image_fill_tile_wraps_uv() {
        register_image_rgba8(
            ImageId(12),
            1,
            1,
            vec![64, 128, 255, 255], // single texel
        )
        .expect("register image");
        let fill = ImageFill {
            image_id: ImageId(12),
            scale_mode: ImageScaleMode::Tile,
            transform: None,
        };
        let c = sample_image_fill(&fill, Vec2::new(2.25, -1.75), Vec2::new(30.0, 10.0))
            .expect("sample should exist");
        assert!((c.x - srgb_to_linear(64.0 / 255.0)).abs() < 1e-6);
        assert!((c.y - srgb_to_linear(128.0 / 255.0)).abs() < 1e-6);
        assert!((c.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sample_image_fill_stretch_preserves_full_image_range() {
        register_image_rgba8(
            ImageId(13),
            4,
            1,
            vec![
                255, 0, 0, 255, // red
                0, 255, 0, 255, // green
                0, 0, 255, 255, // blue
                255, 255, 0, 255, // yellow
            ],
        )
        .expect("register image");

        let fill = ImageFill {
            image_id: ImageId(13),
            scale_mode: ImageScaleMode::Stretch,
            transform: None,
        };

        // With stretch, low/high x UV should map to image extremes even with aspect mismatch.
        let left = sample_image_fill(&fill, Vec2::new(0.1, 0.5), Vec2::new(100.0, 100.0))
            .expect("sample should exist");
        let right = sample_image_fill(&fill, Vec2::new(0.9, 0.5), Vec2::new(100.0, 100.0))
            .expect("sample should exist");

        assert!(
            left.x > left.y && left.x > left.z,
            "left sample should remain red-dominant under stretch, got {:?}",
            left
        );
        assert!(
            right.x > 0.6 && right.y > 0.6 && right.z < 0.4,
            "right sample should approach yellow under stretch, got {:?}",
            right
        );
    }

    #[test]
    fn test_sample_image_fill_blends_texels_for_smoother_sampling() {
        register_image_rgba8(
            ImageId(14),
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, // row 0
                0, 0, 255, 255, 255, 255, 255, 255, // row 1
            ],
        )
        .expect("register image");

        let fill = ImageFill {
            image_id: ImageId(14),
            scale_mode: ImageScaleMode::Stretch,
            transform: None,
        };

        let center = sample_image_fill(&fill, Vec2::new(0.5, 0.5), Vec2::new(128.0, 128.0))
            .expect("sample should exist");

        // Bilinear center sample should roughly average the 4 texels.
        assert!((center.x - 0.5).abs() < 0.1, "unexpected red: {}", center.x);
        assert!(
            (center.y - 0.5).abs() < 0.1,
            "unexpected green: {}",
            center.y
        );
        assert!(
            (center.z - 0.5).abs() < 0.1,
            "unexpected blue: {}",
            center.z
        );
    }
}
