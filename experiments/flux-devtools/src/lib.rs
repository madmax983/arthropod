use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use flux_state::{GraphSnapshot, NodeType, Runtime};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::{
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};

pub fn run_inspector(runtime: Arc<Runtime>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app loop
    let res = run_app(&mut terminal, runtime);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, runtime: Arc<Runtime>) -> Result<()> {
    loop {
        // Fetch snapshot
        let snapshot = runtime.inspect_graph();

        terminal.draw(|f| ui(f, &snapshot))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key)
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') =>
                {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, snapshot: &GraphSnapshot) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Flux State Inspector (Press 'q' to quit)")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main Content (Split into Nodes and Dependencies)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Nodes List
    let nodes: Vec<ListItem> = snapshot
        .nodes
        .iter()
        .map(|n| {
            let style = match n.node_type {
                NodeType::Signal => Style::default().fg(Color::Green),
                NodeType::Computed => Style::default().fg(Color::Blue),
                NodeType::Effect => Style::default().fg(Color::Yellow),
            };
            let stale_marker = if snapshot.stale_nodes.contains(&n.id) {
                " [STALE]"
            } else {
                ""
            };
            ListItem::new(format!("{:?} {}{}", n.node_type, n.label, stale_marker)).style(style)
        })
        .collect();

    let nodes_list = List::new(nodes).block(Block::default().title("Nodes").borders(Borders::ALL));
    f.render_widget(nodes_list, content_chunks[0]);

    // Dependencies List
    let deps: Vec<ListItem> = snapshot
        .dependencies
        .iter()
        .map(|(source, target)| ListItem::new(format!("{:?} -> {:?}", source, target)))
        .collect();

    let deps_list = List::new(deps).block(
        Block::default()
            .title("Dependencies (Source -> Target)")
            .borders(Borders::ALL),
    );
    f.render_widget(deps_list, content_chunks[1]);

    // Status Bar
    let status = Paragraph::new(format!(
        "Nodes: {} | Dependencies: {} | Stale: {}",
        snapshot.nodes.len(),
        snapshot.dependencies.len(),
        snapshot.stale_nodes.len()
    ))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}
