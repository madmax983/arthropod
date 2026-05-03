use flux_state::{Runtime, Signal};
use render_engine::{NodeContent, Scene};
use widget_core::{ProgressBar, Widget, WidgetContext};

#[test]
fn test_progress_bar_creates_node() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime, 0.5);
    let (read, _) = signal.split();
    let pb = ProgressBar::new(read);

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let node_id = pb.build(&mut ctx);

    let scene = ctx.into_scene();
    let track_node = scene.get_node(node_id).unwrap();

    // Check track node is styled
    assert!(matches!(track_node.content, NodeContent::Styled { .. }));

    // Should have one child (the fill)
    assert_eq!(track_node.children.len(), 1);
    let fill_node = scene.get_node(track_node.children[0]).unwrap();
    assert!(matches!(fill_node.content, NodeContent::Styled { .. }));
}

#[test]
fn test_progress_bar_height() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime, 0.5);
    let (read, _) = signal.split();
    let pb = ProgressBar::new(read).height(16.0);

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let node_id = pb.build(&mut ctx);

    let layouts = ctx.layout_styles();
    let track_layout = layouts.get(&node_id).unwrap();
    assert_eq!(track_layout.height, Some(16.0));
}
