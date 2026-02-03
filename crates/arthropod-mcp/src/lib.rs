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
//! - **Standalone**: Separate process with embedded test harness (Phase 1 - Current)
//! - **Embedded**: Library integrated into applications (Phase 2 - Planned/Internal Use Only)
//!
//! ## Example
//!
//! ```ignore
//! use arthropod_mcp::{ArthropodServer, start_tcp_server};
//!
//! // Create server for handling MCP requests
//! let server = ArthropodServer::new();
//!
//! // Start TCP server for live connections (async)
//! // start_tcp_server().await?;
//! ```

pub mod context;
pub mod live;
pub mod protocol;
pub mod registry;
pub mod server;
pub mod tools;

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
