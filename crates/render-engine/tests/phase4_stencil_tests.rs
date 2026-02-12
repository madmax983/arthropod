use render_engine::backend::wgpu::effects::EffectPassKind;
use render_engine::backend::wgpu::pipelines::stencil_pipeline::plan_clip_sequence_for_nested_clips;

#[test]
fn test_nested_clip_pass_sequence_is_balanced() {
    let seq = plan_clip_sequence_for_nested_clips();
    assert_eq!(
        seq,
        vec![
            EffectPassKind::StencilPush,
            EffectPassKind::StencilPush,
            EffectPassKind::StencilPop,
            EffectPassKind::StencilPop
        ]
    );
}
