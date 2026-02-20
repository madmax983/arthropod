use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use flux_state::{GraphSnapshot, NodeInfo, NodeType, Runtime};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table},
};
use std::{
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};

/// Main entry point for the TUI inspector
pub fn run_inspector(runtime: Arc<Runtime>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App state
    let mut app = App::new(runtime);

    // Run app loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

struct App {
    runtime: Arc<Runtime>,
    snapshot: GraphSnapshot,
    list_state: ListState,
    should_quit: bool,
}

impl App {
    fn new(runtime: Arc<Runtime>) -> Self {
        let snapshot = runtime.inspect_graph();
        let mut list_state = ListState::default();
        if !snapshot.nodes.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            runtime,
            snapshot,
            list_state,
            should_quit: false,
        }
    }

    fn update_snapshot(&mut self) {
        let mut new_snapshot = self.runtime.inspect_graph();

        // STABILITY FIX: Sort nodes by ID so the list doesn't jump around
        new_snapshot.nodes.sort_by_key(|n| n.id);

        // Try to preserve selection if possible, or clamp it
        let count = new_snapshot.nodes.len();
        if count > 0 {
            let current = self.list_state.selected().unwrap_or(0);
            if current >= count {
                self.list_state.select(Some(count - 1));
            } else if self.list_state.selected().is_none() {
                self.list_state.select(Some(0));
            }
        } else {
            self.list_state.select(None);
        }

        self.snapshot = new_snapshot;
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
            _ => {}
        }
    }

    fn select_next(&mut self) {
        if self.snapshot.nodes.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.snapshot.nodes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        if self.snapshot.nodes.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.snapshot.nodes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    while !app.should_quit {
        // Refresh data
        app.update_snapshot();

        terminal.draw(|f| ui(f, &app.snapshot, &mut app.list_state))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.on_key(key.code);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, snapshot: &GraphSnapshot, list_state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main Content
            Constraint::Length(3), // Status Bar
        ])
        .split(f.area());

    // --- Header ---
    let header_text = "🎨 Flux State Inspector | Navigation: ↑/↓ or j/k | Quit: q/Esc";
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    f.render_widget(header, chunks[0]);

    // --- Main Content Split (Left: List, Right: Details) ---
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    render_node_list(f, main_chunks[0], snapshot, list_state);

    // --- Right: Details Panel ---
    if let Some(selected_idx) = list_state.selected() {
        if let Some(selected_node) = snapshot.nodes.get(selected_idx) {
            render_details_panel(f, main_chunks[1], selected_node, snapshot);
        }
    } else {
        let placeholder = Paragraph::new("Select a node to view details").block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
        f.render_widget(placeholder, main_chunks[1]);
    }

    render_status_bar(f, chunks[2], snapshot);
}

fn render_node_list(
    f: &mut Frame,
    area: Rect,
    snapshot: &GraphSnapshot,
    list_state: &mut ListState,
) {
    let items: Vec<ListItem> = snapshot
        .nodes
        .iter()
        .map(|node| {
            let (icon, style) = match node.node_type {
                NodeType::Signal => ("⚡", Style::default().fg(Color::Green)),
                NodeType::Computed => ("🧮", Style::default().fg(Color::Cyan)),
                NodeType::Effect => ("🔥", Style::default().fg(Color::Yellow)),
            };

            let stale_marker = if snapshot.stale_nodes.contains(&node.id) {
                "⚠️ "
            } else {
                "  "
            };

            let label = format!("{}{} {}", stale_marker, icon, node.label);
            ListItem::new(label).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Nodes ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, list_state);
}

fn render_details_panel(f: &mut Frame, area: Rect, node: &NodeInfo, snapshot: &GraphSnapshot) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),      // Metadata (Table needs space)
            Constraint::Percentage(50), // Dependencies
            Constraint::Percentage(50), // Subscribers
        ])
        .split(area);

    // 1. Metadata Table
    let type_color = match node.node_type {
        NodeType::Signal => Color::Green,
        NodeType::Computed => Color::Cyan,
        NodeType::Effect => Color::Yellow,
    };

    let is_stale = snapshot.stale_nodes.contains(&node.id);
    let status_str = if is_stale {
        "⚠️ STALE"
    } else {
        "✅ FRESH"
    };
    let status_color = if is_stale { Color::Red } else { Color::Green };

    let rows = vec![
        Row::new(vec![
            Cell::from("ID"),
            Cell::from(format!("{:?}", node.id.0)),
        ]),
        Row::new(vec![
            Cell::from("Type"),
            Cell::from(format!("{:?}", node.node_type)).style(Style::default().fg(type_color)),
        ]),
        Row::new(vec![Cell::from("Label"), Cell::from(node.label.clone())]),
        Row::new(vec![
            Cell::from("Status"),
            Cell::from(status_str).style(Style::default().fg(status_color)),
        ]),
    ];

    let table = Table::new(rows, [Constraint::Length(10), Constraint::Min(10)]).block(
        Block::default()
            .title(" Metadata ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    f.render_widget(table, chunks[0]);

    // 2. Dependencies
    let incoming: Vec<ListItem> = snapshot
        .dependencies
        .iter()
        .filter(|(_, target)| *target == node.id)
        .map(|(source, _)| {
            let src_node = snapshot.nodes.iter().find(|n| n.id == *source);
            let label = if let Some(n) = src_node {
                n.label.clone()
            } else {
                format!("Unknown({:?})", source.0)
            };
            ListItem::new(format!("← {}", label))
        })
        .collect();

    let incoming_list = List::new(incoming).block(
        Block::default()
            .title(" Dependencies (Upstream) ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    f.render_widget(incoming_list, chunks[1]);

    // 3. Subscribers
    let outgoing: Vec<ListItem> = snapshot
        .dependencies
        .iter()
        .filter(|(source, _)| *source == node.id)
        .map(|(_, target)| {
            let target_node = snapshot.nodes.iter().find(|n| n.id == *target);
            let label = if let Some(n) = target_node {
                n.label.clone()
            } else {
                format!("Unknown({:?})", target.0)
            };
            ListItem::new(format!("→ {}", label))
        })
        .collect();

    let outgoing_list = List::new(outgoing).block(
        Block::default()
            .title(" Subscribers (Downstream) ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    f.render_widget(outgoing_list, chunks[2]);
}

fn render_status_bar(f: &mut Frame, area: Rect, snapshot: &GraphSnapshot) {
    let total = snapshot.nodes.len();
    let stale = snapshot.stale_nodes.len();
    let signals = snapshot
        .nodes
        .iter()
        .filter(|n| matches!(n.node_type, NodeType::Signal))
        .count();
    let computeds = snapshot
        .nodes
        .iter()
        .filter(|n| matches!(n.node_type, NodeType::Computed))
        .count();
    let effects = snapshot
        .nodes
        .iter()
        .filter(|n| matches!(n.node_type, NodeType::Effect))
        .count();

    let text = Line::from(vec![
        Span::raw(format!(" Total: {} | ", total)),
        Span::styled(
            format!("Stale: {} | ", stale),
            if stale > 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            },
        ),
        Span::styled(
            format!("⚡ Signals: {} | ", signals),
            Style::default().fg(Color::Green),
        ),
        Span::styled(
            format!("🧮 Computeds: {} | ", computeds),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            format!("🔥 Effects: {} ", effects),
            Style::default().fg(Color::Yellow),
        ),
    ]);

    let paragraph = Paragraph::new(text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}
