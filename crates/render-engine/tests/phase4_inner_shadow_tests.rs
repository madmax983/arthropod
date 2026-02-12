use render_engine::backend::wgpu::effects::inner_shadow_alpha;

#[test]
fn test_inner_shadow_alpha_nonzero_only_inside_mask() {
    assert_eq!(inner_shadow_alpha(0.0, 1.0), 0.0);
    assert!(inner_shadow_alpha(1.0, 0.8) > 0.0);
}
