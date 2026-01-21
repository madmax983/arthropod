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

// =============================================================================
// Widget Integration Tests (TDD - written first, implementation follows)
// =============================================================================

#[test]
fn test_integrate_widgets_transfers_layout_styles() {
    // Setup: Create WidgetContext with a widget that has layout style
    use layout_engine::FlexStyle;
    use std::collections::HashMap;
    use widget_core::WidgetContext;

    let mut widget_ctx = WidgetContext::new_test();

    // Create a node in widget context and set layout style
    let widget_node = widget_ctx.create_node(
        widget_ctx.root(),
        NodeContent::Rect { color: Color::RED },
    );
    widget_ctx.set_layout_style(widget_node, FlexStyle::default());

    // Create app
    let mut app = AppBuilder::new()
        .with_window_config(WindowConfig::default())
        .build_headless()
        .expect("Failed to create app");

    // Copy widget node to app scene and create mapping
    let app_node = {
        let mut scene = app.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Rect { color: Color::RED },
                transform: Transform2D::identity(),
                bounds: Rect::default(),
                children: Vec::new(),
                visible: true,
                opacity: 1.0,
            },
        )
    };

    // Spawn entity for the node
    app.spawn(app_node).insert(Renderable);

    // Create node mapping (widget NodeId -> app NodeId)
    let mut node_map = HashMap::new();
    node_map.insert(widget_node, app_node);

    // ACT: Integrate widgets into app
    app.integrate_widgets(&widget_ctx, &node_map);

    // ASSERT: LayoutStyle component should exist on the entity
    use arthropod_ecs::components::LayoutStyle;
    let has_layout = app
        .world_mut()
        .query::<(&SceneNodeRef, &LayoutStyle)>()
        .iter(app.world())
        .any(|(scene_ref, _)| scene_ref.0 == app_node);

    assert!(has_layout, "Entity should have LayoutStyle component after integration");
}

#[test]
fn test_integrate_widgets_transfers_clickables() {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use widget_core::WidgetContext;

    let mut widget_ctx = WidgetContext::new_test();

    // Create a clickable node
    let widget_node = widget_ctx.create_node(
        widget_ctx.root(),
        NodeContent::Rect { color: Color::BLUE },
    );

    // Add clickable with a callback we can verify
    let clicked = Arc::new(AtomicBool::new(false));
    let clicked_clone = clicked.clone();
    widget_ctx.add_clickable(widget_node, Arc::new(move || {
        clicked_clone.store(true, Ordering::SeqCst);
    }));

    // Create app and copy node
    let mut app = AppBuilder::new()
        .with_window_config(WindowConfig::default())
        .build_headless()
        .expect("Failed to create app");

    let app_node = {
        let mut scene = app.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Rect { color: Color::BLUE },
                transform: Transform2D::identity(),
                bounds: Rect::default(),
                children: Vec::new(),
                visible: true,
                opacity: 1.0,
            },
        )
    };

    app.spawn(app_node).insert(Renderable);

    let mut node_map = HashMap::new();
    node_map.insert(widget_node, app_node);

    // ACT: Integrate widgets
    app.integrate_widgets(&widget_ctx, &node_map);

    // ASSERT: Clickable component should exist and callback should work
    use arthropod_ecs::components::Clickable;
    let clickable = app
        .world_mut()
        .query::<(&SceneNodeRef, &Clickable)>()
        .iter(app.world())
        .find(|(scene_ref, _)| scene_ref.0 == app_node)
        .map(|(_, c)| c.callback.clone());

    assert!(clickable.is_some(), "Entity should have Clickable component");

    // Invoke the callback and verify it works
    if let Some(callback) = clickable {
        callback();
        assert!(clicked.load(Ordering::SeqCst), "Clickable callback should have been invoked");
    }
}

#[test]
fn test_integrate_widgets_transfers_background_colors() {
    use glam::Vec4;
    use std::collections::HashMap;
    use widget_core::WidgetContext;

    let mut widget_ctx = WidgetContext::new_test();

    // Create a node with background color
    let widget_node = widget_ctx.create_node(
        widget_ctx.root(),
        NodeContent::Rect { color: Color::GREEN },
    );
    widget_ctx.set_background_color(widget_node, Vec4::new(0.5, 0.5, 0.5, 1.0));

    // Create app
    let mut app = AppBuilder::new()
        .with_window_config(WindowConfig::default())
        .build_headless()
        .expect("Failed to create app");

    let app_node = {
        let mut scene = app.world_mut().resource_mut::<Scene>();
        let root = scene.root();
        scene.add_node(
            root,
            SceneNode {
                content: NodeContent::Rect { color: Color::GREEN },
                transform: Transform2D::identity(),
                bounds: Rect::default(),
                children: Vec::new(),
                visible: true,
                opacity: 1.0,
            },
        )
    };

    app.spawn(app_node).insert(Renderable);

    let mut node_map = HashMap::new();
    node_map.insert(widget_node, app_node);

    // ACT
    app.integrate_widgets(&widget_ctx, &node_map);

    // ASSERT
    use arthropod_ecs::components::BackgroundColor;
    let bg_color = app
        .world_mut()
        .query::<(&SceneNodeRef, &BackgroundColor)>()
        .iter(app.world())
        .find(|(scene_ref, _)| scene_ref.0 == app_node)
        .map(|(_, bg)| bg.0);

    assert!(bg_color.is_some(), "Entity should have BackgroundColor component");
    let color = bg_color.unwrap();
    assert!((color.x - 0.5).abs() < 0.001, "Background color R should be 0.5");
}
