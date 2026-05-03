use render_engine::{NodeContent, Paint, Scene};
use style_engine::{ImageId, ImageScaleMode};
use widget_core::{Image, Widget, WidgetContext};

#[test]
fn test_image_creates_node_with_image_fill() {
    let image_id = ImageId(42);
    let image = Image::new(image_id);

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let node_id = image.build(&mut ctx);

    let scene = ctx.into_scene();
    let node = scene.get_node(node_id).unwrap();

    if let NodeContent::Styled { style } = &node.content {
        assert_eq!(style.fills.len(), 1);
        if let Paint::Image(fill) = &style.fills[0] {
            assert_eq!(fill.image_id, image_id);
            assert_eq!(fill.scale_mode, ImageScaleMode::Fit); // default
        } else {
            panic!("Image fill should be ImageFill");
        }
    } else {
        panic!("Image node should be styled");
    }
}

#[test]
fn test_image_scale_mode() {
    let image_id = ImageId(42);
    let image = Image::new(image_id).scale_mode(ImageScaleMode::Crop);

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let node_id = image.build(&mut ctx);

    let scene = ctx.into_scene();
    let node = scene.get_node(node_id).unwrap();

    if let NodeContent::Styled { style } = &node.content {
        if let Paint::Image(fill) = &style.fills[0] {
            assert_eq!(fill.scale_mode, ImageScaleMode::Crop);
        }
    }
}

#[test]
fn test_image_size() {
    let image = Image::new(ImageId(42)).width(100.0).height(200.0);

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let node_id = image.build(&mut ctx);

    let styles = ctx.layout_styles();
    let layout = styles.get(&node_id).unwrap();

    assert_eq!(layout.width, Some(100.0));
    assert_eq!(layout.height, Some(200.0));
}
