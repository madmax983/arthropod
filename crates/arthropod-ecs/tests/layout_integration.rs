use arthropod_ecs::{
    components::{LayoutConstraintsResource, LayoutStyle},
    FrameworkContext,
};
use layout_engine::{FlexDirection, FlexStyle, LayoutConstraints};
use render_engine::{Color, NodeContent, Scene, SceneNode, VisualStyle};

#[test]
fn test_layout_system_integration() {
    let mut ctx = FrameworkContext::new();

    // 1. Setup Scene: Root -> Child
    let (root, child) = {
        let mut scene = ctx.world_mut().resource_mut::<Scene>();
        let root = scene.root();

        // Add child
        let child = scene.add_node(
            root,
            SceneNode::new(NodeContent::Styled {
                style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
            }),
        );

        (root, child)
    };

    // 2. Add LayoutConstraintsResource (simulate window size 800x600)
    ctx.world_mut()
        .insert_resource(LayoutConstraintsResource(LayoutConstraints {
            max_width: Some(800.0),
            max_height: Some(600.0),
            ..Default::default()
        }));

    // 3. Add LayoutStyle components
    // Root: Column, Full Size
    ctx.spawn(root).insert(LayoutStyle(FlexStyle {
        direction: FlexDirection::Column,
        width: Some(800.0),
        height: Some(600.0),
        ..Default::default()
    }));

    // Child: Fixed Size 100x100
    ctx.spawn(child).insert(LayoutStyle(FlexStyle {
        width: Some(100.0),
        height: Some(100.0),
        ..Default::default()
    }));

    // 4. Run Update (triggers layout_system)
    ctx.update();

    // 5. Verify Bounds
    let scene = ctx.world().resource::<Scene>();
    let root_node = scene.get_node(root).unwrap();
    let child_node = scene.get_node(child).unwrap();

    assert_eq!(root_node.bounds.width, 800.0);
    assert_eq!(root_node.bounds.height, 600.0);
    assert_eq!(child_node.bounds.width, 100.0);
    assert_eq!(child_node.bounds.height, 100.0);
}

#[test]
fn test_layout_updates_on_constraint_change() {
    let mut ctx = FrameworkContext::new();
    let root = ctx.world().resource::<Scene>().root();

    // Root fills available space
    ctx.spawn(root).insert(LayoutStyle(FlexStyle {
        flex_grow: 1.0,
        ..Default::default()
    }));

    // Initial Constraint: 400x300
    ctx.world_mut()
        .insert_resource(LayoutConstraintsResource(LayoutConstraints {
            max_width: Some(400.0),
            max_height: Some(300.0),
            ..Default::default()
        }));

    ctx.update();

    {
        let scene = ctx.world().resource::<Scene>();
        let root_node = scene.get_node(root).unwrap();
        assert_eq!(root_node.bounds.width, 400.0);
        assert_eq!(root_node.bounds.height, 300.0);
    }

    // Update Constraint: 800x600
    if let Some(mut constraints) = ctx
        .world_mut()
        .get_resource_mut::<LayoutConstraintsResource>()
    {
        constraints.0.max_width = Some(800.0);
        constraints.0.max_height = Some(600.0);
    }

    ctx.update();

    {
        let scene = ctx.world().resource::<Scene>();
        let root_node = scene.get_node(root).unwrap();
        assert_eq!(root_node.bounds.width, 800.0);
        assert_eq!(root_node.bounds.height, 600.0);
    }
}
