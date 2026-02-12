#![allow(dead_code)]

use render_engine::{
    BlendMode, Color, ColorStop, Effect, NodeContent, Paint, Scene, SceneNode, StrokeStyle,
    VisualStyle,
};
use style_engine::{
    BackgroundBlur, ImageFill, ImageId, ImageScaleMode, LayerBlur, LinearGradient, StrokeAlign,
};

pub const PHASE4_IMAGE_TEST_ID: ImageId = ImageId(50_001);

pub fn build_phase4_blur_scene() -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut background = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().fill(Paint::Linear(LinearGradient {
            start: glam::Vec2::new(0.0, 0.0),
            end: glam::Vec2::new(1.0, 1.0),
            stops: vec![
                ColorStop::new(0.0, Color::rgba(0.05, 0.10, 0.18, 1.0).as_vec4()),
                ColorStop::new(1.0, Color::rgba(0.12, 0.20, 0.34, 1.0).as_vec4()),
            ],
        }))),
    });
    background.bounds = plat_core::Rect::new(0.0, 0.0, 1280.0, 720.0);
    scene.add_node(root, background);

    let mut color_bars = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().fill(Paint::Linear(LinearGradient {
            start: glam::Vec2::new(0.0, 0.5),
            end: glam::Vec2::new(1.0, 0.5),
            stops: vec![
                ColorStop::new(0.00, Color::rgba(0.93, 0.29, 0.47, 1.0).as_vec4()),
                ColorStop::new(0.33, Color::rgba(0.96, 0.72, 0.24, 1.0).as_vec4()),
                ColorStop::new(0.66, Color::rgba(0.19, 0.81, 0.63, 1.0).as_vec4()),
                ColorStop::new(1.00, Color::rgba(0.25, 0.68, 0.94, 1.0).as_vec4()),
            ],
        }))),
    });
    color_bars.bounds = plat_core::Rect::new(120.0, 190.0, 980.0, 260.0);
    scene.add_node(root, color_bars);

    let mut layer_blur_card = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(1.0, 1.0, 1.0, 0.08).as_vec4())
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Color::rgba(1.0, 1.0, 1.0, 0.30).as_vec4()),
                    1.0,
                    StrokeAlign::Inside,
                ))
                .corner_radius(22.0)
                .effect(Effect::LayerBlur(LayerBlur {
                    radius: 18.0,
                    visible: true,
                })),
        ),
    });
    layer_blur_card.bounds = plat_core::Rect::new(180.0, 120.0, 460.0, 320.0);
    scene.add_node(root, layer_blur_card);

    let mut backdrop_blur_card = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.95, 0.97, 1.0, 0.10).as_vec4())
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Color::rgba(0.85, 0.92, 1.0, 0.32).as_vec4()),
                    1.0,
                    StrokeAlign::Inside,
                ))
                .corner_radius(22.0)
                .effect(Effect::BackgroundBlur(BackgroundBlur {
                    radius: 16.0,
                    visible: true,
                })),
        ),
    });
    backdrop_blur_card.bounds = plat_core::Rect::new(620.0, 220.0, 460.0, 320.0);
    scene.add_node(root, backdrop_blur_card);

    scene
}

pub fn build_phase4_blend_scene() -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut background = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.07, 0.09, 0.13, 1.0).as_vec4())
                .corner_radius(20.0),
        ),
    });
    background.bounds = plat_core::Rect::new(90.0, 90.0, 1100.0, 540.0);
    scene.add_node(root, background);

    let blend_modes = [
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::Darken,
        BlendMode::Lighten,
        BlendMode::Difference,
        BlendMode::Exclusion,
    ];

    for (idx, mode) in blend_modes.iter().enumerate() {
        let x = 140.0 + idx as f32 * 145.0;
        let mut base = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                VisualStyle::new()
                    .solid_fill(Color::rgba(0.23, 0.46, 0.94, 0.85).as_vec4())
                    .corner_radius(16.0),
            ),
        });
        base.bounds = plat_core::Rect::new(x, 210.0, 120.0, 230.0);
        scene.add_node(root, base);

        let mut top = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                VisualStyle::new()
                    .solid_fill(Color::rgba(0.95, 0.35, 0.30, 0.78).as_vec4())
                    .blend_mode(*mode)
                    .corner_radius(16.0),
            ),
        });
        top.bounds = plat_core::Rect::new(x + 28.0, 170.0, 120.0, 230.0);
        scene.add_node(root, top);
    }

    scene
}

pub fn build_phase4_clipping_scene() -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut background = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new().solid_fill(Color::rgba(0.04, 0.08, 0.12, 1.0).as_vec4()),
        ),
    });
    background.bounds = plat_core::Rect::new(0.0, 0.0, 1280.0, 720.0);
    scene.add_node(root, background);

    let mut outer_clip = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.14, 0.17, 0.23, 1.0).as_vec4())
                .corner_radius(18.0)
                .clips_content(true),
        ),
    });
    outer_clip.bounds = plat_core::Rect::new(160.0, 110.0, 960.0, 500.0);
    let outer_id = scene.add_node(root, outer_clip);

    let mut inner_clip = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.20, 0.24, 0.33, 1.0).as_vec4())
                .corner_radius(14.0)
                .clips_content(true),
        ),
    });
    inner_clip.bounds = plat_core::Rect::new(90.0, 70.0, 780.0, 360.0);
    let inner_id = scene.add_node(outer_id, inner_clip);

    let mut diagonal_band = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().fill(Paint::Linear(LinearGradient {
            start: glam::Vec2::new(0.0, 0.0),
            end: glam::Vec2::new(1.0, 1.0),
            stops: vec![
                ColorStop::new(0.0, Color::rgba(0.97, 0.41, 0.55, 1.0).as_vec4()),
                ColorStop::new(1.0, Color::rgba(0.20, 0.72, 0.98, 1.0).as_vec4()),
            ],
        }))),
    });
    diagonal_band.bounds = plat_core::Rect::new(-140.0, -40.0, 1040.0, 420.0);
    scene.add_node(inner_id, diagonal_band);

    let mut escape_rect = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.30, 0.83, 0.93, 0.92).as_vec4())
                .corner_radius(22.0),
        ),
    });
    escape_rect.bounds = plat_core::Rect::new(680.0, 240.0, 300.0, 220.0);
    scene.add_node(inner_id, escape_rect);

    scene
}

pub fn build_phase4_mask_scene() -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut background = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new().solid_fill(Color::rgba(0.05, 0.07, 0.11, 1.0).as_vec4()),
        ),
    });
    background.bounds = plat_core::Rect::new(0.0, 0.0, 1280.0, 720.0);
    scene.add_node(root, background);

    // Pre-mask sibling remains unaffected.
    let mut before_mask = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new().solid_fill(Color::rgba(0.20, 0.45, 0.95, 0.85).as_vec4()),
        ),
    });
    before_mask.bounds = plat_core::Rect::new(120.0, 120.0, 260.0, 280.0);
    scene.add_node(root, before_mask);

    // Optional backdrop panel behind the mask demo area.
    let mut panel = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.26, 0.30, 0.38, 0.72).as_vec4())
                .corner_radius(20.0),
        ),
    });
    panel.bounds = plat_core::Rect::new(380.0, 90.0, 560.0, 360.0);
    scene.add_node(root, panel);

    let mut mask = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.95, 0.95, 0.98, 0.12).as_vec4())
                .corner_radius(140.0)
                .is_mask(true),
        ),
    });
    mask.bounds = plat_core::Rect::new(460.0, 110.0, 300.0, 300.0);
    scene.add_node(root, mask);

    // Post-mask siblings are clipped to mask bounds.
    let mut masked = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().fill(Paint::Linear(LinearGradient {
            start: glam::Vec2::new(0.0, 0.0),
            end: glam::Vec2::new(1.0, 1.0),
            stops: vec![
                ColorStop::new(0.0, Color::rgba(0.98, 0.42, 0.56, 1.0).as_vec4()),
                ColorStop::new(1.0, Color::rgba(0.23, 0.80, 0.94, 1.0).as_vec4()),
            ],
        }))),
    });
    masked.bounds = plat_core::Rect::new(360.0, 40.0, 520.0, 420.0);
    scene.add_node(root, masked);

    let mut masked_bar = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new().solid_fill(Color::rgba(0.97, 0.78, 0.22, 0.90).as_vec4()),
        ),
    });
    masked_bar.bounds = plat_core::Rect::new(340.0, 250.0, 560.0, 90.0);
    scene.add_node(root, masked_bar);

    let mut masked_chip = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.20, 0.87, 0.70, 0.90).as_vec4())
                .corner_radius(20.0),
        ),
    });
    masked_chip.bounds = plat_core::Rect::new(560.0, 150.0, 220.0, 220.0);
    scene.add_node(root, masked_chip);

    scene
}

pub fn build_phase4_image_scene() -> Scene {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut background = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new().solid_fill(Color::rgba(0.03, 0.06, 0.10, 1.0).as_vec4()),
        ),
    });
    background.bounds = plat_core::Rect::new(0.0, 0.0, 1280.0, 720.0);
    scene.add_node(root, background);

    let mut panel = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.12, 0.17, 0.24, 1.0).as_vec4())
                .corner_radius(20.0),
        ),
    });
    panel.bounds = plat_core::Rect::new(80.0, 70.0, 1120.0, 420.0);
    scene.add_node(root, panel);

    let mut fill_frame = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.07, 0.11, 0.16, 1.0).as_vec4())
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Color::rgba(0.76, 0.82, 0.91, 0.7).as_vec4()),
                    2.0,
                    StrokeAlign::Inside,
                ))
                .corner_radius(14.0),
        ),
    });
    fill_frame.bounds = plat_core::Rect::new(120.0, 120.0, 320.0, 190.0);
    scene.add_node(root, fill_frame);

    let mut fill_node = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().fill(Paint::Image(ImageFill {
            image_id: PHASE4_IMAGE_TEST_ID,
            scale_mode: ImageScaleMode::Fill,
            transform: None,
        }))),
    });
    fill_node.bounds = plat_core::Rect::new(130.0, 130.0, 300.0, 170.0);
    scene.add_node(root, fill_node);

    let mut fit_frame = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.07, 0.11, 0.16, 1.0).as_vec4())
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Color::rgba(0.76, 0.82, 0.91, 0.7).as_vec4()),
                    2.0,
                    StrokeAlign::Inside,
                ))
                .corner_radius(14.0),
        ),
    });
    fit_frame.bounds = plat_core::Rect::new(500.0, 100.0, 260.0, 270.0);
    scene.add_node(root, fit_frame);

    let mut fit_node = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().fill(Paint::Image(ImageFill {
            image_id: PHASE4_IMAGE_TEST_ID,
            scale_mode: ImageScaleMode::Fit,
            transform: None,
        }))),
    });
    fit_node.bounds = plat_core::Rect::new(510.0, 110.0, 240.0, 250.0);
    scene.add_node(root, fit_node);

    let mut tile_frame = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            VisualStyle::new()
                .solid_fill(Color::rgba(0.07, 0.11, 0.16, 1.0).as_vec4())
                .stroke(StrokeStyle::solid(
                    Paint::Solid(Color::rgba(0.76, 0.82, 0.91, 0.7).as_vec4()),
                    2.0,
                    StrokeAlign::Inside,
                ))
                .corner_radius(14.0),
        ),
    });
    tile_frame.bounds = plat_core::Rect::new(820.0, 120.0, 320.0, 190.0);
    scene.add_node(root, tile_frame);

    let mut tile_node = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().fill(Paint::Image(ImageFill {
            image_id: PHASE4_IMAGE_TEST_ID,
            scale_mode: ImageScaleMode::Tile,
            transform: None,
        }))),
    });
    tile_node.bounds = plat_core::Rect::new(830.0, 130.0, 300.0, 170.0);
    scene.add_node(root, tile_node);

    scene
}
