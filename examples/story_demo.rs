#[cfg(feature = "nova")]
mod tui_app {
    use std::{
        io::{self, Stdout},
        time::{Duration, Instant},
    };

    use arthropod::experimental::story::{NarrativeGenerator, register_story};
    use arthropod::prelude::*;
    use crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::{
        Terminal,
        backend::CrosstermBackend,
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
    };
    use render_engine::{NodeContent, Scene};

    enum AppState {
        Intro,
        Generating {
            start_time: Instant,
        },
        Display {
            story: String,
            scroll_offset: u16,
            visible_chars: usize,
            total_chars: usize,
        },
        Error {
            message: String,
        },
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        // Setup Terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Setup App
        // We'll create the app fresh each time we generate, or reset it.
        // Actually, let's keep one app instance but we might need to clear the scene if we regenerate.
        // For simplicity, let's just create a new app instance when needed or just append.
        // But `App` owns the world.
        // Let's create the app inside the generation step to simulate a fresh start.

        // Run Loop
        let res = run_app(&mut terminal);

        // Restore Terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            eprintln!("Error: {:?}", err);
        }

        Ok(())
    }

    #[allow(clippy::collapsible_if)]
    fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        let mut state = AppState::Intro;
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(100);

        loop {
            terminal.draw(|f| ui(f, &state))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('r') => state = AppState::Intro,
                            KeyCode::Enter => {
                                if let AppState::Intro = state {
                                    state = AppState::Generating {
                                        start_time: Instant::now(),
                                    };
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if let AppState::Display { scroll_offset, .. } = &mut state {
                                    *scroll_offset = scroll_offset.saturating_add(1);
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if let AppState::Display { scroll_offset, .. } = &mut state {
                                    *scroll_offset = scroll_offset.saturating_sub(1);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();

                // State updates
                let mut next_state = None;

                match &mut state {
                    AppState::Generating { start_time } => {
                        // Simulate some "work" for 1.5 seconds to show off the spinner
                        if start_time.elapsed() > Duration::from_millis(1500) {
                            match generate_story() {
                                Ok(text) => {
                                    let total = text.chars().count();
                                    next_state = Some(AppState::Display {
                                        story: text,
                                        scroll_offset: 0,
                                        visible_chars: 0,
                                        total_chars: total,
                                    });
                                }
                                Err(e) => {
                                    next_state = Some(AppState::Error {
                                        message: e.to_string(),
                                    });
                                }
                            }
                        }
                    }
                    AppState::Display {
                        visible_chars,
                        total_chars,
                        ..
                    } => {
                        // Typewriter effect: reveal 5 chars per tick (50 chars/sec at 100ms tick)
                        if *visible_chars < *total_chars {
                            *visible_chars = (*visible_chars + 5).min(*total_chars);
                        }
                    }
                    _ => {}
                }

                if let Some(s) = next_state {
                    state = s;
                }
            }
        }
    }

    fn parse_markdown(text: &str) -> Vec<Line<'_>> {
        let mut lines = Vec::new();

        for line in text.lines() {
            if line.trim().is_empty() {
                lines.push(Line::from(""));
                continue;
            }

            if let Some(rest) = line.strip_prefix("# ") {
                // H1: Centered, Yellow, Bold, Underlined
                lines.push(
                    Line::from(vec![Span::styled(
                        rest,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                    )])
                    .alignment(Alignment::Center),
                );
                lines.push(Line::from(""));
            } else if let Some(rest) = line.strip_prefix("## ") {
                // H2: Cyan, Bold
                lines.push(Line::from(vec![Span::styled(
                    rest,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]));
            } else {
                // Body text with **bold** parsing
                let mut spans = Vec::new();
                let mut current_text = line;

                while let Some(start_idx) = current_text.find("**") {
                    if start_idx > 0 {
                        spans.push(Span::styled(
                            &current_text[..start_idx],
                            Style::default().fg(Color::Gray),
                        ));
                    }

                    let rest = &current_text[start_idx + 2..];
                    if let Some(end_idx) = rest.find("**") {
                        let bold_text = &rest[..end_idx];
                        spans.push(Span::styled(
                            bold_text,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ));
                        current_text = &rest[end_idx + 2..];
                    } else {
                        // Unclosed **, treat as raw
                        spans.push(Span::styled(
                            &current_text[start_idx..],
                            Style::default().fg(Color::Gray),
                        ));
                        current_text = "";
                        break;
                    }
                }

                if !current_text.is_empty() {
                    spans.push(Span::styled(current_text, Style::default().fg(Color::Gray)));
                }

                lines.push(Line::from(spans));
            }
        }

        lines
    }

    #[allow(clippy::collapsible_if)]
    fn generate_story() -> Result<String, Box<dyn std::error::Error>> {
        let mut app = App::new_headless()?;
        register_story(&mut app);

        let root_id = app.world().resource::<Scene>().root();
        app.spawn(root_id).insert(NarrativeGenerator);
        app.update();

        let scene = app.world().resource::<Scene>();
        let root = scene.root();
        let root_node = scene.get_node(root).ok_or("Root node missing")?;

        if !root_node.children.is_empty() {
            for &child_id in &root_node.children {
                if let Some(child) = scene.get_node(child_id) {
                    if let NodeContent::Styled { style } = &child.content {
                        if let Some(text_content) = &style.text {
                            return Ok(text_content.text.clone());
                        }
                    }
                }
            }
        }

        Err("No narrative generated".into())
    }

    fn ui(f: &mut ratatui::Frame, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(f.area());

        // Title
        let title = Paragraph::new(" ✨ Nova Story Generator ✨ ")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Footer
        let footer_text = match state {
            AppState::Intro => " Press <ENTER> to Generate | q: Quit ".to_string(),
            AppState::Generating { .. } => " Generating... | q: Quit ".to_string(),
            AppState::Display {
                visible_chars,
                total_chars,
                ..
            } => {
                let progress = if *total_chars > 0 {
                    (*visible_chars as f32 / *total_chars as f32 * 100.0) as usize
                } else {
                    100
                };
                format!(
                    " Scroll: ↑/↓ | {}% Revealed | r: Retry | q: Quit ",
                    progress
                )
            }
            AppState::Error { .. } => " r: Retry | q: Quit ".to_string(),
        };
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);

        // Content
        let content_area = chunks[1];
        match state {
            AppState::Intro => {
                let text = vec![
                    Line::from("Welcome to the Narrative Engine."),
                    Line::from(""),
                    Line::from(
                        "This tool demonstrates the procedural story generation capabilities",
                    ),
                    Line::from("of the experimental 'Nova' feature set."),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Ready to weave a new tale?",
                        Style::default().fg(Color::Green),
                    )),
                ];
                let p = Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::NONE))
                    .wrap(Wrap { trim: true });

                // Center vertically
                let v_center = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(40),
                        Constraint::Percentage(20),
                        Constraint::Percentage(40),
                    ])
                    .split(content_area);

                f.render_widget(p, v_center[1]);
            }
            AppState::Generating { start_time } => {
                let elapsed = start_time.elapsed().as_millis();
                let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let i = (elapsed / 100) as usize % frames.len();
                let spinner = frames[i];

                let text = vec![Line::from(Span::styled(
                    format!("{} Weaving destiny...", spinner),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ))];
                let p = Paragraph::new(text).alignment(Alignment::Center);

                let v_center = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(45),
                        Constraint::Length(1),
                        Constraint::Percentage(45),
                    ])
                    .split(content_area);

                f.render_widget(p, v_center[1]);
            }
            AppState::Display {
                story,
                scroll_offset,
                visible_chars,
                ..
            } => {
                // Ensure we don't slice mid-char
                let safe_end = story
                    .char_indices()
                    .map(|(i, _)| i)
                    .nth(*visible_chars)
                    .unwrap_or(story.len());
                let sliced_story = &story[..safe_end];

                let text = parse_markdown(sliced_story);
                let p = Paragraph::new(text)
                    .block(
                        Block::default()
                            .title(" 📜 The Chronicle ")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Double)
                            .border_style(Style::default().fg(Color::Yellow))
                            .padding(Padding::new(2, 2, 1, 1)),
                    )
                    .wrap(Wrap { trim: true })
                    .scroll((*scroll_offset, 0));
                f.render_widget(p, content_area);

                // Render Scrollbar
                // We'll put it on the right edge of the content area
                // Since we don't know the full height easily, we'll just indicate current position
                // and pretend max is somewhat large or based on scroll_offset
                use ratatui::widgets::Scrollbar;
                use ratatui::widgets::ScrollbarOrientation;
                use ratatui::widgets::ScrollbarState;

                // Simple heuristic: content length = scroll_offset + visible height (approx) + some more
                // A better approach is to let the user scroll until the end, but Ratatui needs a max.
                // Let's assume a max height of 100 lines for now, or 2 * scroll_offset if > 50.
                let content_height = 100.max(*scroll_offset as usize + 20);
                let mut scrollbar_state =
                    ScrollbarState::new(content_height).position(*scroll_offset as usize);

                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("▲"))
                    .end_symbol(Some("▼"));

                f.render_stateful_widget(
                    scrollbar,
                    content_area.inner(ratatui::layout::Margin {
                        vertical: 1,
                        horizontal: 0,
                    }), // Adjust to be inside borders
                    &mut scrollbar_state,
                );
            }
            AppState::Error { message } => {
                let p = Paragraph::new(Span::styled(message, Style::default().fg(Color::Red)))
                    .block(Block::default().title(" Error ").borders(Borders::ALL));
                f.render_widget(p, content_area);
            }
        }
    }
}

#[cfg(feature = "nova")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tui_app::run()
}

#[cfg(not(feature = "nova"))]
fn main() {
    eprintln!("Error: This example requires the 'nova' feature.");
    eprintln!("Try running with:");
    eprintln!("    cargo run --example story_demo --features nova");
    std::process::exit(1);
}
