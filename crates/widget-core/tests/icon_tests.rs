use render_engine::{NodeContent, Scene};
use widget_core::{Icon, Widget, WidgetContext};

#[test]
fn test_icon_creates_node_with_text() {
    let icon = Icon::new("home");

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let node_id = icon.build(&mut ctx);

    let scene = ctx.into_scene();
    let node = scene.get_node(node_id).unwrap();

    if let NodeContent::Styled { style } = &node.content {
        if let Some(text) = &style.text {
            assert_eq!(text.text, "home");
        } else {
            panic!("Icon node should have text content");
        }
    } else {
        panic!("Icon node should be styled");
    }
}

#[test]
fn test_icon_size() {
    let icon = Icon::new("home").size(48.0);

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let node_id = icon.build(&mut ctx);

    let scene = ctx.into_scene();
    let node = scene.get_node(node_id).unwrap();

    if let NodeContent::Styled { style } = &node.content {
        if let Some(text) = &style.text {
            assert_eq!(text.font_size, 48.0);
        } else {
            panic!("Icon node should have text content");
        }
    }
}

#[test]
fn test_icon_color() {
    let color = render_engine::Color::RED;
    let icon = Icon::new("home").color(color);

    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    let node_id = icon.build(&mut ctx);

    let scene = ctx.into_scene();
    let node = scene.get_node(node_id).unwrap();

    if let NodeContent::Styled { style } = &node.content {
        assert_eq!(style.fills.len(), 1);
        if let render_engine::Paint::Solid(c) = style.fills[0] {
            assert_eq!(c, color.as_vec4());
        } else {
            panic!("Icon fill should be solid");
        }
    }
}

#[test]
fn test_icon_presets() {
    let scene = Scene::new();
    let mut ctx = WidgetContext::new(scene);

    // Test the predefined icons
    let search = Icon::search();
    let search_id = search.build(&mut ctx);

    let home = Icon::home();
    let home_id = home.build(&mut ctx);

    let settings = Icon::settings();
    let settings_id = settings.build(&mut ctx);

    let check = Icon::check();
    let check_id = check.build(&mut ctx);

    let close = Icon::close();
    let close_id = close.build(&mut ctx);

    let menu = Icon::menu();
    let menu_id = menu.build(&mut ctx);

    let scene = ctx.into_scene();

    let get_text = |id| {
        let node = scene.get_node(id).unwrap();
        if let NodeContent::Styled { style } = &node.content {
            style.text.as_ref().unwrap().text.clone()
        } else {
            panic!("Expected styled text content");
        }
    };

    assert_eq!(get_text(search_id), "\u{e8b6}");
    assert_eq!(get_text(home_id), "\u{e88a}");
    assert_eq!(get_text(settings_id), "\u{e8b8}");
    assert_eq!(get_text(check_id), "\u{e5ca}");
    assert_eq!(get_text(close_id), "\u{e5cd}");
    assert_eq!(get_text(menu_id), "\u{e5d2}");
}
