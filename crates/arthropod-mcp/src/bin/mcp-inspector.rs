//! MCP Inspector TUI
//!
//! A tool to inspect and interact with the Arthropod MCP server.
//!
//! Usage:
//!     cargo run --bin mcp-inspector -- \[SERVER_CMD\] \[SERVER_ARGS\]...
//!
//! Example:
//!     cargo run --bin mcp-inspector -- cargo run --bin arthropod-mcp

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
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

#[derive(PartialEq, Clone, Copy)]
enum LogDirection {
    Outgoing, // ->
    Incoming, // <-
    Error,    // ERR:
}

struct LogEntry {
    #[allow(dead_code)]
    timestamp: Instant,
    direction: LogDirection,
    content: String,
    parsed: Option<Value>,
}

#[derive(PartialEq, Clone, Copy)]
enum Focus {
    ToolsList,
    LogHistory,
    LogDetails,
    Input,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

struct ToolInfo {
    name: String,
    description: String,
}

/// Application state
struct App {
    /// List of available tools
    tools: Vec<ToolInfo>,
    /// Selected tool index
    tool_list_state: ListState,

    /// Logs (requests and responses)
    logs: VecDeque<LogEntry>,
    /// Selected log index for details view
    log_list_state: ListState,
    /// Vertical scroll offset for details view
    details_scroll: u16,

    /// Current UI Focus
    focus: Focus,

    /// Whether to quit
    should_quit: bool,
    /// Command being typed (for custom JSON params)
    input: String,
    /// Input mode
    input_mode: InputMode,
    /// Connection status
    status: String,
    /// Connected server command
    server_cmd: String,
    /// Current request ID counter
    request_id: u64,
}

enum AppEvent {
    Input(KeyEvent),
    Message(String),
    Tick,
}

impl App {
    fn new(server_cmd: String) -> Self {
        let mut tool_list_state = ListState::default();
        tool_list_state.select(Some(0));

        let log_list_state = ListState::default();
        // log_list_state.select(Some(0)); // No logs initially

        Self {
            tools: Vec::new(),
            tool_list_state,
            logs: VecDeque::with_capacity(100),
            log_list_state,
            details_scroll: 0,
            focus: Focus::ToolsList,
            should_quit: false,
            input: String::new(),
            input_mode: InputMode::Normal,
            status: "Starting...".to_string(),
            server_cmd,
            request_id: 1,
        }
    }

    fn add_log(&mut self, content: String, direction: LogDirection) {
        let parsed = serde_json::from_str::<Value>(&content).ok();
        let entry = LogEntry {
            timestamp: Instant::now(),
            direction,
            content,
            parsed,
        };
        self.logs.push_back(entry);
        if self.logs.len() > 100 {
            self.logs.pop_front();
        }

        // Auto-scroll if we are at the bottom or nothing selected
        if self.focus != Focus::LogHistory {
            self.log_list_state
                .select(Some(self.logs.len().saturating_sub(1)));
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

    fn next_log(&mut self) {
        if self.logs.is_empty() {
            return;
        }
        let i = match self.log_list_state.selected() {
            Some(i) => {
                if i >= self.logs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.log_list_state.select(Some(i));
        self.details_scroll = 0;
    }

    fn previous_log(&mut self) {
        if self.logs.is_empty() {
            return;
        }
        let i = match self.log_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.logs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.log_list_state.select(Some(i));
        self.details_scroll = 0;
    }

    async fn handle_tool_call(
        &mut self,
        stdin_writer: &mut tokio::io::BufWriter<tokio::process::ChildStdin>,
    ) -> Result<()> {
        let Some(idx) = self.tool_list_state.selected() else {
            return Ok(());
        };

        if idx >= self.tools.len() {
            return Ok(());
        }

        let tool_name = self.tools[idx].name.clone();
        let id = self.next_request_id();
        let req = JsonRpcRequest::new(
            id,
            "tools/call",
            serde_json::json!({
                "name": tool_name,
                "arguments": {}
            }),
        );
        let req_str = serde_json::to_string(&req)?;
        stdin_writer
            .write_all(format!("{}\n", req_str).as_bytes())
            .await?;
        let _ = stdin_writer.flush().await;
        self.add_log(req_str, LogDirection::Outgoing);
        Ok(())
    }

    async fn handle_input_cmd(
        &mut self,
        stdin_writer: &mut tokio::io::BufWriter<tokio::process::ChildStdin>,
        input_cmd: String,
    ) -> Result<()> {
        let parts: Vec<&str> = input_cmd.trim().splitn(2, ' ').collect();
        if !parts.is_empty() {
            let method = parts[0];
            let params_str = if parts.len() > 1 { parts[1] } else { "{}" };

            match serde_json::from_str::<serde_json::Value>(params_str) {
                Ok(params) => {
                    let id = self.next_request_id();
                    let req = JsonRpcRequest::new(id, method, params);
                    if let Ok(req_str) = serde_json::to_string(&req) {
                        stdin_writer
                            .write_all(format!("{}\n", req_str).as_bytes())
                            .await?;
                        let _ = stdin_writer.flush().await;
                        self.add_log(req_str, LogDirection::Outgoing);
                    }
                }
                Err(e) => self.add_log(format!("Error parsing params: {}", e), LogDirection::Error),
            }
        }
        Ok(())
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
            (
                "cargo".to_string(),
                vec![
                    "run".to_string(),
                    "--bin".to_string(),
                    "arthropod-mcp".to_string(),
                ],
            )
        }
    } else {
        // Default
        (
            "cargo".to_string(),
            vec![
                "run".to_string(),
                "--bin".to_string(),
                "arthropod-mcp".to_string(),
            ],
        )
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
            let _ = tx_stderr
                .send(AppEvent::Message(format!("ERR: {}", line)))
                .await;
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

            let key_event = event::poll(timeout)
                .unwrap_or(false)
                .then(|| event::read().ok())
                .flatten();

            #[allow(clippy::collapsible_if)]
            if let Some(Event::Key(key)) = key_event {
                if tx_input.send(AppEvent::Input(key)).await.is_err() {
                    return;
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
    let server_cmd_display = format!("{} {}", program, program_args.join(" "));
    let mut app = App::new(server_cmd_display);

    // Send initialize
    let id = app.next_request_id();
    let init_req = JsonRpcRequest::new(
        id,
        "initialize",
        serde_json::json!({
            "client_name": "mcp-inspector",
            "client_version": "0.1.0"
        }),
    );
    let init_str = serde_json::to_string(&init_req)?;
    stdin_writer
        .write_all(format!("{}\n", init_str).as_bytes())
        .await?;
    stdin_writer.flush().await?;
    app.add_log(init_str, LogDirection::Outgoing);
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
                                KeyCode::Tab => {
                                    app.focus = match app.focus {
                                        Focus::ToolsList => Focus::LogHistory,
                                        Focus::LogHistory => Focus::LogDetails,
                                        Focus::LogDetails => Focus::Input,
                                        Focus::Input => Focus::ToolsList,
                                    };
                                }
                                KeyCode::Down => match app.focus {
                                    Focus::ToolsList => app.next_tool(),
                                    Focus::LogHistory => app.next_log(),
                                    Focus::LogDetails => {
                                        app.details_scroll = app.details_scroll.saturating_add(1)
                                    }
                                    Focus::Input => {}
                                },
                                KeyCode::Up => match app.focus {
                                    Focus::ToolsList => app.previous_tool(),
                                    Focus::LogHistory => app.previous_log(),
                                    Focus::LogDetails => {
                                        app.details_scroll = app.details_scroll.saturating_sub(1)
                                    }
                                    Focus::Input => {}
                                },
                                KeyCode::Enter => {
                                    // Trigger selected tool with default/empty params if focused
                                    if app.focus != Focus::ToolsList {
                                        continue;
                                    }
                                    let _ = app.handle_tool_call(&mut stdin_writer).await;
                                }
                                KeyCode::Char('i') => {
                                    app.input_mode = InputMode::Editing;
                                    app.focus = Focus::Input;
                                }
                                KeyCode::Char('l') => {
                                    // Refresh tools
                                    let id = app.next_request_id();
                                    let req = JsonRpcRequest::new(
                                        id,
                                        "tools/list",
                                        serde_json::json!({}),
                                    );
                                    let req_str = serde_json::to_string(&req)?;
                                    stdin_writer
                                        .write_all(format!("{}\n", req_str).as_bytes())
                                        .await?;
                                    stdin_writer.flush().await?;
                                    app.add_log(req_str, LogDirection::Outgoing);
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
                                    app.focus = Focus::ToolsList; // Return focus to tools

                                    let _ =
                                        app.handle_input_cmd(&mut stdin_writer, input_cmd).await;
                                }
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                    app.focus = Focus::ToolsList;
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
                    let (direction, content) = if let Some(stripped) = msg.strip_prefix("ERR: ") {
                        (LogDirection::Error, stripped.to_string())
                    } else {
                        (LogDirection::Incoming, msg.clone())
                    };

                    app.add_log(content.clone(), direction);

                    // Try to parse message for tools list
                    let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&content) else {
                        continue;
                    };

                    let Some(result) = resp.result else {
                        continue;
                    };

                    let Some(tools_val) = result.get("tools") else {
                        continue;
                    };

                    let Some(tools_arr) = tools_val.as_array() else {
                        continue;
                    };

                    app.tools = tools_arr
                        .iter()
                        .filter_map(|t| {
                            let name = t.get("name").and_then(|n| n.as_str())?.to_string();
                            let description = t
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                                .to_string();
                            Some(ToolInfo { name, description })
                        })
                        .collect();
                    app.status = format!("Connected ({} tools)", app.tools.len());
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
    // Top level layout: Main Area vs Status Bar
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(f.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(main_layout[0]);

    // Left Column: Tools
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(chunks[0]);

    // Right Column: History (Top), Details (Middle), Input (Bottom)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    // Styles based on Focus
    let tools_border_style = if app.focus == Focus::ToolsList {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let history_border_style = if app.focus == Focus::LogHistory {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let details_border_style = if app.focus == Focus::LogDetails {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let input_border_style = if app.focus == Focus::Input {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    // --- Tools List ---
    let items: Vec<ListItem> = app
        .tools
        .iter()
        .map(|t| {
            let mut spans = vec![Span::styled(
                &t.name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )];

            if !t.description.is_empty() {
                spans.push(Span::raw(" - "));
                spans.push(Span::styled(
                    &t.description,
                    Style::default().fg(Color::DarkGray),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let tools_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Tools")
                .border_style(tools_border_style),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(tools_list, left_chunks[0], &mut app.tool_list_state);

    // --- Log History ---
    // We want to show logs in order.
    let log_items: Vec<ListItem> = app
        .logs
        .iter()
        .map(|entry| {
            let (prefix, style) = match entry.direction {
                LogDirection::Outgoing => ("-> ", Style::default().fg(Color::Blue)),
                LogDirection::Incoming => ("<- ", Style::default().fg(Color::Green)),
                LogDirection::Error => ("ERR: ", Style::default().fg(Color::Red)),
            };

            // Summary: Method name or short preview
            let summary = if let Some(val) = &entry.parsed {
                if let Some(method) = val.get("method").and_then(|v| v.as_str()) {
                    format!("{} {}", prefix, method)
                } else if val.get("result").is_some() {
                    format!("{} Response", prefix)
                } else if val.get("error").is_some() {
                    format!("{} Error", prefix)
                } else {
                    format!("{} JSON", prefix)
                }
            } else {
                format!(
                    "{} {}",
                    prefix,
                    entry.content.chars().take(50).collect::<String>()
                )
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::raw(summary),
            ]))
        })
        .collect();

    let logs_list = List::new(log_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("History")
                .border_style(history_border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(logs_list, right_chunks[0], &mut app.log_list_state);

    // --- Log Details ---
    let selected_index = app.log_list_state.selected();
    let details_text = if let Some(idx) = selected_index {
        if let Some(entry) = app.logs.get(idx) {
            if let Some(parsed) = &entry.parsed {
                pretty_print_json(parsed)
            } else {
                vec![Line::from(entry.content.clone())]
            }
        } else {
            vec![Line::from("No log selected")]
        }
    } else {
        vec![Line::from("Select a log entry to view details")]
    };

    let details = Paragraph::new(details_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Details")
                .border_style(details_border_style),
        )
        .wrap(Wrap { trim: false }) // Preserved indentation
        .scroll((app.details_scroll, 0));

    f.render_widget(details, right_chunks[1]);

    // --- Input ---
    let input_title = match app.input_mode {
        InputMode::Normal => "Input (i: edit, q: quit, l: refresh)",
        InputMode::Editing => "Input (Esc: cancel, Enter: send)",
    };

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(input_title)
                .border_style(input_border_style),
        );

    f.render_widget(input, right_chunks[2]);

    // --- Status Bar ---
    let status_line = Line::from(vec![
        Span::styled(" Status: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(&app.status),
        Span::raw(" | "),
        Span::styled(" Cmd: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(&app.server_cmd),
        Span::raw(" | "),
        Span::styled("Focus: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(match app.focus {
            Focus::ToolsList => "Tools (Tab to switch)",
            Focus::LogHistory => "History (Tab to switch)",
            Focus::LogDetails => "Details (Up/Down to scroll, Tab to switch)",
            Focus::Input => "Input (Esc to exit)",
        }),
    ]);

    let status_bar =
        Paragraph::new(status_line).style(Style::default().bg(Color::Blue).fg(Color::White));

    f.render_widget(status_bar, main_layout[1]);
}

fn pretty_print_json(value: &Value) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    format_json_value(None, value, 0, &mut lines, false);
    lines
}

fn format_json_value(
    key: Option<&str>,
    value: &Value,
    indent_level: usize,
    lines: &mut Vec<Line<'static>>,
    trailing_comma: bool,
) {
    let indent = " ".repeat(indent_level * 2);
    let comma = if trailing_comma { "," } else { "" };

    let mut spans = Vec::new();
    spans.push(Span::raw(indent.clone()));

    if let Some(k) = key {
        spans.push(Span::styled(
            format!("\"{}\"", k),
            Style::default().fg(Color::Blue),
        ));
        spans.push(Span::raw(": "));
    }

    match value {
        Value::Null => {
            spans.push(Span::styled("null", Style::default().fg(Color::DarkGray)));
            spans.push(Span::raw(comma));
            lines.push(Line::from(spans));
        }
        Value::Bool(b) => {
            spans.push(Span::styled(
                b.to_string(),
                Style::default().fg(Color::Magenta),
            ));
            spans.push(Span::raw(comma));
            lines.push(Line::from(spans));
        }
        Value::Number(n) => {
            spans.push(Span::styled(
                n.to_string(),
                Style::default().fg(Color::Cyan),
            ));
            spans.push(Span::raw(comma));
            lines.push(Line::from(spans));
        }
        Value::String(s) => {
            spans.push(Span::styled(
                format!("\"{}\"", s),
                Style::default().fg(Color::Green),
            ));
            spans.push(Span::raw(comma));
            lines.push(Line::from(spans));
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                spans.push(Span::raw("[]"));
                spans.push(Span::raw(comma));
                lines.push(Line::from(spans));
                return;
            }

            spans.push(Span::raw("["));
            lines.push(Line::from(spans));

            for (i, v) in arr.iter().enumerate() {
                format_json_value(None, v, indent_level + 1, lines, i < arr.len() - 1);
            }

            lines.push(Line::from(vec![
                Span::raw(indent),
                Span::raw("]"),
                Span::raw(comma),
            ]));
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                spans.push(Span::raw("{}"));
                spans.push(Span::raw(comma));
                lines.push(Line::from(spans));
                return;
            }

            spans.push(Span::raw("{"));
            lines.push(Line::from(spans));

            for (i, (k, v)) in obj.iter().enumerate() {
                format_json_value(Some(k), v, indent_level + 1, lines, i < obj.len() - 1);
            }

            lines.push(Line::from(vec![
                Span::raw(indent),
                Span::raw("}"),
                Span::raw(comma),
            ]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pretty_print_simple() {
        let v = serde_json::json!({ "key": "value", "num": 123 });
        let lines = pretty_print_json(&v);
        assert!(!lines.is_empty());
        // Basic check that we have lines for braces and keys
        assert!(
            lines
                .iter()
                .any(|l| l.spans.iter().any(|s| s.content == "{"))
        );
        assert!(
            lines
                .iter()
                .any(|l| l.spans.iter().any(|s| s.content == "\"key\""))
        );
    }
}
