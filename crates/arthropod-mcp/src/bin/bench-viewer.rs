use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use regex::Regex;
use serde::Serialize;
use std::{collections::HashMap, env, fmt, fs, io, time::Duration};

#[derive(Debug, Clone, Serialize)]
struct DurationVal {
    value: f64,
    unit: String,
    nanos: f64,
}

impl fmt::Display for DurationVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.value, self.unit)
    }
}

#[derive(Debug, Clone, Serialize)]
struct Benchmark {
    name: String,
    group: String,
    variant: String,
    time_min: DurationVal,
    time_mean: DurationVal,
    time_max: DurationVal,
    throughput: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SortBy {
    Name,
    Time,
    Group,
}

struct App {
    benchmarks: Vec<Benchmark>,
    state: TableState,
    group_stats: HashMap<String, f64>,
    sort_by: SortBy,
    sort_desc: bool,
}

impl App {
    fn new(benchmarks: Vec<Benchmark>) -> Self {
        let mut state = TableState::default();
        if !benchmarks.is_empty() {
            state.select(Some(0));
        }

        let mut group_stats = HashMap::new();
        for b in &benchmarks {
            let max = group_stats.entry(b.group.clone()).or_insert(0.0);
            if b.time_mean.nanos > *max {
                *max = b.time_mean.nanos;
            }
        }

        Self {
            benchmarks,
            state,
            group_stats,
            sort_by: SortBy::Group,
            sort_desc: false,
        }
    }

    fn sort(&mut self) {
        self.benchmarks.sort_by(|a, b| {
            let order = match self.sort_by {
                SortBy::Name => a.name.cmp(&b.name),
                SortBy::Group => match a.group.cmp(&b.group) {
                    std::cmp::Ordering::Equal => a.variant.cmp(&b.variant),
                    ord => ord,
                },
                SortBy::Time => a.time_mean.nanos.partial_cmp(&b.time_mean.nanos).unwrap(),
            };
            if self.sort_desc {
                order.reverse()
            } else {
                order
            }
        });
        // Reset selection to top after sort to avoid out of bounds or confusing jump
        if !self.benchmarks.is_empty() {
            self.state.select(Some(0));
        }
    }

    fn toggle_sort(&mut self, new_sort: SortBy) {
        if self.sort_by == new_sort {
            self.sort_desc = !self.sort_desc;
        } else {
            self.sort_by = new_sort;
            self.sort_desc = false;
        }
        self.sort();
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
    let json_output = args.iter().any(|arg| arg == "--json");

    // Naive argument parsing for file path
    let file_path = args
        .iter()
        .skip(1)
        .find(|arg| *arg != "--json")
        .map(|s| s.as_str())
        .unwrap_or("benchmark_results.txt");

    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read benchmark file: {}", file_path))?;

    // 2. Parse benchmarks
    let benchmarks = parse_benchmarks(&content);

    if json_output {
        let json = serde_json::to_string_pretty(&benchmarks)?;
        println!("{}", json);
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

fn parse_duration(val_str: &str, unit_str: &str) -> DurationVal {
    let value: f64 = val_str.parse().unwrap_or(0.0);
    // Normalize to nanoseconds for comparison
    let nanos = match unit_str {
        "ns" => value,
        "µs" | "us" => value * 1_000.0,
        "ms" => value * 1_000_000.0,
        "s" => value * 1_000_000_000.0,
        _ => value,
    };
    DurationVal {
        value,
        unit: unit_str.to_string(),
        nanos,
    }
}

fn parse_benchmarks(content: &str) -> Vec<Benchmark> {
    let mut benchmarks = Vec::new();

    let time_re = Regex::new(
        r"^(.*?)\s+time:\s+\[([\d\.]+)\s+(\w+)\s+([\d\.]+)\s+(\w+)\s+([\d\.]+)\s+(\w+)\]",
    )
    .unwrap();

    let thrpt_re = Regex::new(r"thrpt:\s+\[.*? ([\d\.]+ \w+/s) .*?\]").unwrap();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if let Some(caps) = time_re.captures(line) {
            let name = caps[1].trim().to_string();

            let (group, variant) = if let Some((g, v)) = name.rsplit_once('/') {
                (g.to_string(), v.to_string())
            } else {
                (name.clone(), "-".to_string())
            };

            let time_min = parse_duration(&caps[2], &caps[3]);
            let time_mean = parse_duration(&caps[4], &caps[5]);
            let time_max = parse_duration(&caps[6], &caps[7]);

            let mut throughput = None;
            if i + 1 < lines.len() {
                if let Some(t_caps) = thrpt_re.captures(lines[i + 1]) {
                    throughput = Some(t_caps[1].to_string());
                }
            }

            benchmarks.push(Benchmark {
                name,
                group,
                variant,
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
                        KeyCode::Char('n') => app.toggle_sort(SortBy::Name),
                        KeyCode::Char('t') => app.toggle_sort(SortBy::Time),
                        KeyCode::Char('g') => app.toggle_sort(SortBy::Group),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    if app.benchmarks.is_empty() {
        let vertical_center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ])
            .split(f.area());
        let horizontal_center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(vertical_center[1]);

        let text = vec![
            Line::from(Span::styled(
                "No benchmarks found.",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Run `cargo bench` to generate results."),
            Line::from(""),
            Line::from("Press 'q' to quit."),
        ];

        let paragraph = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Arthropod Bench Viewer "),
            );

        f.render_widget(paragraph, horizontal_center[1]);
        return;
    }

    // Top-level layout: Header, Main, Footer
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // 1. Header
    let header_text = format!(
        " Arthropod Benchmark Viewer | Loaded {} benchmarks ",
        app.benchmarks.len()
    );
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, vertical_chunks[0]);

    // 2. Main Content Layout
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_chunks[1]);

    // Left Pane: Table of Benchmarks
    let headers = [
        ("Group", SortBy::Group),
        ("Variant", SortBy::Group), // Variant doesn't have its own sort, it follows Group
        ("Time", SortBy::Time),
        ("Throughput", SortBy::Time), // Throughput correlates with Time
    ];

    let header_cells = headers.iter().map(|(title, sort_key)| {
        let mut style = Style::default().fg(Color::Yellow);
        let mut text = title.to_string();

        if app.sort_by == *sort_key {
            style = style.fg(Color::Cyan).add_modifier(Modifier::BOLD);
            let arrow = if app.sort_desc { " ▼" } else { " ▲" };
            text.push_str(arrow);
        }

        // Handle "Name" sort which isn't explicitly a column but we map it conceptually?
        // Actually, we added SortBy::Name but didn't add a Name column (it's split into Group/Variant).
        // If sorting by Name, maybe highlight Group and Variant?
        if app.sort_by == SortBy::Name && (*title == "Group" || *title == "Variant") {
            style = style.fg(Color::Cyan).add_modifier(Modifier::BOLD);
            if *title == "Group" {
                let arrow = if app.sort_desc { " ▼" } else { " ▲" };
                text.push_str(arrow);
            }
        }

        Cell::from(text).style(style)
    });

    let header_row = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let rows = app.benchmarks.iter().map(|b| {
        let thrpt = b.throughput.as_deref().unwrap_or("-");
        let cells = vec![
            Cell::from(b.group.clone()),
            Cell::from(b.variant.clone()),
            Cell::from(b.time_mean.to_string()),
            Cell::from(thrpt.to_string()),
        ];
        Row::new(cells).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(header_row)
    .block(Block::default().borders(Borders::ALL).title("Benchmarks"))
    .row_highlight_style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan),
    )
    .highlight_symbol(">> ");

    f.render_stateful_widget(table, main_chunks[0], &mut app.state);

    // Right Pane: Details & Chart
    if let Some(selected_index) = app.state.selected() {
        let b = &app.benchmarks[selected_index];

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(main_chunks[1]);

        let mut text = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&b.name),
            ]),
            Line::from(vec![
                Span::styled("Group: ", Style::default().fg(Color::Blue)),
                Span::raw(&b.group),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Mean Time: ", Style::default().fg(Color::Green)),
                Span::raw(b.time_mean.to_string()),
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

        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            "------------------------------------------------",
            Style::default().fg(Color::DarkGray),
        )));

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .style(Style::default());

        f.render_widget(paragraph, right_chunks[0]);

        // BarChart for Group Comparison
        let group_benchmarks: Vec<&Benchmark> = app
            .benchmarks
            .iter()
            .filter(|bm| bm.group == b.group)
            .collect();

        let max_nanos = *app.group_stats.get(&b.group).unwrap_or(&1.0);

        // Determine label (unit) based on max
        let unit_label = if max_nanos > 1_000_000_000.0 {
            "s"
        } else if max_nanos > 1_000_000.0 {
            "ms"
        } else if max_nanos > 1_000.0 {
            "µs"
        } else {
            "ns"
        };

        let divisor = match unit_label {
            "s" => 1_000_000_000.0,
            "ms" => 1_000_000.0,
            "µs" => 1_000.0,
            _ => 1.0,
        };

        let scaled_data: Vec<(&str, u64)> = group_benchmarks
            .iter()
            .map(|bm| (bm.variant.as_str(), (bm.time_mean.nanos / divisor) as u64))
            .collect();

        let barchart = BarChart::default()
            .block(
                Block::default()
                    .title(format!("Comparison ({})", unit_label))
                    .borders(Borders::ALL),
            )
            .data(&scaled_data)
            .bar_width(8)
            .bar_gap(1)
            .bar_style(Style::default().fg(Color::Cyan))
            .value_style(Style::default().fg(Color::Black).bg(Color::Cyan));

        f.render_widget(barchart, right_chunks[1]);
    } else {
        let paragraph = Paragraph::new("Select a benchmark to view details")
            .block(Block::default().borders(Borders::ALL).title("Details"));
        f.render_widget(paragraph, main_chunks[1]);
    }

    // 3. Footer
    let footer_text = " n: Name | g: Group | t: Time | ↑/↓: Navigate | q: Quit ";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, vertical_chunks[2]);
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
        assert_eq!(b.group, "ecs_update");
        assert_eq!(b.variant, "10");
        assert_eq!(b.time_min.value, 230.40);
        assert_eq!(b.time_min.unit, "ns");
        assert_eq!(b.time_mean.value, 231.79);
        assert_eq!(b.throughput.as_deref(), Some("43.142 Melem/s"));
    }
}
