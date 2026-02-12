//! Visual regression testing utilities

use anyhow::Result;
use std::path::Path;

/// Compare two images pixel by pixel
pub fn compare_images(image1: &[u8], image2: &[u8], width: u32, height: u32) -> Result<f32> {
    if image1.len() != image2.len() {
        anyhow::bail!("Image sizes don't match");
    }

    let total_pixels = (width * height) as usize;
    let mut diff_pixels = 0;

    for i in 0..total_pixels {
        let idx = i * 4; // RGBA
        if image1[idx..idx + 4] != image2[idx..idx + 4] {
            diff_pixels += 1;
        }
    }

    Ok(diff_pixels as f32 / total_pixels as f32)
}

/// Compare two images with per-channel tolerance and return ratio of differing pixels.
pub fn compare_images_with_tolerance(
    image1: &[u8],
    image2: &[u8],
    width: u32,
    height: u32,
    tolerance: u8,
) -> Result<f32> {
    if image1.len() != image2.len() {
        anyhow::bail!("Image sizes don't match");
    }

    let total_pixels = (width * height) as usize;
    let mut diff_pixels = 0usize;

    for i in 0..total_pixels {
        let idx = i * 4; // RGBA
        let mut pixel_differs = false;
        for c in 0..4 {
            let a = image1[idx + c] as i16;
            let b = image2[idx + c] as i16;
            if (a - b).abs() > tolerance as i16 {
                pixel_differs = true;
                break;
            }
        }
        if pixel_differs {
            diff_pixels += 1;
        }
    }

    Ok(diff_pixels as f32 / total_pixels as f32)
}

/// Load an image from a file
pub fn load_image(path: impl AsRef<Path>) -> Result<(Vec<u8>, u32, u32)> {
    let img = image::open(path)?.to_rgba8();
    let (width, height) = img.dimensions();
    Ok((img.into_raw(), width, height))
}

/// Save an image to a file
pub fn save_image(path: impl AsRef<Path>, data: &[u8], width: u32, height: u32) -> Result<()> {
    image::save_buffer(path, data, width, height, image::ColorType::Rgba8)?;
    Ok(())
}

/// Check if a specific pixel has the expected color (with tolerance)
pub fn check_pixel_color(
    image_data: &[u8],
    x: u32,
    y: u32,
    width: u32,
    expected_rgba: [u8; 4],
    tolerance: u8,
) -> bool {
    let idx = ((y * width + x) * 4) as usize;
    if idx + 3 >= image_data.len() {
        return false;
    }

    for i in 0..4 {
        let diff = (image_data[idx + i] as i16 - expected_rgba[i] as i16).abs();
        if diff > tolerance as i16 {
            return false;
        }
    }

    true
}

/// Count pixels of a specific color (with tolerance)
pub fn count_color_pixels(
    image_data: &[u8],
    width: u32,
    height: u32,
    expected_rgba: [u8; 4],
    tolerance: u8,
) -> usize {
    let mut count = 0;
    for y in 0..height {
        for x in 0..width {
            if check_pixel_color(image_data, x, y, width, expected_rgba, tolerance) {
                count += 1;
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::{compare_images, compare_images_with_tolerance};

    #[test]
    fn test_compare_images_with_tolerance_ignores_small_channel_delta() {
        let width = 1;
        let height = 1;
        let a = [10u8, 20u8, 30u8, 255u8];
        let b = [11u8, 20u8, 31u8, 255u8];

        let strict = compare_images(&a, &b, width, height).expect("strict compare should work");
        let tolerant = compare_images_with_tolerance(&a, &b, width, height, 2)
            .expect("tolerant compare should work");

        assert_eq!(strict, 1.0);
        assert_eq!(tolerant, 0.0);
    }

    #[test]
    fn test_compare_images_with_tolerance_counts_large_delta_as_difference() {
        let width = 1;
        let height = 1;
        let a = [10u8, 20u8, 30u8, 255u8];
        let b = [20u8, 20u8, 30u8, 255u8];

        let tolerant = compare_images_with_tolerance(&a, &b, width, height, 2)
            .expect("tolerant compare should work");

        assert_eq!(tolerant, 1.0);
    }
}
