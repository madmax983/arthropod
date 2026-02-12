use render_engine::backend::wgpu::effects::{EffectPassKind, EffectPlanNode, plan_effect_passes};
use render_engine::{Effect, NodeContent, Scene, SceneNode};
use style_engine::LayerBlur;

fn make_scene_with_layer_blur() -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut node = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            style_engine::VisualStyle::new().effect(Effect::LayerBlur(LayerBlur {
                radius: 16.0,
                visible: true,
            })),
        ),
    });
    node.bounds = plat_core::Rect::new(10.0, 20.0, 120.0, 80.0);
    scene.add_node(root, node);

    scene
}

#[test]
fn test_effect_plan_orders_layer_blur_pipeline_steps() {
    let scene = make_scene_with_layer_blur();
    let nodes: Vec<EffectPlanNode> = scene
        .iter_visuals()
        .map(|(node_id, node)| EffectPlanNode::from_scene(node_id, node))
        .collect();

    let plan = plan_effect_passes(&nodes, 1280, 720);
    let kinds: Vec<_> = plan.iter().map(|p| p.kind).collect();

    assert_eq!(
        kinds,
        vec![
            EffectPassKind::OffscreenLayer,
            EffectPassKind::BlurHorizontal,
            EffectPassKind::BlurVertical,
            EffectPassKind::BlendComposite,
        ]
    );
}
