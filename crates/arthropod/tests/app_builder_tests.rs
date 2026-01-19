// TDD tests for arthropod::App builder API
//
// This test file defines the DESIRED API behavior:
// - App builder encapsulates Window, WgpuBackend, Runtime, FrameworkContext
// - All resources (Scene, Runtime, WgpuBackend) live in ECS World
// - Clean, ergonomic API for setting up Arthropod applications
// - No manual resource management - App handles everything

use arthropod::prelude::*;
use plat_core::{Size, WindowConfig};

#[test]
fn test_app_builder_creates_context() {
    // App should create FrameworkContext with Scene as Resource
    let config = WindowConfig {
        title: "Test App".to_string(),
        size: Size {
            width: 800,
            height: 600,
        },
        ..Default::default()
    };

    let app = AppBuilder::new()
        .with_window_config(config)
        .build_headless() // No real window for tests
        .expect("Failed to create app");

    // Should be able to access Scene from context
    let scene = app.world().resource::<Scene>();
    assert_eq!(scene.root().0, 0, "Root node should exist");
}

#[test]
fn test_app_builder_has_runtime() {
    // App should create Runtime and provide accessor
    let config = WindowConfig::default();

    let app = AppBuilder::new()
        .with_window_config(config)
        .build_headless()
        .expect("Failed to create app");

    // Should be able to access Runtime via accessor
    let runtime = app.runtime();
    // Runtime should be usable for creating signals
    let _signal = Signal::new(runtime.clone(), 42);
}

#[test]
fn test_app_spawn_with_reactive_components() {
    // Should be able to spawn entities with reactive components easily
    let config = WindowConfig::default();
    let mut app = AppBuilder::new()
        .with_window_config(config)
        .build_headless()
        .expect("Failed to create app");

    // Access Runtime to create signal
    let runtime = app.runtime().clone();
    let color_signal = Signal::new(runtime, Color::RED);
    let (read_signal, write_signal) = color_signal.split();

    // Add node to scene
    let node_id = {
        let mut scene = app.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Rect {
                    color: Color::GREEN,
                },
                transform: Transform2D::identity(),
                bounds: Rect::default(),
                children: Vec::new(),
                visible: true,
                opacity: 1.0,
            },
        )
    };

    // Spawn entity with reactive color
    app.spawn(node_id)
        .insert(Renderable)
        .insert(ReactiveColor::new(read_signal));

    // Update signal
    write_signal.set(Color::BLUE);

    // Run update
    app.update();

    // Verify color updated
    let scene = app.world().resource::<Scene>();
    let node = scene.get_node(node_id).unwrap();
    match &node.content {
        NodeContent::Rect { color } => {
            // Color doesn't implement PartialEq, so compare components
            assert!((color.r() - Color::BLUE.r()).abs() < 0.001);
            assert!((color.g() - Color::BLUE.g()).abs() < 0.001);
            assert!((color.b() - Color::BLUE.b()).abs() < 0.001);
        }
        _ => panic!("Expected Rect node"),
    }
}

#[test]
fn test_app_render_without_backend() {
    // Headless mode should support render() call but not actually render to GPU
    let config = WindowConfig::default();
    let mut app = AppBuilder::new()
        .with_window_config(config)
        .build_headless()
        .expect("Failed to create app");

    // Add a renderable node
    let node_id = {
        let mut scene = app.world_mut().resource_mut::<Scene>();
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
            },
        )
    };

    app.spawn(node_id).insert(Renderable);

    // Render should collect instances
    let instances = app.render();
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].color, [1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn test_app_provides_convenient_spawn() {
    // App should provide convenient spawn method
    let config = WindowConfig::default();
    let mut app = AppBuilder::new()
        .with_window_config(config)
        .build_headless()
        .expect("Failed to create app");

    let node_id = {
        let scene = app.world().resource::<Scene>();
        scene.root()
    };

    // Should be able to spawn directly on app
    let entity = app.spawn(node_id).insert(Renderable).id();

    // Entity should exist in World
    assert!(app.world().get_entity(entity).is_ok());
}
