use crossterm::event::{Event, KeyCode, KeyEvent};
use plat_core::Rect;
use render_engine::{
    Color, NodeContent, Paint, Scene, SceneNode, StrokeStyle, TextContent, VisualStyle,
};
use style_engine::StrokeAlign;
use tui_renderer::TuiRunner;

fn main() -> std::io::Result<()> {
    let mut scene = Scene::new();
    let root = scene.root();

    // Add a container (Blue box with white border)
    let container = scene.add_node(
        root,
        SceneNode::new(NodeContent::Styled {
            style: Box::new(VisualStyle::new().solid_fill(Color::BLUE.as_vec4()).stroke(
                StrokeStyle::solid(
                    Paint::solid(Color::WHITE.as_vec4()),
                    1.0,
                    StrokeAlign::Inside,
                ),
            )),
        }),
    );
    if let Some(node) = scene.get_mut(container) {
        node.bounds = Rect::new(10.0, 5.0, 40.0, 10.0);
    }

    // Add text inside
    let text = scene.add_node(
        container,
        SceneNode::new(NodeContent::Styled {
            style: Box::new(
                VisualStyle::new()
                    .text(TextContent::new("Hello TUI!", 16.0))
                    .solid_fill(Color::WHITE.as_vec4()), // Text color
            ),
        }),
    );
    if let Some(node) = scene.get_mut(text) {
        node.bounds = Rect::new(12.0, 7.0, 20.0, 3.0);
    }

    // Run the TUI
    TuiRunner::run(scene, |scene, event| {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                return false;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('m'),
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                // Move container right
                if let Some(node) = scene.get_mut(container) {
                    node.bounds.x += 1.0;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                // Move container left
                if let Some(node) = scene.get_mut(container) {
                    node.bounds.x -= 1.0;
                }
            }
            _ => {}
        }
        true
    })
}
