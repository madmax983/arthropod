use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use regex::Regex;
use std::{env, fs, io, time::Duration};

struct Benchmark {
    name: String,
    time_min: String,
    time_mean: String,
    time_max: String,
    throughput: Option<String>, // "43.142 Melem/s"
}

struct App {
    benchmarks: Vec<Benchmark>,
    state: ListState,
}

impl App {
    fn new(benchmarks: Vec<Benchmark>) -> Self {
        let mut state = ListState::default();
        if !benchmarks.is_empty() {
            state.select(Some(0));
        }
        Self { benchmarks, state }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.benchmarks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.benchmarks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn main() -> Result<()> {
    // 1. Parse arguments and read file
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "benchmark_results.txt"
    };

    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read benchmark file: {}", file_path))?;

    // 2. Parse benchmarks
    let benchmarks = parse_benchmarks(&content);
    if benchmarks.is_empty() {
        eprintln!("No benchmarks found in {}. Ensure the file contains Criterion output.", file_path);
        return Ok(());
    }

    // 3. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 4. Run App Loop
    let mut app = App::new(benchmarks);
    let res = run_app(&mut terminal, &mut app);

    // 5. Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}

fn parse_benchmarks(content: &str) -> Vec<Benchmark> {
    let mut benchmarks = Vec::new();

    // Regex for time line: "ecs_update/10           time:   [230.40 ns 231.79 ns 233.44 ns]"
    // Captures: 1: Name, 2: min val, 3: min unit, 4: mean val, 5: mean unit, 6: max val, 7: max unit
    let time_re = Regex::new(r"^(.*?)\s+time:\s+\[([\d\.]+ \w+) ([\d\.]+ \w+) ([\d\.]+ \w+)\]").unwrap();

    // Regex for throughput line: "                        thrpt:  [42.837 Melem/s 43.142 Melem/s 43.403 Melem/s]"
    // We only care about mean (middle value) for summary
    let thrpt_re = Regex::new(r"thrpt:\s+\[.*? ([\d\.]+ \w+/s) .*?\]").unwrap();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if let Some(caps) = time_re.captures(line) {
            let name = caps[1].trim().to_string();
            let time_min = caps[2].to_string();
            let time_mean = caps[3].to_string();
            let time_max = caps[4].to_string();

            // Look ahead for throughput
            let mut throughput = None;
            if i + 1 < lines.len() {
                if let Some(t_caps) = thrpt_re.captures(lines[i + 1]) {
                    throughput = Some(t_caps[1].to_string());
                }
            }

            benchmarks.push(Benchmark {
                name,
                time_min,
                time_mean,
                time_max,
                throughput,
            });
        }
        i += 1;
    }

    benchmarks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_benchmarks() {
        let content = r#"
Benchmarking ecs_update/10
Benchmarking ecs_update/10: Warming up for 3.0000 s
Benchmarking ecs_update/10: Collecting 100 samples in estimated 5.0011 s (21M iterations)
Benchmarking ecs_update/10: Analyzing
ecs_update/10           time:   [230.40 ns 231.79 ns 233.44 ns]
                        thrpt:  [42.837 Melem/s 43.142 Melem/s 43.403 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
"#;

        let benchmarks = parse_benchmarks(content);
        assert_eq!(benchmarks.len(), 1);
        let b = &benchmarks[0];
        assert_eq!(b.name, "ecs_update/10");
        assert_eq!(b.time_min, "230.40 ns");
        assert_eq!(b.time_mean, "231.79 ns");
        assert_eq!(b.time_max, "233.44 ns");
        assert_eq!(b.throughput.as_deref(), Some("43.142 Melem/s"));
    }
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(f.area());

    // Left Pane: List of Benchmarks
    let items: Vec<ListItem> = app.benchmarks
        .iter()
        .map(|b| {
            let content = Line::from(Span::raw(&b.name));
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Benchmarks"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.state);

    // Right Pane: Details
    if let Some(selected_index) = app.state.selected() {
        let b = &app.benchmarks[selected_index];

        let mut text = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&b.name),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Mean Time: ", Style::default().fg(Color::Green)),
                Span::raw(&b.time_mean),
            ]),
            Line::from(vec![
                Span::raw("Range:     "),
                Span::raw(format!("{} - {}", b.time_min, b.time_max)),
            ]),
        ];

        if let Some(ref thrpt) = b.throughput {
             text.push(Line::from(""));
             text.push(Line::from(vec![
                Span::styled("Throughput: ", Style::default().fg(Color::Yellow)),
                Span::raw(thrpt),
            ]));
        }

        // Add some fancy separator
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            "------------------------------------------------",
            Style::default().fg(Color::DarkGray),
        )));

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .style(Style::default());

        f.render_widget(paragraph, chunks[1]);
    } else {
        let paragraph = Paragraph::new("Select a benchmark to view details")
            .block(Block::default().borders(Borders::ALL).title("Details"));
        f.render_widget(paragraph, chunks[1]);
    }
}
