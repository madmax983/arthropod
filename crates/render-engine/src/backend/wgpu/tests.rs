use super::*;
use crate::backend::wgpu::effects;
use crate::backend::wgpu::instance_collector;
use crate::backend::wgpu::instance_collector::{TextFill, apply_text_fill_to_glyph};
use crate::backend::wgpu::multipass_executor;
use crate::backend::wgpu::pipelines::primitive_instance::FLAG_FILL_TYPE_MASK;
use crate::{NodeContent, SceneNode, Transform2D};

#[test]
fn test_instance_collection_from_scene() {
    // Create a test scene with rectangles
    let mut scene = Scene::new();
    let root = scene.root();

    // Add visible rectangle
    let red_rect = SceneNode {
        content: NodeContent::Styled {
            style: Box::new(
                style_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4()),
            ),
        },
        transform: Transform2D::identity(),
        bounds: plat_core::Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 200.0,
        },
        children: vec![],
        parent: None, // Set by add_node
        visible: true,
        opacity: 1.0,
    };
    scene.add_node(root, red_rect);

    // Add invisible rectangle (should be skipped)
    let invisible_rect = SceneNode {
        content: NodeContent::Styled {
            style: Box::new(
                style_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(0.0, 1.0, 0.0, 1.0).as_vec4()),
            ),
        },
        transform: Transform2D::identity(),
        bounds: plat_core::Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        },
        children: vec![],
        parent: None, // Set by add_node
        visible: false,
        opacity: 1.0,
    };
    scene.add_node(root, invisible_rect);

    // Add rectangle with opacity
    let blue_rect = SceneNode {
        content: NodeContent::Styled {
            style: Box::new(
                style_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(0.0, 0.0, 1.0, 1.0).as_vec4()),
            ),
        },
        transform: Transform2D::identity(),
        bounds: plat_core::Rect {
            x: 200.0,
            y: 100.0,
            width: 150.0,
            height: 150.0,
        },
        children: vec![],
        parent: None, // Set by add_node
        visible: true,
        opacity: 0.5,
    };
    scene.add_node(root, blue_rect);

    // Collect instances
    let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);

    // Should have 2 instances (red and blue, not invisible)
    assert_eq!(instances.len(), 2);

    // Find red rectangle instance
    let red_instance = instances
        .iter()
        .find(|i| i.color[0] > 0.9 && i.color[1] < 0.1)
        .expect("Red instance not found");
    assert_eq!(red_instance.pos, [10.0, 20.0]);
    assert_eq!(red_instance.size, [100.0, 200.0]);
    assert_eq!(red_instance.color, [1.0, 0.0, 0.0, 1.0]);

    // Find blue rectangle instance (with opacity applied)
    let blue_instance = instances
        .iter()
        .find(|i| i.color[2] > 0.9 && i.color[0] < 0.1)
        .expect("Blue instance not found");
    assert_eq!(blue_instance.pos, [200.0, 100.0]);
    assert_eq!(blue_instance.size, [150.0, 150.0]);
    assert_eq!(blue_instance.color, [0.0, 0.0, 1.0, 0.5]); // opacity applied

    // Both flat rects should have corner_radii = 0.0
    assert_eq!(red_instance.corner_radii, [0.0; 4]);
    assert_eq!(blue_instance.corner_radii, [0.0; 4]);
}

#[test]
fn test_style_opacity_multiplies_node_opacity_for_primitive_instances() {
    let mut scene = Scene::new();
    let root = scene.root();

    let node = SceneNode {
        content: NodeContent::Styled {
            style: Box::new(
                style_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4())
                    .opacity(0.4),
            ),
        },
        transform: Transform2D::identity(),
        bounds: plat_core::Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        children: vec![],
        parent: None,
        visible: true,
        opacity: 0.5,
    };
    scene.add_node(root, node);

    let (instances, _text_nodes, _path_batches) =
        instance_collector::collect_instances_for_tests(&scene);
    assert_eq!(instances.len(), 1);
    assert!(
        (instances[0].color[3] - 0.2).abs() < 1e-6,
        "expected fill alpha 0.2 from node.opacity(0.5) * style.opacity(0.4), got {}",
        instances[0].color[3]
    );
}

#[test]
fn test_rounded_rect_preserves_corner_radius() {
    let mut scene = Scene::new();
    let root = scene.root();

    // Add a RoundedRect with corner_radius
    let rounded = SceneNode {
        content: NodeContent::Styled {
            style: Box::new(
                style_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(0.5, 0.5, 0.5, 1.0).as_vec4())
                    .corner_radius(12.0),
            ),
        },
        transform: Transform2D::identity(),
        bounds: plat_core::Rect {
            x: 30.0,
            y: 30.0,
            width: 200.0,
            height: 100.0,
        },
        children: vec![],
        parent: None,
        visible: true,
        opacity: 1.0,
    };
    scene.add_node(root, rounded);

    // Add a flat Rect for comparison
    let flat = SceneNode {
        content: NodeContent::Styled {
            style: Box::new(
                style_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4()),
            ),
        },
        transform: Transform2D::identity(),
        bounds: plat_core::Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        },
        children: vec![],
        parent: None,
        visible: true,
        opacity: 1.0,
    };
    scene.add_node(root, flat);

    let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);
    assert_eq!(instances.len(), 2);

    // Rounded rect should preserve corner_radius (uniform radii)
    let rounded_inst = instances
        .iter()
        .find(|i| i.corner_radii[0] > 0.0)
        .expect("Rounded instance not found");
    assert_eq!(rounded_inst.corner_radii, [12.0; 4]); // uniform radii
    assert_eq!(rounded_inst.pos, [30.0, 30.0]);

    // Flat rect should have 0.0
    let flat_inst = instances
        .iter()
        .find(|i| i.color[0] > 0.9)
        .expect("Flat instance not found");
    assert_eq!(flat_inst.corner_radii, [0.0; 4]);
}

#[test]
fn test_text_parallel_shaping_below_threshold() {
    // Create scene with text nodes below parallel threshold
    let mut scene = Scene::new();
    let root = scene.root();

    // Add 4 text nodes (below TEXT_PARALLEL_THRESHOLD of 8)
    for i in 0..4 {
        let text_node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    crate::VisualStyle::new()
                        .solid_fill(Color::rgba(1.0, 1.0, 1.0, 1.0).as_vec4())
                        .text(crate::TextContent::new(format!("Text {}", i), 16.0)),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 0.0,
                y: (i * 20) as f32,
                width: 100.0,
                height: 20.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        scene.add_node(root, text_node);
    }

    let (_, text_nodes, _) = instance_collector::collect_instances_for_tests(&scene);
    assert_eq!(text_nodes.len(), 4);
    // Sequential path should be taken
}

#[test]
fn test_text_parallel_shaping_above_threshold() {
    // Create scene with text nodes above parallel threshold
    let mut scene = Scene::new();
    let root = scene.root();

    // Add 10 text nodes (above TEXT_PARALLEL_THRESHOLD of 8)
    for i in 0..10 {
        let text_node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    crate::VisualStyle::new()
                        .solid_fill(Color::rgba(1.0, 1.0, 1.0, 1.0).as_vec4())
                        .text(crate::TextContent::new(format!("Text {}", i), 16.0)),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 0.0,
                y: (i * 20) as f32,
                width: 100.0,
                height: 20.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        scene.add_node(root, text_node);
    }

    let (_, text_nodes, _) = instance_collector::collect_instances_for_tests(&scene);
    assert_eq!(text_nodes.len(), 10);
    // Parallel path should be taken
}

#[test]
fn test_text_shaping_with_empty_strings() {
    // Create scene with mix of empty and non-empty text
    let mut scene = Scene::new();
    let root = scene.root();

    for i in 0..5 {
        let text = if i % 2 == 0 {
            String::new()
        } else {
            format!("Text {}", i)
        };

        let text_node = SceneNode {
            content: NodeContent::Styled {
                style: Box::new(
                    crate::VisualStyle::new()
                        .solid_fill(Color::rgba(1.0, 1.0, 1.0, 1.0).as_vec4())
                        .text(crate::TextContent::new(text, 16.0)),
                ),
            },
            transform: Transform2D::identity(),
            bounds: plat_core::Rect {
                x: 0.0,
                y: (i * 20) as f32,
                width: 100.0,
                height: 20.0,
            },
            children: vec![],
            parent: None,
            visible: true,
            opacity: 1.0,
        };
        scene.add_node(root, text_node);
    }

    let (_, text_nodes, _) = instance_collector::collect_instances_for_tests(&scene);
    assert_eq!(text_nodes.len(), 5);
    // Empty strings should be filtered out during shaping
}

#[test]
fn test_apply_text_fill_solid_sets_color_and_solid_flag() {
    let mut instance =
        PrimitiveInstance::glyph([0.0, 0.0], [10.0, 12.0], [1.0, 1.0, 1.0, 1.0], [0.0; 4]);
    let color = glam::Vec4::new(0.2, 0.4, 0.6, 0.8);

    apply_text_fill_to_glyph(&mut instance, TextFill::Solid(color));

    assert_eq!(instance.color, color.to_array());
    assert_eq!(
        instance.flags & FLAG_FILL_TYPE_MASK,
        0,
        "solid fill type bits should be 0"
    );
}

#[test]
fn test_apply_text_fill_gradient_sets_flag_and_gradient_index() {
    let mut instance =
        PrimitiveInstance::glyph([0.0, 0.0], [10.0, 12.0], [1.0, 1.0, 1.0, 1.0], [0.0; 4]);

    apply_text_fill_to_glyph(
        &mut instance,
        TextFill::Gradient {
            param_index: 42,
            fill_type: 3,
            opacity: 0.65,
            text_bounds: [10.0, 20.0, 100.0, 40.0],
        },
    );

    assert_eq!(
        instance.flags & FLAG_FILL_TYPE_MASK,
        3,
        "gradient fill type bits should be set"
    );
    assert_eq!(instance.gradient_params[0], 42.0);
    assert_eq!(instance.gradient_params[1], 10.0);
    assert_eq!(instance.gradient_params[2], 20.0);
    assert_eq!(instance.gradient_params[3], 100.0);
    assert_eq!(instance.stroke_params[0], 40.0);
    assert!((instance.color[3] - 0.65).abs() < 1e-6);
    assert_eq!(instance.corner_radii, [0.0; 4]);
}

#[test]
fn test_create_node_instances_emits_fill_stroke_and_shadow_for_non_text_style() {
    let node = SceneNode {
        content: NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new()
                    .fill(style_engine::Paint::Linear(style_engine::LinearGradient {
                        start: glam::Vec2::new(0.0, 0.5),
                        end: glam::Vec2::new(1.0, 0.5),
                        stops: vec![
                            style_engine::ColorStop::new(0.0, glam::Vec4::new(1.0, 0.0, 0.0, 1.0)),
                            style_engine::ColorStop::new(1.0, glam::Vec4::new(0.0, 0.0, 1.0, 1.0)),
                        ],
                    }))
                    .stroke(style_engine::StrokeStyle::solid(
                        style_engine::Paint::solid(glam::Vec4::new(1.0, 1.0, 1.0, 1.0)),
                        2.0,
                        style_engine::StrokeAlign::Inside,
                    ))
                    .drop_shadow(
                        glam::Vec2::new(2.0, 2.0),
                        6.0,
                        glam::Vec4::new(0.0, 0.0, 0.0, 0.3),
                    ),
            ),
        },
        transform: Transform2D::identity(),
        bounds: plat_core::Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        },
        children: vec![],
        parent: None,
        visible: true,
        opacity: 1.0,
    };

    let instances = create_node_instances(&node);

    assert_eq!(instances.len(), 3, "expected shadow + fill + stroke");
    assert!(
        instances
            .iter()
            .any(|i| (i.flags & pipelines::primitive_instance::FLAG_IS_SHADOW) != 0),
        "expected one shadow instance"
    );
    assert!(
        instances.iter().any(|i| (i.flags & (1 << 4)) != 0),
        "expected one stroke instance"
    );
}

#[test]
fn test_collect_instances_routes_fill_geometry_to_path_batches() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut path = style_engine::VectorPath::new();
    path.move_to(glam::Vec2::new(0.0, 0.0));
    path.line_to(glam::Vec2::new(80.0, 0.0));
    path.line_to(glam::Vec2::new(40.0, 60.0));
    path.close();

    let node = SceneNode {
        content: NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new()
                    .solid_fill(glam::Vec4::new(0.2, 0.8, 0.4, 1.0))
                    .fill_geometry(vec![path]),
            ),
        },
        transform: Transform2D::identity(),
        bounds: plat_core::Rect {
            x: 100.0,
            y: 120.0,
            width: 80.0,
            height: 60.0,
        },
        children: vec![],
        parent: None,
        visible: true,
        opacity: 0.75,
    };
    scene.add_node(root, node);

    let (instances, _text_nodes, path_batches) =
        instance_collector::collect_instances_for_tests(&scene);
    assert!(
        instances.is_empty(),
        "path-only node should not emit primitive rect instances"
    );
    assert_eq!(path_batches.len(), 1, "expected one tessellated path batch");
    assert!(!path_batches[0].mesh.indices.is_empty());
}

#[test]
fn test_collect_instances_adds_stroke_geometry_batches() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut path = style_engine::VectorPath::new();
    path.move_to(glam::Vec2::new(0.0, 0.0));
    path.line_to(glam::Vec2::new(60.0, 0.0));
    path.line_to(glam::Vec2::new(30.0, 40.0));
    path.close();

    let node = SceneNode {
        content: NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new()
                    .solid_fill(glam::Vec4::new(0.2, 0.8, 0.4, 1.0))
                    .fill_geometry(vec![path])
                    .stroke(style_engine::StrokeStyle::solid(
                        style_engine::Paint::solid(glam::Vec4::new(1.0, 1.0, 1.0, 1.0)),
                        3.0,
                        style_engine::StrokeAlign::Center,
                    )),
            ),
        },
        transform: Transform2D::identity(),
        bounds: plat_core::Rect {
            x: 50.0,
            y: 50.0,
            width: 60.0,
            height: 40.0,
        },
        children: vec![],
        parent: None,
        visible: true,
        opacity: 1.0,
    };
    scene.add_node(root, node);

    let (instances, _text_nodes, path_batches) =
        instance_collector::collect_instances_for_tests(&scene);
    assert!(
        instances.is_empty(),
        "geometry node should route through path batches"
    );
    assert_eq!(path_batches.len(), 2, "expected fill + stroke path batches");
}

#[test]
fn test_style_opacity_multiplies_node_opacity_for_path_batches() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut path = style_engine::VectorPath::new();
    path.move_to(glam::Vec2::new(0.0, 0.0));
    path.line_to(glam::Vec2::new(60.0, 0.0));
    path.line_to(glam::Vec2::new(30.0, 40.0));
    path.close();

    let node = SceneNode {
        content: NodeContent::Styled {
            style: Box::new(
                crate::VisualStyle::new()
                    .solid_fill(glam::Vec4::new(0.2, 0.8, 0.4, 1.0))
                    .opacity(0.5)
                    .fill_geometry(vec![path]),
            ),
        },
        transform: Transform2D::identity(),
        bounds: plat_core::Rect {
            x: 10.0,
            y: 10.0,
            width: 60.0,
            height: 40.0,
        },
        children: vec![],
        parent: None,
        visible: true,
        opacity: 0.8,
    };
    scene.add_node(root, node);

    let (_instances, _text_nodes, path_batches) =
        instance_collector::collect_instances_for_tests(&scene);
    assert_eq!(path_batches.len(), 1);
    assert!(
        (path_batches[0].opacity - 0.4).abs() < 1e-6,
        "expected path opacity 0.4 from node.opacity(0.8) * style.opacity(0.5), got {}",
        path_batches[0].opacity
    );
}

#[test]
fn test_clips_content_partially_clips_child_primitive_bounds() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut clip_parent = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            crate::VisualStyle::new()
                .solid_fill(Color::rgba(0.2, 0.2, 0.2, 1.0).as_vec4())
                .clips_content(true),
        ),
    });
    clip_parent.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
    let parent_id = scene.add_node(root, clip_parent);

    let mut child = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            crate::VisualStyle::new().solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4()),
        ),
    });
    child.bounds = plat_core::Rect::new(80.0, 10.0, 40.0, 20.0);
    scene.add_node(parent_id, child);

    let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);
    let clipped_child = instances
        .iter()
        .find(|i| (i.color[0] - 1.0).abs() < 1e-6)
        .expect("expected red child instance");

    assert_eq!(clipped_child.pos[0], 80.0);
    assert_eq!(
        clipped_child.size[0], 20.0,
        "child width should be clipped to parent bounds"
    );
}

#[test]
fn test_clips_content_skips_child_fully_outside_clip_bounds() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut clip_parent = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            crate::VisualStyle::new()
                .solid_fill(Color::rgba(0.2, 0.2, 0.2, 1.0).as_vec4())
                .clips_content(true),
        ),
    });
    clip_parent.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
    let parent_id = scene.add_node(root, clip_parent);

    let mut child = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            crate::VisualStyle::new().solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4()),
        ),
    });
    child.bounds = plat_core::Rect::new(120.0, 10.0, 40.0, 20.0);
    scene.add_node(parent_id, child);

    let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);
    let red_count = instances
        .iter()
        .filter(|i| (i.color[0] - 1.0).abs() < 1e-6)
        .count();

    assert_eq!(
        red_count, 0,
        "child outside clipping parent should not emit instances"
    );
}

#[test]
fn test_image_fill_rect_is_routed_to_path_batches() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut node = SceneNode::new(NodeContent::Styled {
        style: Box::new(crate::VisualStyle::new().fill(style_engine::Paint::Image(
            style_engine::ImageFill {
                image_id: style_engine::ImageId(7001),
                scale_mode: style_engine::ImageScaleMode::Fill,
                transform: None,
            },
        ))),
    });
    node.bounds = plat_core::Rect::new(20.0, 30.0, 140.0, 90.0);
    scene.add_node(root, node);

    let (instances, _, path_batches) = instance_collector::collect_instances_for_tests(&scene);
    assert!(
        instances.is_empty(),
        "image-fill rect should not emit primitive fallback instances"
    );
    assert_eq!(
        path_batches.len(),
        1,
        "expected one path batch for image fill"
    );
    assert!(matches!(
        path_batches[0].paint,
        style_engine::Paint::Image(_)
    ));
}

#[test]
fn test_mask_node_clips_subsequent_sibling_bounds() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut mask = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            crate::VisualStyle::new()
                .solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4())
                .is_mask(true),
        ),
    });
    mask.bounds = plat_core::Rect::new(0.0, 0.0, 40.0, 40.0);
    scene.add_node(root, mask);

    let mut masked = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            crate::VisualStyle::new().solid_fill(Color::rgba(0.0, 1.0, 0.0, 1.0).as_vec4()),
        ),
    });
    masked.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 40.0);
    scene.add_node(root, masked);

    let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);
    let green = instances
        .iter()
        .find(|i| (i.color[1] - 1.0).abs() < 1e-6 && (i.color[0]).abs() < 1e-6)
        .expect("expected green masked instance");
    assert_eq!(green.size[0], 40.0);
    assert_eq!(green.size[1], 40.0);
}

#[test]
fn test_mask_node_does_not_clip_preceding_sibling() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut before = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            crate::VisualStyle::new().solid_fill(Color::rgba(0.0, 0.0, 1.0, 1.0).as_vec4()),
        ),
    });
    before.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 40.0);
    scene.add_node(root, before);

    let mut mask = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            crate::VisualStyle::new()
                .solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4())
                .is_mask(true),
        ),
    });
    mask.bounds = plat_core::Rect::new(0.0, 0.0, 40.0, 40.0);
    scene.add_node(root, mask);

    let mut after = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            crate::VisualStyle::new().solid_fill(Color::rgba(0.0, 1.0, 0.0, 1.0).as_vec4()),
        ),
    });
    after.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 40.0);
    scene.add_node(root, after);

    let (instances, _, _) = instance_collector::collect_instances_for_tests(&scene);
    let blue = instances
        .iter()
        .find(|i| (i.color[2] - 1.0).abs() < 1e-6 && (i.color[0]).abs() < 1e-6)
        .expect("expected blue pre-mask instance");
    let green = instances
        .iter()
        .find(|i| (i.color[1] - 1.0).abs() < 1e-6 && (i.color[0]).abs() < 1e-6)
        .expect("expected green post-mask instance");
    assert_eq!(
        blue.size[0], 100.0,
        "preceding sibling should remain unclipped"
    );
    assert_eq!(
        green.size[0], 40.0,
        "sibling after mask should be clipped by mask bounds"
    );
}

#[test]
fn test_style_requires_multipass_detects_blend_and_blur() {
    let blend_only = style_engine::VisualStyle::new().blend_mode(style_engine::BlendMode::Multiply);
    assert!(effects::style_requires_multipass(&blend_only));

    let layer_blur = style_engine::VisualStyle::new().effect(style_engine::Effect::LayerBlur(
        style_engine::LayerBlur {
            radius: 12.0,
            visible: true,
        },
    ));
    assert!(effects::style_requires_multipass(&layer_blur));

    let normal =
        style_engine::VisualStyle::new().solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0).as_vec4());
    assert!(!effects::style_requires_multipass(&normal));
}

#[test]
fn test_collect_multipass_node_ids_preserves_visual_order() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut normal = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            style_engine::VisualStyle::new().solid_fill(Color::rgba(0.2, 0.2, 0.2, 1.0).as_vec4()),
        ),
    });
    normal.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
    scene.add_node(root, normal);

    let mut blend = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            style_engine::VisualStyle::new()
                .solid_fill(Color::rgba(1.0, 0.0, 0.0, 0.8).as_vec4())
                .blend_mode(style_engine::BlendMode::Screen),
        ),
    });
    blend.bounds = plat_core::Rect::new(10.0, 10.0, 40.0, 40.0);
    let blend_id = scene.add_node(root, blend);

    let mut blur =
        SceneNode::new(NodeContent::Styled {
            style: Box::new(style_engine::VisualStyle::new().effect(
                style_engine::Effect::LayerBlur(style_engine::LayerBlur {
                    radius: 8.0,
                    visible: true,
                }),
            )),
        });
    blur.bounds = plat_core::Rect::new(20.0, 20.0, 30.0, 30.0);
    let blur_id = scene.add_node(root, blur);

    let ids = multipass_executor::collect_multipass_node_ids(&scene);
    assert_eq!(ids, vec![blend_id, blur_id]);
}

#[test]
fn test_collect_instances_without_multipass_skips_effect_nodes() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut normal = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            style_engine::VisualStyle::new().solid_fill(Color::rgba(0.0, 1.0, 0.0, 1.0).as_vec4()),
        ),
    });
    normal.bounds = plat_core::Rect::new(0.0, 0.0, 20.0, 20.0);
    scene.add_node(root, normal);

    let mut blend = SceneNode::new(NodeContent::Styled {
        style: Box::new(
            style_engine::VisualStyle::new()
                .solid_fill(Color::rgba(1.0, 0.0, 0.0, 0.8).as_vec4())
                .blend_mode(style_engine::BlendMode::Multiply),
        ),
    });
    blend.bounds = plat_core::Rect::new(30.0, 0.0, 20.0, 20.0);
    scene.add_node(root, blend);

    let (instances, _text_nodes, _path_batches) =
        instance_collector::collect_instances_without_multipass_for_tests(&scene);
    assert_eq!(instances.len(), 1);
    assert!((instances[0].color[1] - 1.0).abs() < 1e-6);
}

#[test]
fn test_classify_scene_effect_kinds_detects_offscreen_effects() {
    let mut scene = Scene::new();
    let root = scene.root();

    let mut node =
        SceneNode::new(NodeContent::Styled {
            style: Box::new(style_engine::VisualStyle::new().effect(
                style_engine::Effect::LayerBlur(style_engine::LayerBlur {
                    radius: 12.0,
                    visible: true,
                }),
            )),
        });
    node.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
    scene.add_node(root, node);

    let kinds = multipass_executor::classify_scene_effect_kinds(&scene);
    assert!(kinds.contains(&effects::EffectPassKind::OffscreenLayer));
    assert!(kinds.contains(&effects::EffectPassKind::BlurHorizontal));
    assert!(kinds.contains(&effects::EffectPassKind::BlurVertical));
}
