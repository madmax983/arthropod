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
