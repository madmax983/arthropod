use flux_state::{Runtime, Signal};
use render_engine::{NodeContent, Scene};
use widget_core::{Checkbox, Widget, WidgetContext};

#[test]
fn test_checkbox_creates_node_without_label() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime, false);
    let checkbox = Checkbox::new(signal);

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let node_id = checkbox.build(&mut ctx);

    let scene = ctx.into_scene();
    let node = scene.get_node(node_id).unwrap();

    assert!(matches!(node.content, NodeContent::Styled { .. }));
    assert_eq!(node.children.len(), 1);
}

#[test]
fn test_checkbox_creates_node_with_label() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime, false);
    let checkbox = Checkbox::new(signal).label("Accept terms");

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let node_id = checkbox.build(&mut ctx);

    let scene = ctx.into_scene();
    let node = scene.get_node(node_id).unwrap();

    assert!(matches!(node.content, NodeContent::Empty));
    assert_eq!(node.children.len(), 2);
}

#[test]
fn test_checkbox_click_toggles_state() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), false);
    let (read, _) = signal.clone().split();
    let checkbox = Checkbox::new(signal);

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let box_id = checkbox.build(&mut ctx);

    let callback = ctx.clickables().get(&box_id).unwrap();
    callback();

    assert!(read.get_untracked());

    callback();
    assert!(!read.get_untracked());
}

#[test]
fn test_checkbox_disabled() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), false);
    let checkbox = Checkbox::new(signal).disabled(true);

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let box_id = checkbox.build(&mut ctx);

    assert!(ctx.clickables().get(&box_id).is_none());
}

#[test]
fn test_checkbox_label_click_toggles_state() {
    let runtime = Runtime::new();
    let signal = Signal::new(runtime.clone(), false);
    let (read, _) = signal.clone().split();
    let checkbox = Checkbox::new(signal).label("Click me");

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let row_id = checkbox.build(&mut ctx);

    let node = ctx.scene().get_node(row_id).unwrap();
    let label_id = node.children[1];

    let callback = ctx.clickables().get(&label_id).unwrap();
    callback();

    assert!(read.get_untracked());
}
