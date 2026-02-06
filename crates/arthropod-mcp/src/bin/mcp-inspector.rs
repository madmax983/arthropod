//! MCP Inspector TUI
//!
//! A tool to inspect and interact with the Arthropod MCP server.
//!
//! Usage:
//!     cargo run --bin mcp-inspector -- [SERVER_CMD] [SERVER_ARGS]...
//!
//! Example:
//!     cargo run --bin mcp-inspector -- cargo run --bin arthropod-mcp

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use serde_json::Value;
use std::{
    collections::VecDeque,
    io,
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::mpsc,
};

use arthropod_mcp::protocol::{JsonRpcRequest, JsonRpcResponse};

/// Application state
struct App {
    /// List of available tools
    tools: Vec<String>,
    /// Selected tool index
    tool_list_state: ListState,
    /// Logs (requests and responses)
    logs: VecDeque<String>,
    /// Whether to quit
    should_quit: bool,
    /// Command being typed (for custom JSON params)
    input: String,
    /// Input mode
    input_mode: InputMode,
    /// Connection status
    status: String,
    /// Current request ID counter
    request_id: u64,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

enum AppEvent {
    Input(KeyEvent),
    Message(String),
    Tick,
}

impl App {
    fn new() -> Self {
        let mut tool_list_state = ListState::default();
        tool_list_state.select(Some(0));

        Self {
            tools: Vec::new(),
            tool_list_state,
            logs: VecDeque::with_capacity(100),
            should_quit: false,
            input: String::new(),
            input_mode: InputMode::Normal,
            status: "Starting...".to_string(),
            request_id: 1,
        }
    }

    fn next_tool(&mut self) {
        if self.tools.is_empty() {
            return;
        }
        let i = match self.tool_list_state.selected() {
            Some(i) => {
                if i >= self.tools.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.tool_list_state.select(Some(i));
    }

    fn previous_tool(&mut self) {
        if self.tools.is_empty() {
            return;
        }
        let i = match self.tool_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tools.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.tool_list_state.select(Some(i));
    }

    fn next_request_id(&mut self) -> u64 {
        let id = self.request_id;
        self.request_id += 1;
        id
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Parse Args & Spawn Server
    let args: Vec<String> = std::env::args().collect();

    // Logic: If "--" is present, use args after it.
    // If not, default to "cargo run --bin arthropod-mcp"
    let (program, program_args) = if let Some(pos) = args.iter().position(|arg| arg == "--") {
        if pos + 1 < args.len() {
            (args[pos + 1].clone(), args[pos + 2..].to_vec())
        } else {
             // Default if "--" is last
             ("cargo".to_string(), vec!["run".to_string(), "--bin".to_string(), "arthropod-mcp".to_string()])
        }
    } else {
         // Default
         ("cargo".to_string(), vec!["run".to_string(), "--bin".to_string(), "arthropod-mcp".to_string()])
    };

    let mut child = Command::new(&program)
        .args(&program_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdin = child.stdin.take().expect("Failed to open stdin");
    let child_stdout = child.stdout.take().expect("Failed to open stdout");
    let child_stderr = child.stderr.take().expect("Failed to open stderr");

    let mut stdin_writer = tokio::io::BufWriter::new(child_stdin);

    // 3. Channels
    let (tx, mut rx) = mpsc::channel(100);

    // Stdout Reader Task
    let tx_stdout = tx.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(child_stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_stdout.send(AppEvent::Message(line)).await;
        }
    });

    // Stderr Reader Task
    let tx_stderr = tx.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(child_stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_stderr.send(AppEvent::Message(format!("ERR: {}", line))).await;
        }
    });

    // Input Reader Task
    let tx_input = tx.clone();
    tokio::spawn(async move {
        let tick_rate = Duration::from_millis(200);
        let mut last_tick = Instant::now();

        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap_or(false) {
                if let Event::Key(key) = event::read().unwrap_or(Event::FocusLost) {
                    if tx_input.send(AppEvent::Input(key)).await.is_err() {
                        return;
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if tx_input.send(AppEvent::Tick).await.is_err() {
                    return;
                }
                last_tick = Instant::now();
            }
        }
    });

    // 4. App Loop
    let mut app = App::new();

    // Send initialize
    let id = app.next_request_id();
    let init_req = JsonRpcRequest::new(id, "initialize", serde_json::json!({
        "client_name": "mcp-inspector",
        "client_version": "0.1.0"
    }));
    let init_str = serde_json::to_string(&init_req)?;
    stdin_writer.write_all(format!("{}\n", init_str).as_bytes()).await?;
    stdin_writer.flush().await?;
    app.logs.push_back(format!("-> {}", init_str));
    app.status = "Initializing...".to_string();

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Some(evt) = rx.recv().await {
            match evt {
                AppEvent::Input(key) => {
                    match app.input_mode {
                        InputMode::Normal => {
                            match key.code {
                                KeyCode::Char('q') => app.should_quit = true,
                                KeyCode::Down => app.next_tool(),
                                KeyCode::Up => app.previous_tool(),
                                KeyCode::Enter => {
                                    // Trigger selected tool with default/empty params
                                    if let Some(idx) = app.tool_list_state.selected() {
                                        if idx < app.tools.len() {
                                            let tool_name = app.tools[idx].clone();
                                            let id = app.next_request_id();
                                            let req = JsonRpcRequest::new(id, "tools/call", serde_json::json!({
                                                "name": tool_name,
                                                "arguments": {}
                                            }));
                                            let req_str = serde_json::to_string(&req)?;
                                            stdin_writer.write_all(format!("{}\n", req_str).as_bytes()).await?;
                                            stdin_writer.flush().await?;
                                            app.logs.push_back(format!("-> {}", req_str));
                                        }
                                    }
                                }
                                KeyCode::Char('i') => {
                                    app.input_mode = InputMode::Editing;
                                }
                                KeyCode::Char('l') => {
                                    // Refresh tools
                                    let id = app.next_request_id();
                                    let req = JsonRpcRequest::new(id, "tools/list", serde_json::json!({}));
                                    let req_str = serde_json::to_string(&req)?;
                                    stdin_writer.write_all(format!("{}\n", req_str).as_bytes()).await?;
                                    stdin_writer.flush().await?;
                                    app.logs.push_back(format!("-> {}", req_str));
                                }
                                _ => {}
                            }
                        }
                        InputMode::Editing => {
                            match key.code {
                                KeyCode::Enter => {
                                    let input_cmd = app.input.clone();
                                    app.input.clear();
                                    app.input_mode = InputMode::Normal;

                                    // Parse input: "method json_params"
                                    let parts: Vec<&str> = input_cmd.trim().splitn(2, ' ').collect();
                                    if !parts.is_empty() {
                                        let method = parts[0];
                                        let params_str = if parts.len() > 1 { parts[1] } else { "{}" };

                                        match serde_json::from_str::<Value>(params_str) {
                                            Ok(params) => {
                                                let id = app.next_request_id();
                                                let req = JsonRpcRequest::new(id, method, params);
                                                if let Ok(req_str) = serde_json::to_string(&req) {
                                                     stdin_writer.write_all(format!("{}\n", req_str).as_bytes()).await?;
                                                     stdin_writer.flush().await?;
                                                     app.logs.push_back(format!("-> {}", req_str));
                                                }
                                            }
                                            Err(e) => {
                                                app.logs.push_back(format!("Error parsing params: {}", e));
                                            }
                                        }
                                    }
                                }
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Backspace => {
                                    app.input.pop();
                                }
                                KeyCode::Char(c) => {
                                    app.input.push(c);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                AppEvent::Message(msg) => {
                    app.logs.push_back(format!("<- {}", msg));
                    // Try to parse message
                    if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&msg) {
                         // Check if it's tools/list response
                         if let Some(result) = resp.result {
                             if let Some(tools_val) = result.get("tools") {
                                 if let Some(tools_arr) = tools_val.as_array() {
                                     app.tools = tools_arr.iter()
                                         .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                                         .collect();
                                     app.status = format!("Connected ({} tools)", app.tools.len());
                                 }
                             }
                         }
                    }

                    // Keep logs trimmed
                    if app.logs.len() > 100 {
                        app.logs.pop_front();
                    }
                }
                AppEvent::Tick => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Don't wait forever
    child.kill().await?;

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.area());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(chunks[1]);

    // Tools List
    let items: Vec<ListItem> = app.tools
        .iter()
        .map(|t| ListItem::new(Line::from(vec![Span::raw(t)])))
        .collect();

    let tools_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Tools"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
        .highlight_symbol(">> ");

    f.render_stateful_widget(tools_list, left_chunks[0], &mut app.tool_list_state);

    // Logs
    // Extract last 20 entries
    let recent_logs: Vec<&String> = app.logs.iter().rev().take(20).rev().collect();

    let mut log_lines = Vec::new();
    for entry in recent_logs {
         log_lines.extend(format_log_entry(entry));
         log_lines.push(Line::raw("")); // Spacing between entries
    }

    let logs = Paragraph::new(log_lines)
        .block(Block::default().borders(Borders::ALL).title(format!("Logs - {}", app.status)))
        .wrap(Wrap { trim: true });

    f.render_widget(logs, right_chunks[0]);

    // Input
    let input_title = match app.input_mode {
        InputMode::Normal => "Input (i: edit, q: quit, l: refresh)",
        InputMode::Editing => "Input (Esc: cancel, Enter: send)",
    };

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title(input_title));

    f.render_widget(input, right_chunks[1]);
}

fn format_log_entry(entry: &str) -> Vec<Line<'_>> {
    let (prefix, rest) = if let Some(stripped) = entry.strip_prefix("-> ") {
        ("-> ", stripped)
    } else if let Some(stripped) = entry.strip_prefix("<- ") {
        ("<- ", stripped)
    } else if let Some(stripped) = entry.strip_prefix("ERR: ") {
        ("ERR: ", stripped)
    } else {
        ("", entry)
    };

    let style = match prefix {
        "-> " => Style::default().fg(Color::Blue),
        "<- " => Style::default().fg(Color::Green),
        "ERR: " => Style::default().fg(Color::Red),
        _ => Style::default(),
    };

    let mut lines = Vec::new();

    // Attempt to pretty print JSON
    let content = if !rest.trim().is_empty() {
         match serde_json::from_str::<Value>(rest) {
            Ok(val) => match serde_json::to_string_pretty(&val) {
                Ok(pretty) => pretty,
                Err(_) => rest.to_string(),
            },
            Err(_) => rest.to_string(),
        }
    } else {
        rest.to_string()
    };

    // Add first line with prefix
    let mut content_lines = content.lines();
    if let Some(first) = content_lines.next() {
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::raw(first.to_string()),
        ]));
    } else {
        // Empty content, just prefix
        lines.push(Line::from(Span::styled(prefix, style)));
    }

    // Add remaining lines indented
    for line in content_lines {
        lines.push(Line::from(vec![
            Span::raw("   "), // Indentation matching prefix length roughly
            Span::raw(line.to_string()),
        ]));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_log_entry_json() {
        let entry = r#"-> {"jsonrpc":"2.0","method":"test"}"#;
        let lines = format_log_entry(entry);

        // Should have multiple lines due to pretty printing
        assert!(lines.len() > 1);

        // First line should have blue prefix
        let first_line = &lines[0];
        assert_eq!(first_line.spans[0].content, "-> ");
        assert_eq!(first_line.spans[0].style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_format_log_entry_error() {
        let entry = "ERR: Connection failed";
        let lines = format_log_entry(entry);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].spans[0].content, "ERR: ");
        assert_eq!(lines[0].spans[0].style.fg, Some(Color::Red));
        assert_eq!(lines[0].spans[1].content, "Connection failed");
    }
}
