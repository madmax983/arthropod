//! Live app integration - TCP-based connection between MCP server and running apps
//!
//! Architecture:
//! - MCP Server listens on localhost:7777
//! - Arthropod apps connect when they start
//! - Apps send Scene/Context state updates
//! - MCP tools operate on the connected app
//!
//! Only ONE app can be connected at a time (last one wins).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

/// Default port for MCP server to listen on
pub const MCP_PORT: u16 = 7777;

/// Message sent from app to MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppMessage {
    /// App registration
    Register { name: String, pid: u32 },

    /// Scene state update (full scene serialized as JSON)
    SceneUpdate { scene: Value },

    /// Keep-alive heartbeat
    Heartbeat,
}

/// Message sent from MCP server to app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Acknowledge registration
    Registered,

    /// Execute a tool command
    Command {
        id: u64,
        tool: String,
        params: Value,
    },

    /// Ping to check if app is alive
    Ping,
}

/// Shared state of the connected app
pub struct ConnectedApp {
    /// App name
    pub name: String,

    /// Process ID
    pub pid: u32,

    /// Latest scene state
    pub scene: Option<Value>,

    /// TCP stream for sending commands
    pub stream: TcpStream,
}

/// Start the TCP server that listens for app connections
///
/// Returns an Arc<Mutex<Option<ConnectedApp>>> that MCP tools can use
pub fn start_tcp_server() -> Arc<Mutex<Option<ConnectedApp>>> {
    let connected_app = Arc::new(Mutex::new(None));
    let connected_app_clone = connected_app.clone();

    thread::spawn(move || {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", MCP_PORT))
            .expect("Failed to bind MCP server");

        tracing::info!("MCP server listening on localhost:{}", MCP_PORT);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let app = connected_app_clone.clone();
                    thread::spawn(move || handle_app_connection(stream, app));
                }
                Err(e) => {
                    tracing::error!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    connected_app
}

/// Handle a connection from an Arthropod app
fn handle_app_connection(stream: TcpStream, connected_app: Arc<Mutex<Option<ConnectedApp>>>) {
    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));
    let mut line = String::new();

    while reader.read_line(&mut line).is_ok() {
        if line.is_empty() {
            break; // Connection closed
        }

        match serde_json::from_str::<AppMessage>(&line) {
            Ok(AppMessage::Register { name, pid }) => {
                tracing::info!("App registered: {} (PID: {})", name, pid);

                // Replace any existing connected app
                *connected_app.lock().unwrap() = Some(ConnectedApp {
                    name: name.clone(),
                    pid,
                    scene: None,
                    stream: stream.try_clone().expect("Failed to clone stream"),
                });

                // Send acknowledgment
                let _ = writeln!(
                    &stream,
                    "{}",
                    serde_json::to_string(&ServerMessage::Registered).unwrap()
                );
            }
            Ok(AppMessage::SceneUpdate { scene }) => {
                if let Some(app) = connected_app.lock().unwrap().as_mut() {
                    let node_count = scene
                        .get("node_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    app.scene = Some(scene);
                    tracing::info!("Scene state updated from app - {} nodes", node_count);
                }
            }
            Ok(AppMessage::Heartbeat) => {
                // App is alive, do nothing
            }
            Err(e) => {
                tracing::error!("Failed to parse message: {}", e);
            }
        }

        line.clear();
    }

    // Connection closed
    tracing::info!("App disconnected");
    *connected_app.lock().unwrap() = None;
}

/// Helper for apps to connect to the MCP server
///
/// Call this from your app on startup to register with the MCP server.
/// Returns a handle that can be used to send updates.
pub fn connect_to_mcp_server(app_name: impl Into<String>) -> Result<AppConnection, std::io::Error> {
    let stream = TcpStream::connect(format!("127.0.0.1:{}", MCP_PORT))?;
    let app_name = app_name.into();
    let pid = std::process::id();

    // Send registration message
    let mut stream_clone = stream.try_clone()?;
    let register_msg = AppMessage::Register {
        name: app_name.clone(),
        pid,
    };
    writeln!(
        &mut stream_clone,
        "{}",
        serde_json::to_string(&register_msg).unwrap()
    )?;

    Ok(AppConnection { stream, app_name })
}

/// Handle for an app connected to the MCP server
pub struct AppConnection {
    stream: TcpStream,
    app_name: String,
}

impl AppConnection {
    /// Send a scene state update to the MCP server
    pub fn send_scene_update(&mut self, scene: Value) -> Result<(), std::io::Error> {
        let msg = AppMessage::SceneUpdate { scene };
        writeln!(&mut self.stream, "{}", serde_json::to_string(&msg).unwrap())
    }

    /// Send a heartbeat to keep the connection alive
    pub fn send_heartbeat(&mut self) -> Result<(), std::io::Error> {
        let msg = AppMessage::Heartbeat;
        writeln!(&mut self.stream, "{}", serde_json::to_string(&msg).unwrap())
    }

    /// Get the app name
    pub fn name(&self) -> &str {
        &self.app_name
    }
}
