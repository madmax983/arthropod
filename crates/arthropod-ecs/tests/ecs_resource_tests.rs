// Tests for the refactored API where Scene is a Resource in the World
//
// This test file defines the DESIRED API behavior:
// - FrameworkContext owns Scene as a Resource
// - No unsafe pointer juggling
// - Scene accessed via world.resource_mut::<Scene>()
// - update() and render() take no Scene arguments

use arthropod_ecs::{
    FrameworkContext, ReactiveColor, ReactiveOpacity, ReactiveTransform, Renderable,
};
use flux_state::{Runtime, Signal};
use plat_core::Rect;
use render_engine::{Color, NodeContent, Scene, SceneNode, Transform2D, Vec2};

#[test]
fn test_scene_owned_by_context() {
    // Scene should be created and owned by FrameworkContext
    let ctx = FrameworkContext::new();

    // Should be able to access Scene as a resource
    let scene = ctx.world().resource::<Scene>();
    assert!(scene.root().0 == 0, "Root node should exist");
}

#[test]
fn test_add_node_via_world_resource() {
    let mut ctx = FrameworkContext::new();

    // Access Scene through World to add nodes
    let node_id = {
        let mut scene = ctx.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        scene.add_node(
            root,
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
                parent: None,
            },
        )
    };

    // Verify node exists
    let scene = ctx.world().resource::<Scene>();
    assert!(scene.get_node(node_id).is_some());
}

#[test]
fn test_reactive_color_updates_without_passing_scene() {
    // Setup runtime and signal
    let runtime = Runtime::new();
    let color_signal = Signal::new(runtime.clone(), Color::RED);
    let (read_signal, write_signal) = color_signal.split();

    // Setup context (Scene is inside)
    let mut ctx = FrameworkContext::new();

    // Add node via World resource
    let node_id = {
        let mut scene = ctx.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        scene.add_node(
            root,
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
                parent: None,
            },
        )
    };

    // Spawn entity with ReactiveColor
    ctx.spawn(node_id).insert(ReactiveColor::new(read_signal));

    // Update signal to BLUE
    write_signal.set(Color::BLUE);

    // Run update systems - NO SCENE ARGUMENT
    ctx.update();

    // Verify node color updated to BLUE
    let scene = ctx.world().resource::<Scene>();
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
fn test_reactive_transform_without_passing_scene() {
    let runtime = Runtime::new();
    let initial_transform = Transform2D::identity();
    let new_transform = Transform2D::translate(10.0, 20.0);
    let transform_signal = Signal::new(runtime.clone(), initial_transform);
    let (read_signal, write_signal) = transform_signal.split();

    let mut ctx = FrameworkContext::new();

    let node_id = {
        let mut scene = ctx.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Empty,
                transform: initial_transform,
                bounds: Rect::default(),
                children: Vec::new(),
                visible: true,
                opacity: 1.0,
                parent: None,
            },
        )
    };

    ctx.spawn(node_id)
        .insert(ReactiveTransform::new(read_signal));

    write_signal.set(new_transform);

    // Update without passing scene
    ctx.update();

    // Verify transform updated
    let scene = ctx.world().resource::<Scene>();
    let node = scene.get_node(node_id).unwrap();
    let test_point = Vec2::new(10.0, 20.0);
    let node_result = node.transform.transform_point(test_point);
    let new_result = new_transform.transform_point(test_point);

    assert!(
        (node_result.x - new_result.x).abs() < 0.001
            && (node_result.y - new_result.y).abs() < 0.001,
        "Transform mismatch"
    );
}

#[test]
fn test_reactive_opacity_without_passing_scene() {
    let runtime = Runtime::new();
    let opacity_signal = Signal::new(runtime.clone(), 1.0);
    let (read_signal, write_signal) = opacity_signal.split();

    let mut ctx = FrameworkContext::new();

    let node_id = {
        let mut scene = ctx.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Empty,
                transform: Transform2D::identity(),
                bounds: Rect::default(),
                children: Vec::new(),
                visible: true,
                opacity: 1.0,
                parent: None,
            },
        )
    };

    ctx.spawn(node_id).insert(ReactiveOpacity::new(read_signal));

    write_signal.set(0.5);

    // Update without passing scene
    ctx.update();

    // Verify opacity updated
    let scene = ctx.world().resource::<Scene>();
    let node = scene.get_node(node_id).unwrap();
    assert_eq!(node.opacity, 0.5);
}

#[test]
fn test_render_without_passing_scene() {
    let mut ctx = FrameworkContext::new();

    let node_id = {
        let mut scene = ctx.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        scene.add_node(
            root,
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
                parent: None,
            },
        )
    };

    ctx.spawn(node_id).insert(Renderable);

    // Update (runs render collection) then extract instances
    ctx.update();
    let instances = ctx.render();

    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].color, [1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn test_integration_reactive_and_render_no_scene_args() {
    let runtime = Runtime::new();
    let color_signal = Signal::new(runtime.clone(), Color::RED);
    let (read_signal, write_signal) = color_signal.split();

    let mut ctx = FrameworkContext::new();

    let node_id = {
        let mut scene = ctx.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        scene.add_node(
            root,
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
                parent: None,
            },
        )
    };

    ctx.spawn(node_id)
        .insert(Renderable)
        .insert(ReactiveColor::new(read_signal));

    write_signal.set(Color::BLUE);

    // Update and render without scene arguments
    ctx.update();
    let instances = ctx.render();

    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].color, [0.0, 0.0, 1.0, 1.0]);
}
