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

/// Maximum message size allowed (64MB) to prevent DoS
const MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;

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
    let (app, _) = start_tcp_server_with_port(MCP_PORT);
    app
}

/// Start the TCP server on a specific port.
/// Returns the shared app state and the bound port.
pub fn start_tcp_server_with_port(port: u16) -> (Arc<Mutex<Option<ConnectedApp>>>, u16) {
    let listener =
        TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind MCP server");
    let local_port = listener.local_addr().unwrap().port();

    let connected_app = Arc::new(Mutex::new(None));
    let connected_app_clone = connected_app.clone();

    tracing::info!("MCP server listening on localhost:{}", local_port);

    thread::spawn(move || {
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

    (connected_app, local_port)
}

/// Read a line with a maximum length limit to prevent DoS.
///
/// Returns the number of bytes read (including delimiter).
fn read_line_bounded(
    reader: &mut impl BufRead,
    buf: &mut String,
    limit: usize,
) -> std::io::Result<usize> {
    let mut bytes = Vec::new();
    let mut total_read = 0;

    loop {
        let available = reader.fill_buf()?;
        let len = available.len();
        if len == 0 {
            break; // EOF
        }

        let (consume, newline) = if let Some(i) = available.iter().position(|&b| b == b'\n') {
            (i + 1, true)
        } else {
            (len, false)
        };

        if total_read + consume > limit {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Line too long",
            ));
        }

        bytes.extend_from_slice(&available[..consume]);
        reader.consume(consume);
        total_read += consume;

        if newline {
            break;
        }
    }

    if total_read == 0 {
        return Ok(0);
    }

    // Convert to string (validates UTF-8)
    let s = String::from_utf8(bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    buf.push_str(&s);
    Ok(total_read)
}

/// Handle a connection from an Arthropod app
fn handle_app_connection(stream: TcpStream, connected_app: Arc<Mutex<Option<ConnectedApp>>>) {
    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));
    let mut line = String::new();

    loop {
        line.clear();
        match read_line_bounded(&mut reader, &mut line, MAX_MESSAGE_SIZE) {
            Ok(0) => break, // EOF
            Ok(_) => {
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
            }
            Err(e) => {
                tracing::error!("Connection error: {}", e);
                break;
            }
        }
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
