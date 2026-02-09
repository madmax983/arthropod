//! Arthropod MCP standalone server binary.
//!
//! Provides an MCP server for testing and automation via Model Context Protocol.
//!
//! ## Usage
//!
//! ```bash
//! # Start the server (communicates via stdin/stdout)
//! cargo run --bin arthropod-mcp
//! ```

use arthropod_mcp::{ArthropodServer, MCP_PORT, McpFrameworkContext, start_tcp_server};
use render_engine::{Color, NodeContent, SceneNode};
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (critical for stdio protocol)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Arthropod MCP server (stdio mode)");

    // Start TCP server for live app connections
    let connected_app = start_tcp_server();
    tracing::info!(
        "TCP server listening on localhost:{} for app connections",
        MCP_PORT
    );

    // Create framework context (used as fallback when no app is connected)
    let mut context = McpFrameworkContext::new();

    // Setup a basic test scene for exploration
    setup_test_scene(&mut context);

    // Create server with context and live app connection
    let server = ArthropodServer::with_live_app(context, connected_app);

    tracing::info!("MCP server ready - listening on stdin");

    // Create stdio transport
    let transport = (stdin(), stdout());

    // Serve using rmcp SDK
    let server_handle = server.serve(transport).await?;

    // Wait for server to complete
    let _quit_reason = server_handle.waiting().await?;

    tracing::info!("MCP server shutting down");

    Ok(())
}

/// Setup a basic test scene with some rectangles for demonstration
fn setup_test_scene(ctx: &mut McpFrameworkContext) {
    let (background, rect1, rect2) = {
        let scene = ctx.scene_mut();
        let root = scene.root();

        // Background rectangle
        let mut bg_node = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                render_engine::VisualStyle::new()
                    .solid_fill(Color::rgba(0.1, 0.1, 0.1, 1.0).as_vec4()),
            ),
        });
        bg_node.bounds = plat_core::Rect::new(0.0, 0.0, 800.0, 600.0);
        bg_node.visible = true;
        bg_node.opacity = 1.0;
        let background = scene.add_node(root, bg_node);

        // Red rectangle
        let mut rect1_node = SceneNode::new(NodeContent::Styled {
            style: Box::new(render_engine::VisualStyle::new().solid_fill(Color::RED.as_vec4())),
        });
        rect1_node.bounds = plat_core::Rect::new(100.0, 100.0, 200.0, 150.0);
        rect1_node.visible = true;
        rect1_node.opacity = 1.0;
        let rect1 = scene.add_node(background, rect1_node);

        // Blue rounded rectangle
        let mut rect2_node = SceneNode::new(NodeContent::Styled {
            style: Box::new(
                render_engine::VisualStyle::new()
                    .solid_fill(Color::BLUE.as_vec4())
                    .corner_radius(8.0),
            ),
        });
        rect2_node.bounds = plat_core::Rect::new(400.0, 200.0, 250.0, 180.0);
        rect2_node.visible = true;
        rect2_node.opacity = 0.8;
        let rect2 = scene.add_node(background, rect2_node);

        (background, rect1, rect2)
    };

    // Register nodes with names for easy access
    ctx.register_node("background".to_string(), background);
    ctx.register_node("rect1".to_string(), rect1);
    ctx.register_node("rect2".to_string(), rect2);

    let node_count = ctx.scene().nodes().count();
    tracing::info!("Test scene created with {} nodes", node_count);
}
