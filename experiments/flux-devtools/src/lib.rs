use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use flux_state::{GraphSnapshot, NodeInfo, NodeType, Runtime};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
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

        terminal.draw(|f| ui(f, app))?;

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

fn ui(f: &mut Frame, app: &mut App) {
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

    // --- Left: Node List ---
    let items: Vec<ListItem> = app
        .snapshot
        .nodes
        .iter()
        .map(|node| {
            let mut style = Style::default();

            // Color coding by type
            match node.node_type {
                NodeType::Signal => style = style.fg(Color::Green),
                NodeType::Computed => style = style.fg(Color::Blue),
                NodeType::Effect => style = style.fg(Color::Yellow),
            }

            // Stale marker
            let stale_marker = if app.snapshot.stale_nodes.contains(&node.id) {
                "⚠️ "
            } else {
                "  "
            };

            // Formatting: "  Signal(1) Label"
            let label = format!("{}{:?} {:?}", stale_marker, node.node_type, node.id.0);

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

    f.render_stateful_widget(list, main_chunks[0], &mut app.list_state);

    // --- Right: Details Panel ---
    // Render details for the selected node
    if let Some(selected_idx) = app.list_state.selected() {
        if let Some(selected_node) = app.snapshot.nodes.get(selected_idx) {
            render_details(f, main_chunks[1], selected_node, &app.snapshot);
        }
    } else {
        let placeholder = Paragraph::new("Select a node to view details").block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
        f.render_widget(placeholder, main_chunks[1]);
    }

    // --- Status Bar ---
    let status_text = format!(
        " Total Nodes: {} | Stale: {} ",
        app.snapshot.nodes.len(),
        app.snapshot.stale_nodes.len()
    );
    let status = Paragraph::new(status_text)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .block(Block::default().borders(Borders::NONE)); // Flat look
    f.render_widget(status, chunks[2]);
}

fn render_details(f: &mut Frame, area: Rect, node: &NodeInfo, snapshot: &GraphSnapshot) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),      // Metadata
            Constraint::Percentage(50), // Dependencies (Upstream)
            Constraint::Percentage(50), // Subscribers (Downstream)
        ])
        .split(area);

    // 1. Metadata
    let type_color = match node.node_type {
        NodeType::Signal => Color::Green,
        NodeType::Computed => Color::Blue,
        NodeType::Effect => Color::Yellow,
    };

    let is_stale = snapshot.stale_nodes.contains(&node.id);
    let status_str = if is_stale {
        "⚠️ STALE"
    } else {
        "✅ FRESH"
    };
    let status_color = if is_stale { Color::Red } else { Color::Green };

    let meta_text = vec![
        Line::from(vec![
            Span::raw("ID: "),
            Span::styled(
                format!("{:?}", node.id.0),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Type: "),
            Span::styled(
                format!("{:?}", node.node_type),
                Style::default().fg(type_color),
            ),
        ]),
        Line::from(vec![Span::raw("Label: "), Span::raw(&node.label)]),
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(status_str, Style::default().fg(status_color)),
        ]),
    ];

    let meta_p = Paragraph::new(meta_text)
        .block(Block::default().title(" Metadata ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(meta_p, chunks[0]);

    // 2. Dependencies (Incoming: Source -> This Node)
    // Find edges where target == node.id
    let incoming: Vec<ListItem> = snapshot
        .dependencies
        .iter()
        .filter(|(_, target)| *target == node.id)
        .map(|(source, _)| {
            // Find source node info for better display
            let src_node = snapshot.nodes.iter().find(|n| n.id == *source);
            let label = if let Some(n) = src_node {
                format!("{:?} ({:?})", n.node_type, n.id.0)
            } else {
                format!("Unknown({:?})", source.0)
            };
            ListItem::new(format!("← {}", label))
        })
        .collect();

    let incoming_list = List::new(incoming).block(
        Block::default()
            .title(" Dependencies (Upstream) ")
            .borders(Borders::ALL),
    );
    f.render_widget(incoming_list, chunks[1]);

    // 3. Subscribers (Outgoing: This Node -> Target)
    // Find edges where source == node.id
    let outgoing: Vec<ListItem> = snapshot
        .dependencies
        .iter()
        .filter(|(source, _)| *source == node.id)
        .map(|(_, target)| {
            let target_node = snapshot.nodes.iter().find(|n| n.id == *target);
            let label = if let Some(n) = target_node {
                format!("{:?} ({:?})", n.node_type, n.id.0)
            } else {
                format!("Unknown({:?})", target.0)
            };
            ListItem::new(format!("→ {}", label))
        })
        .collect();

    let outgoing_list = List::new(outgoing).block(
        Block::default()
            .title(" Subscribers (Downstream) ")
            .borders(Borders::ALL),
    );
    f.render_widget(outgoing_list, chunks[2]);
}
