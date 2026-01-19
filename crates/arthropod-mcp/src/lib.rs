//! # Arthropod MCP Server
//!
//! Model Context Protocol (MCP) server for Arthropod GUI framework.
//! Enables AI-driven testing, debugging, and automation through a standardized protocol.
//!
//! ## Features
//!
//! - **Scene Inspection**: Query and manipulate the scene graph
//! - **ECS Queries**: Inspect entities and components
//! - **State Manipulation**: Control reactive signals
//! - **Visual Testing**: Frame capture and comparison
//! - **Performance Monitoring**: Profile systems and operations
//!
//! ## Architecture
//!
//! The MCP server can run in two modes:
//!
//! - **Standalone**: Separate process with embedded test harness
//! - **Embedded**: Library integrated into applications (dev builds)
//!
//! ## Example
//!
//! ```no_run
//! use arthropod_mcp::{McpServer, McpFrameworkContext};
//!
//! fn main() -> anyhow::Result<()> {
//!     let mut server = McpServer::new();
//!     server.run()?;
//!     Ok(())
//! }
//! ```

pub mod context;
pub mod live;
pub mod protocol;
pub mod registry;
pub mod server;
pub mod tools;

// Re-export main types
pub use context::McpFrameworkContext;
pub use live::{start_tcp_server, connect_to_mcp_server, AppMessage, ServerMessage, ConnectedApp, AppConnection, MCP_PORT};
pub use protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
pub use registry::{SignalRegistry, SignalEntry};
pub use server::ArthropodServer;
pub use tools::{Tool, ToolRegistry};

// Legacy alias for backwards compatibility
#[deprecated(note = "Use ArthropodServer instead")]
pub type McpServer = ArthropodServer;
