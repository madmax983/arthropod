use arthropod_ecs::{
    FrameworkContext, ReactiveColor, ReactiveOpacity, ReactiveTransform, Renderable,
};
use flux_state::{Runtime, Signal};
use plat_core::Rect;
use render_engine::{Color, NodeContent, Scene, SceneNode, Transform2D, Vec2};

#[test]
fn test_reactive_color_updates_scene_node() {
    // Setup runtime and signal
    let runtime = Runtime::new();
    let color_signal = Signal::new(runtime.clone(), Color::RED);
    let (read_signal, write_signal) = color_signal.split();

    // Setup scene with a rect node
    let mut scene = Scene::new();
    let node_id = scene.add_node(
        scene.root(),
        SceneNode {
            content: NodeContent::Rect {
                color: Color::GREEN,
            },
            transform: Transform2D::identity(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
        },
    );

    // Setup ECS context and spawn entity with ReactiveColor
    let mut ctx = FrameworkContext::new();
    ctx.spawn(node_id).insert(ReactiveColor::new(read_signal));

    // Update signal to BLUE
    write_signal.set(Color::BLUE);

    // Run update systems
    ctx.update(&mut scene);

    // Verify node color updated to BLUE
    let node = scene.get_node(node_id).unwrap();
    match &node.content {
        NodeContent::Rect { color } => {
            assert!(
                (color.r() - Color::BLUE.r()).abs() < 0.001
                    && (color.g() - Color::BLUE.g()).abs() < 0.001
                    && (color.b() - Color::BLUE.b()).abs() < 0.001
                    && (color.a() - Color::BLUE.a()).abs() < 0.001,
                "Color should be updated to BLUE"
            );
        }
        _ => panic!("Expected Rect node"),
    }
}

#[test]
fn test_reactive_transform_updates_scene_node() {
    // Setup runtime and signal
    let runtime = Runtime::new();
    let initial_transform = Transform2D::identity();
    let new_transform = Transform2D::translate(10.0, 20.0);
    let transform_signal = Signal::new(runtime.clone(), initial_transform);
    let (read_signal, write_signal) = transform_signal.split();

    // Setup scene with a node
    let mut scene = Scene::new();
    let node_id = scene.add_node(
        scene.root(),
        SceneNode {
            content: NodeContent::Empty,
            transform: initial_transform,
            bounds: Rect::default(),
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
        },
    );

    // Setup ECS context with ReactiveTransform
    let mut ctx = FrameworkContext::new();
    ctx.spawn(node_id)
        .insert(ReactiveTransform::new(read_signal));

    // Update signal
    write_signal.set(new_transform);

    // Run update systems
    ctx.update(&mut scene);

    // Verify transform updated
    let node = scene.get_node(node_id).unwrap();
    // Verify transforms are equal by transforming a test point
    let test_point = Vec2::new(10.0, 20.0);
    let node_result = node.transform.transform_point(test_point);
    let new_result = new_transform.transform_point(test_point);

    assert!(
        (node_result.x - new_result.x).abs() < 0.001 && (node_result.y - new_result.y).abs() < 0.001,
        "Transform mismatch"
    );
}

#[test]
fn test_reactive_opacity_updates_scene_node() {
    // Setup runtime and signal
    let runtime = Runtime::new();
    let opacity_signal = Signal::new(runtime.clone(), 1.0);
    let (read_signal, write_signal) = opacity_signal.split();

    // Setup scene
    let mut scene = Scene::new();
    let node_id = scene.add_node(
        scene.root(),
        SceneNode {
            content: NodeContent::Empty,
            transform: Transform2D::identity(),
            bounds: Rect::default(),
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
        },
    );

    // Setup ECS context with ReactiveOpacity
    let mut ctx = FrameworkContext::new();
    ctx.spawn(node_id).insert(ReactiveOpacity::new(read_signal));

    // Update opacity to 0.5
    write_signal.set(0.5);

    // Run update systems
    ctx.update(&mut scene);

    // Verify opacity updated
    let node = scene.get_node(node_id).unwrap();
    assert_eq!(node.opacity, 0.5);
}

#[test]
fn test_collect_renderables_filters_invisible() {
    // Setup scene with visible and invisible nodes
    let mut scene = Scene::new();

    let visible_node = scene.add_node(
        scene.root(),
        SceneNode {
            content: NodeContent::Rect { color: Color::RED },
            transform: Transform2D::identity(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
        },
    );

    let hidden_node = scene.add_node(
        scene.root(),
        SceneNode {
            content: NodeContent::Rect { color: Color::BLUE },
            transform: Transform2D::identity(),
            bounds: Rect {
                x: 100.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            children: Vec::new(),
            visible: false, // Hidden!
            opacity: 1.0,
        },
    );

    // Setup ECS context
    let mut ctx = FrameworkContext::new();
    ctx.spawn(visible_node).insert(Renderable);
    ctx.spawn(hidden_node).insert(Renderable);

    // Render
    let instances = ctx.render(&scene);

    // Should only have 1 instance (the visible one)
    assert_eq!(instances.len(), 1, "Should only render visible node");
    assert_eq!(instances[0].pos, [0.0, 0.0]);
}

#[test]
fn test_collect_renderables_filters_zero_opacity() {
    // Setup scene with nodes of different opacity
    let mut scene = Scene::new();

    let opaque_node = scene.add_node(
        scene.root(),
        SceneNode {
            content: NodeContent::Rect { color: Color::RED },
            transform: Transform2D::identity(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
        },
    );

    let transparent_node = scene.add_node(
        scene.root(),
        SceneNode {
            content: NodeContent::Rect { color: Color::BLUE },
            transform: Transform2D::identity(),
            bounds: Rect {
                x: 100.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            children: Vec::new(),
            visible: true,
            opacity: 0.0, // Fully transparent!
        },
    );

    // Setup ECS
    let mut ctx = FrameworkContext::new();
    ctx.spawn(opaque_node).insert(Renderable);
    ctx.spawn(transparent_node).insert(Renderable);

    // Render
    let instances = ctx.render(&scene);

    // Should only render the opaque node
    assert_eq!(instances.len(), 1);
}

#[test]
fn test_collect_renderables_applies_opacity() {
    // Setup scene
    let mut scene = Scene::new();
    let node_id = scene.add_node(
        scene.root(),
        SceneNode {
            content: NodeContent::Rect {
                color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            },
            transform: Transform2D::identity(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            children: Vec::new(),
            visible: true,
            opacity: 0.5, // 50% opacity
        },
    );

    // Setup ECS
    let mut ctx = FrameworkContext::new();
    ctx.spawn(node_id).insert(Renderable);

    // Render
    let instances = ctx.render(&scene);

    // Verify opacity is applied to color alpha
    assert_eq!(instances.len(), 1);
    assert_eq!(
        instances[0].color[3], 0.5,
        "Alpha should be multiplied by opacity"
    );
}

#[test]
fn test_framework_context_update_and_render() {
    // Integration test: reactive updates + rendering
    let runtime = Runtime::new();
    let color_signal = Signal::new(runtime.clone(), Color::RED);
    let (read_signal, write_signal) = color_signal.split();

    let mut scene = Scene::new();
    let node_id = scene.add_node(
        scene.root(),
        SceneNode {
            content: NodeContent::Rect {
                color: Color::GREEN,
            },
            transform: Transform2D::identity(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
        },
    );

    let mut ctx = FrameworkContext::new();
    ctx.spawn(node_id)
        .insert(Renderable)
        .insert(ReactiveColor::new(read_signal));

    // Change color to BLUE
    write_signal.set(Color::BLUE);

    // Update (should poll reactive signals)
    ctx.update(&mut scene);

    // Render (should collect with new color)
    let instances = ctx.render(&scene);

    assert_eq!(instances.len(), 1);
    // Blue with full opacity
    assert_eq!(instances[0].color, [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn test_empty_nodes_not_rendered() {
    let mut scene = Scene::new();
    let empty_node = scene.add_node(
        scene.root(),
        SceneNode {
            content: NodeContent::Empty,
            transform: Transform2D::identity(),
            bounds: Rect::default(),
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
        },
    );

    let mut ctx = FrameworkContext::new();
    ctx.spawn(empty_node).insert(Renderable);

    let instances = ctx.render(&scene);

    assert_eq!(
        instances.len(),
        0,
        "Empty nodes should not generate render instances"
    );
}
