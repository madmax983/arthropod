use proptest::prelude::*;
use render_engine::backend::wgpu::effects::RenderTargetKey;

proptest! {
    // 👺 HAVOC: Property testing the memory footprint calculation.
    // This throws random extreme sizes at the allocation logic to ensure it doesn't panic.
    #[test]
    fn test_render_target_estimated_bytes_overflow(
        width in 0..=u32::MAX,
        height in 0..=u32::MAX,
        has_stencil in any::<bool>(),
    ) {
        let key = RenderTargetKey::new(width, height, has_stencil);
        let bytes = key.estimated_bytes();

        // Assert that we don't return less than what we asked for unless we saturated
        if width > 0 && height > 0 {
            assert!(bytes > 0);
        }
    }
}
