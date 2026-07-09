use glam::Vec2;
use render_engine::backend::wgpu::image_store::{register_image_rgba8, sample_image_fill};
use style_engine::{ImageFill, ImageId, ImageScaleMode};

#[test]
fn test_havoc_image_sample_f32_overflow() {
    // 16_777_219 is 2^24 + 3. In f32, the nearest representable float is 16_777_220.
    // When converted back to u32, it becomes 16_777_220, which is > max_x.
    let width = 16_777_219 + 1; // max_x = 16_777_219
    let height = 2; // max_y = 1
    let rgba8 = vec![0; (width * height * 4) as usize];
    register_image_rgba8(ImageId(999), width, height, rgba8).unwrap();

    let fill = ImageFill {
        image_id: ImageId(999),
        scale_mode: ImageScaleMode::Stretch,
        transform: None,
    };

    println!("Trying to sample at 1.0, 1.0");
    // u = 1.0, v = 1.0. fx = 1.0 * 16_777_219 as f32 = 16_777_220.
    // fy = 1.0 * 1 as f32 = 1.0.
    // x0 = fx.floor() as u32 = 16_777_220.
    // y0 = 1.
    // idx = (1 * 16_777_220 + 16_777_220) * 4 ... wait, width is 16_777_220.
    // idx = (1 * 16_777_220 + 16_777_220) * 4 = (33_554_440) * 4 = 134_217_760.
    // Length is 16_777_220 * 2 * 4 = 134_217_760.
    // Wait, idx = 134_217_760 is out of bounds by 1! (len is 134_217_760, max idx is len-1).
    let _ = sample_image_fill(&fill, Vec2::new(1.0, 1.0), Vec2::new(100.0, 100.0));
}
