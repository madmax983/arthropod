#[cfg(feature = "nova")]
mod tui_app {
    use std::{
        io::{self, Stdout},
        time::{Duration, Instant},
    };

    use arthropod::experimental::story::{NarrativeGenerator, StoryRuntime, register_story};
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
        StoryLoop,
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        // Setup Terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

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
    fn run_app(
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize Arthropod App
        let mut app = App::new_headless()?;
        register_story(&mut app);

        // Spawn NarrativeGenerator entity attached to root
        let root_id = app.world().resource::<Scene>().root();
        app.spawn(root_id).insert(NarrativeGenerator);

        // Initial update to generate first frame
        app.update();

        let mut state = AppState::Intro;
        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();
        let mut scroll_offset = 0u16;

        loop {
            // Extract current text from Scene for rendering
            let story_data = extract_story_data(&app);

            terminal.draw(|f| ui(f, &state, &story_data, scroll_offset))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Enter => {
                                if let AppState::Intro = state {
                                    state = AppState::StoryLoop;
                                    app.update(); // Ensure fresh state
                                }
                            }
                            // Number keys for choices
                            KeyCode::Char(c) if c.is_ascii_digit() => {
                                if let AppState::StoryLoop = state {
                                    let idx = c.to_digit(10).unwrap() as usize;
                                    if idx > 0 {
                                        // 1-based index
                                        let mut runtime =
                                            app.world_mut().resource_mut::<StoryRuntime>();
                                        // Try to make a choice (0-based internally)
                                        if runtime.choose(idx - 1).is_ok() {
                                            // Reset scroll on new passage
                                            scroll_offset = 0;
                                        }
                                    }
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                scroll_offset = scroll_offset.saturating_add(1);
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                scroll_offset = scroll_offset.saturating_sub(1);
                            }
                            _ => {}
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
                // Run ECS systems
                app.update();
            }
        }
    }

    struct StoryData {
        passage: String,
        choices: Vec<String>,
    }

    fn extract_story_data(app: &App) -> StoryData {
        let scene = app.world().resource::<Scene>();
        let root = scene.root();
        let root_node = match scene.get_node(root) {
            Some(n) => n,
            None => {
                return StoryData {
                    passage: String::new(),
                    choices: Vec::new(),
                };
            }
        };

        let mut passage = String::new();
        let mut choices = Vec::new();

        // Iterate all children (Text + Choices)
        for (i, &child_id) in root_node.children.iter().enumerate() {
            if let Some(child) = scene.get_node(child_id)
                && let NodeContent::Styled { style } = &child.content
                && let Some(text_content) = &style.text
            {
                if i == 0 {
                    passage = text_content.text.clone();
                } else {
                    // Clean up choice text (remove leading newline if present)
                    choices.push(text_content.text.trim().to_string());
                }
            }
        }

        StoryData { passage, choices }
    }

    fn parse_markdown(text: &str) -> Vec<Line<'_>> {
        let mut lines = Vec::new();

        for line in text.lines() {
            if line.trim().is_empty() {
                lines.push(Line::from(""));
                continue;
            }

            if let Some(rest) = line.strip_prefix("# ") {
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
                lines.push(Line::from(vec![Span::styled(
                    rest,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]));
            } else if let Some(rest) = line.strip_prefix("> ") {
                // Blockquote
                lines.push(Line::from(vec![
                    Span::styled("▎ ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        rest,
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            } else {
                // Standard markdown body
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

    fn ui(f: &mut ratatui::Frame, state: &AppState, data: &StoryData, scroll_offset: u16) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Main Content (Passage + Choices)
                Constraint::Length(3), // Footer
            ])
            .split(f.area());

        // Title
        let title = Paragraph::new(" ✨ Nova Story Engine ✨ ")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Main Content Area
        let main_area = chunks[1];

        match state {
            AppState::Intro => {
                let text = vec![
                    Line::from("Welcome to the Nova Story Engine."),
                    Line::from(""),
                    Line::from("This is a fully reactive, ECS-driven narrative runtime."),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press <ENTER> to Begin",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::SLOW_BLINK),
                    )),
                ];
                let p = Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::NONE))
                    .wrap(Wrap { trim: true });

                let v_center = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(40),
                        Constraint::Percentage(20),
                        Constraint::Percentage(40),
                    ])
                    .split(main_area);

                f.render_widget(p, v_center[1]);
            }
            AppState::StoryLoop => {
                // Split Main Area into Passage (Top) and Choices (Bottom)
                let content_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(10), // Passage takes remaining space
                        Constraint::Length(data.choices.len() as u16 + 2), // Choices take fixed space
                    ])
                    .split(main_area);

                // 1. Passage
                let text = parse_markdown(&data.passage);
                let p = Paragraph::new(text)
                    .block(
                        Block::default()
                            .title(" 📜 The Chronicle ")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Double)
                            .padding(Padding::new(2, 2, 1, 1)),
                    )
                    .wrap(Wrap { trim: true })
                    .scroll((scroll_offset, 0));
                f.render_widget(p, content_chunks[0]);

                // 2. Choices
                use ratatui::widgets::{List, ListItem};
                let choice_items: Vec<ListItem> = data
                    .choices
                    .iter()
                    .map(|c| {
                        // Highlight the [N] part if present
                        let spans = if let Some((prefix, rest)) = c.split_once(' ') {
                            if prefix.starts_with('[') && prefix.ends_with(']') {
                                vec![
                                    Span::styled(
                                        prefix,
                                        Style::default()
                                            .fg(Color::Yellow)
                                            .add_modifier(Modifier::BOLD),
                                    ),
                                    Span::raw(" "),
                                    Span::styled(rest, Style::default().fg(Color::Cyan)),
                                ]
                            } else {
                                vec![Span::styled(c, Style::default().fg(Color::Cyan))]
                            }
                        } else {
                            vec![Span::styled(c, Style::default().fg(Color::Cyan))]
                        };
                        ListItem::new(Line::from(spans))
                    })
                    .collect();

                let choices_list = List::new(choice_items).block(
                    Block::default()
                        .title(" Actions ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                );
                f.render_widget(choices_list, content_chunks[1]);
            }
        }

        // Footer
        let footer_text = match state {
            AppState::Intro => " q: Quit ".to_string(),
            AppState::StoryLoop => " Select Choice: [1-9] | Scroll: ↑/↓ | q: Quit ".to_string(),
        };
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
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
