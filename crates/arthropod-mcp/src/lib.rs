#![allow(missing_docs)]
//! # Arthropod MCP Server
//!
//! The Model Context Protocol (MCP) server for the Arthropod GUI framework.
//!
//! This crate enables AI agents, testing tools, and external debuggers to interact with running
//! Arthropod applications. It provides a standardized JSON-RPC interface to inspect the scene graph,
//! query ECS entities, manipulate reactive state, and perform visual verification.
//!
//! ## Purpose
//!
//! `arthropod-mcp` bridges the gap between the internal state of a GUI application and external
//! automation tools. It allows you to:
//!
//! - **Inspect**: View the hierarchy of UI widgets and their properties.
//! - **Debug**: Query the ECS world to see active entities and components.
//! - **Test**: Verify that the UI state matches expectations (e.g., "Is the button visible?").
//! - **Automate**: Simulate user interactions and state changes.
//!
//! ## Architecture
//!
//! The MCP server operates in two primary modes:
//!
//! 1.  **Standalone Mode** (Default): Runs as a separate process that communicates with a live
//!     Arthropod application via TCP. The standalone server exposes an MCP interface over Stdio
//!     (standard input/output) for AI agents to consume.
//! 2.  **Embedded Mode**: Can be integrated directly into an application (mostly for internal use).
//!
//! ## Usage
//!
//! The following example demonstrates how to set up the MCP server to listen for connections from
//! a live application and expose the MCP interface via Stdio.
//!
//! ```no_run
//! use arthropod_mcp::{ArthropodServer, McpFrameworkContext, start_tcp_server, MCP_PORT};
//! use rmcp::ServiceExt;
//! use tokio::io::{stdin, stdout};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 1. Start the TCP server to accept connections from the live app
//!     let connected_app = start_tcp_server();
//!     println!("Listening for app on localhost:{}", MCP_PORT);
//!
//!     // 2. Create a fallback context (used if no app is connected)
//!     let context = McpFrameworkContext::new();
//!
//!     // 3. Initialize the MCP server with the live app connection
//!     let server = ArthropodServer::with_live_app(context, connected_app);
//!
//!     // 4. Serve the MCP protocol over Stdio (Standard Input/Output)
//!     // This is how AI agents typically communicate with the tool.
//!     let transport = (stdin(), stdout());
//!     let server_handle = server.serve(transport).await?;
//!
//!     // 5. Wait for the server to shut down
//!     server_handle.waiting().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Available Tools
//!
//! The server exposes several categories of tools to the MCP client:
//!
//! ### 🌳 Scene Tools (`scene.*`)
//! Tools for inspecting and manipulating the visual scene graph.
//! - `scene.list_nodes`: List all nodes with optional filtering.
//! - `scene.get_node`: Get detailed properties of a specific node.
//! - `scene.query_hierarchy`: Traverse the tree structure.
//! - `scene.find_nodes_at_position`: Hit-testing for UI elements.
//!
//! ### 🧩 ECS Tools (`ecs.*`)
//! Tools for querying the Entity Component System.
//! - `ecs.query_entities`: Find entities with specific components.
//! - `ecs.get_entity`: Inspect component data for an entity.
//! - `ecs.count_entities`: rapid statistical queries.
//!
//! ### ⚡ State Tools (`state.*`)
//! Tools for interacting with the `flux-state` reactive system.
//! - `state.get_signal`: Read the current value of a signal.
//! - `state.trigger_update`: Force a reactivity update cycle.
//!
//! ### 🧪 Test Tools (`test.*`)
//! Utilities for automated verification.
//! - `test.assert_node_state`: Verify a node matches expected JSON.
//! - `test.verify_render_output`: Check visual output (requires rendering backend).

pub mod context;
pub mod live;
pub mod protocol;
pub mod registry;
pub mod server;
pub mod tools;
pub mod validation;

// Re-export main types
pub use context::McpFrameworkContext;
pub use live::{
    AppConnection, AppMessage, ConnectedApp, MCP_PORT, ServerMessage, connect_to_mcp_server,
    start_tcp_server,
};
pub use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use registry::{SignalEntry, SignalRegistry};
pub use server::ArthropodServer;
pub use tools::{Tool, ToolRegistry};

// Legacy alias for backwards compatibility
#[deprecated(note = "Use ArthropodServer instead")]
pub type McpServer = ArthropodServer;
