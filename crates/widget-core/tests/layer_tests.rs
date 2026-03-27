//! Layer system integration tests

use render_engine::node::NodeContent;
use widget_core::{Layer, WidgetContext};

#[test]
fn test_layer_ordering() {
    let layers = Layer::ALL;
    assert_eq!(layers[0], Layer::Content);
    assert_eq!(layers[1], Layer::Dropdown);
    assert_eq!(layers[2], Layer::Dialog);
    assert_eq!(layers[3], Layer::Notification);
    assert_eq!(layers[4], Layer::Tooltip);
    assert_eq!(layers[5], Layer::Cursor);
}

#[test]
fn test_layer_manager_creates_layer_nodes() {
    let ctx = WidgetContext::new_test();
    let content = ctx.layer_root(Layer::Content);
    let dropdown = ctx.layer_root(Layer::Dropdown);
    let dialog = ctx.layer_root(Layer::Dialog);
    let notification = ctx.layer_root(Layer::Notification);
    let tooltip = ctx.layer_root(Layer::Tooltip);
    let cursor = ctx.layer_root(Layer::Cursor);

    let ids = [content, dropdown, dialog, notification, tooltip, cursor];
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(
                ids[i],
                ids[j],
                "Layer {:?} and {:?} share NodeId",
                Layer::ALL[i],
                Layer::ALL[j]
            );
        }
    }
}

#[test]
fn test_root_returns_content_layer() {
    let ctx = WidgetContext::new_test();
    assert_eq!(ctx.root(), ctx.layer_root(Layer::Content));
    assert_ne!(ctx.root(), ctx.scene_root());
}

#[test]
fn test_add_to_layer() {
    let mut ctx = WidgetContext::new_test();
    let node_id = ctx.add_to_layer(Layer::Dialog, NodeContent::Empty);

    let dialog_root = ctx.layer_root(Layer::Dialog);
    let scene = ctx.scene();
    let dialog_node = scene.get_node(dialog_root).unwrap();
    assert!(dialog_node.children.contains(&node_id));
}

#[test]
fn test_move_to_layer() {
    let mut ctx = WidgetContext::new_test();
    let content_root = ctx.root();
    let node_id = ctx.create_node(content_root, NodeContent::Empty);

    ctx.move_to_layer(node_id, Layer::Tooltip);

    let tooltip_root = ctx.layer_root(Layer::Tooltip);
    let scene = ctx.scene();
    let tooltip_node = scene.get_node(tooltip_root).unwrap();
    assert!(tooltip_node.children.contains(&node_id));

    let content_node = scene.get_node(content_root).unwrap();
    assert!(!content_node.children.contains(&node_id));
}

#[test]
fn test_move_to_content() {
    let mut ctx = WidgetContext::new_test();
    let node_id = ctx.add_to_layer(Layer::Dialog, NodeContent::Empty);
    ctx.move_to_content(node_id);

    let content_root = ctx.root();
    let scene = ctx.scene();
    let content_node = scene.get_node(content_root).unwrap();
    assert!(content_node.children.contains(&node_id));
}

#[test]
fn test_layers_are_children_of_scene_root() {
    let ctx = WidgetContext::new_test();
    let scene = ctx.scene();
    let root = scene.get_node(ctx.scene_root()).unwrap();

    assert_eq!(root.children.len(), 6);
    for (i, layer) in Layer::ALL.iter().enumerate() {
        assert_eq!(root.children[i], ctx.layer_root(*layer));
    }
}

#[test]
fn test_widget_context_extensions() {
    let mut ctx = WidgetContext::new_test();
    assert!(ctx.get_extension::<String>().is_none());

    ctx.set_extension("hello".to_string());
    assert_eq!(ctx.get_extension::<String>().unwrap(), "hello");

    ctx.set_extension("world".to_string());
    assert_eq!(ctx.get_extension::<String>().unwrap(), "world");

    ctx.set_extension(42u32);
    assert_eq!(ctx.get_extension::<u32>().unwrap(), &42);
    assert_eq!(ctx.get_extension::<String>().unwrap(), "world");
}
