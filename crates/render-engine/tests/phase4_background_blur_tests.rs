use render_engine::backend::wgpu::effects::backdrop_capture_bounds;

#[test]
fn test_background_blur_inflates_bounds_by_radius_and_clips_to_frame() {
    let b = backdrop_capture_bounds([10, 10, 100, 100], 12.0, [0, 0, 120, 120]);
    assert_eq!(b, [0, 0, 120, 120]);
}
