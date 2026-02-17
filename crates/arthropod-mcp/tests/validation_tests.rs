use arthropod_mcp::context::McpFrameworkContext;
use arthropod_mcp::tools::Tool;
use arthropod_mcp::tools::scene::UpdateNodeTool;
use render_engine::{Color, NodeContent, SceneNode, VisualStyle};
use serde_json::json;

#[test]
fn test_update_node_negative_opacity_returns_error() {
    let mut ctx = McpFrameworkContext::new();
    let scene = ctx.scene_mut();

    // Create a node
    let mut node = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
    });
    node.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
    let id = scene.add_node(scene.root(), node);

    let tool = UpdateNodeTool;
    // Inject negative opacity
    let result = tool.execute(
        json!({
            "id": id.0,
            "updates": {
                "opacity": -1.0
            }
        }),
        &mut ctx,
    );

    assert!(result.is_err(), "Should return error for negative opacity");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid value for opacity")
    );
}

#[test]
fn test_update_node_large_opacity_returns_error() {
    let mut ctx = McpFrameworkContext::new();
    let scene = ctx.scene_mut();

    let mut node = SceneNode::new(NodeContent::Styled {
        style: Box::new(VisualStyle::new().solid_fill(Color::RED.as_vec4())),
    });
    node.bounds = plat_core::Rect::new(0.0, 0.0, 100.0, 100.0);
    let id = scene.add_node(scene.root(), node);

    let tool = UpdateNodeTool;
    let result = tool.execute(
        json!({
            "id": id.0,
            "updates": {
                "opacity": 2.0
            }
        }),
        &mut ctx,
    );

    assert!(result.is_err(), "Should return error for opacity > 1.0");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid value for opacity")
    );
}
