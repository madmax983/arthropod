use render_engine::backend::wgpu::pipelines::blur_pipeline::{BlurTier, select_blur_tier};

#[test]
fn test_select_blur_tier_uses_downsample_path_for_large_radius() {
    assert_eq!(select_blur_tier(8.0), BlurTier::FullRes);
    assert_eq!(select_blur_tier(36.0), BlurTier::HalfRes);
}
