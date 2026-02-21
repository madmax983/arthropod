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
    let expected_len = width as usize * height as usize * 4;
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
    Some(sample_nearest(image, mapped))
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

fn sample_nearest(image: &CpuImage, uv: Vec2) -> Vec4 {
    if uv.x < 0.0 || uv.y < 0.0 {
        return Vec4::ZERO;
    }

    let u = uv.x.clamp(0.0, 1.0);
    let v = uv.y.clamp(0.0, 1.0);
    let x = (u * (image.width.saturating_sub(1)) as f32).round() as u32;
    let y = (v * (image.height.saturating_sub(1)) as f32).round() as u32;
    let idx = ((y * image.width + x) * 4) as usize;
    let r = image.rgba8[idx] as f32 / 255.0;
    let g = image.rgba8[idx + 1] as f32 / 255.0;
    let b = image.rgba8[idx + 2] as f32 / 255.0;
    let a = image.rgba8[idx + 3] as f32 / 255.0;
    Vec4::new(r, g, b, a)
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
        assert!((c.x - 64.0 / 255.0).abs() < 1e-6);
        assert!((c.y - 128.0 / 255.0).abs() < 1e-6);
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

        assert!(left.x > 0.9 && left.y < 0.1 && left.z < 0.1);
        assert!(right.x > 0.9 && right.y > 0.9 && right.z < 0.1);
    }
}
