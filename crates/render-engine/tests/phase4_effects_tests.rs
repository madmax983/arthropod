use render_engine::backend::wgpu::effects::{
    EffectPassKind, RenderTargetKey, RenderTargetPool, blend_multiply, classify_effect_passes,
    gaussian_kernel_1d,
};
use render_engine::{BlendMode, Effect, VisualStyle};
use style_engine::{BackgroundBlur, LayerBlur};

#[test]
fn test_render_target_pool_reuses_same_key() {
    let mut pool = RenderTargetPool::new(8 * 1024 * 1024);
    let key = RenderTargetKey::new(512, 512, false);

    let first = pool.acquire(key, |_key| (1, 1024));
    pool.release(first).expect("release should succeed");

    let second = pool.acquire(key, |_key| (2, 1024));
    assert_eq!(first, second, "pool should reuse released target");
}

#[test]
fn test_blur_kernel_weights_sum_to_one() {
    let kernel = gaussian_kernel_1d(12.0, 25);
    let sum: f32 = kernel.iter().sum();
    assert!(
        (sum - 1.0).abs() < 0.001,
        "gaussian kernel must normalize to 1, got {}",
        sum
    );
}

#[test]
fn test_blend_mode_multiply_matches_reference() {
    let src = [0.2, 0.5, 0.8];
    let dst = [0.9, 0.4, 0.25];
    let out = blend_multiply(src, dst);
    assert!((out[0] - 0.18).abs() < 1e-6);
    assert!((out[1] - 0.20).abs() < 1e-6);
    assert!((out[2] - 0.20).abs() < 1e-6);
}

#[test]
fn test_effect_planner_classifies_layer_and_background_blur() {
    let style = VisualStyle::new()
        .effect(Effect::LayerBlur(LayerBlur {
            radius: 16.0,
            visible: true,
        }))
        .effect(Effect::BackgroundBlur(BackgroundBlur {
            radius: 12.0,
            visible: true,
        }))
        .blend_mode(BlendMode::Multiply);

    let mut passes = Vec::new();
    classify_effect_passes(&style, false, &mut passes);
    assert!(passes.contains(&EffectPassKind::OffscreenLayer));
    assert!(passes.contains(&EffectPassKind::BackgroundCapture));
    assert!(passes.contains(&EffectPassKind::BlurHorizontal));
    assert!(passes.contains(&EffectPassKind::BlurVertical));
    assert!(passes.contains(&EffectPassKind::BlendComposite));
}

#[test]
fn test_effect_planner_marks_stencil_for_clips_content() {
    let style = VisualStyle::new().clips_content(true);
    let mut passes = Vec::new();
    classify_effect_passes(&style, true, &mut passes);
    assert!(passes.contains(&EffectPassKind::StencilPush));
    assert!(passes.contains(&EffectPassKind::StencilPop));
}
