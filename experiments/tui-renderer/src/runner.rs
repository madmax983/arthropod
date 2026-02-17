use crate::TuiBackend;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use render_engine::Scene;
use render_engine::backend::RenderBackend;
use std::io;
use std::time::Duration;

pub struct TuiRunner;

impl TuiRunner {
    pub fn run(
        mut scene: Scene,
        mut update_fn: impl FnMut(&mut Scene, Event) -> bool,
    ) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        // Create backend (uses default CrosstermBackend)
        let mut backend = TuiBackend::new().map_err(|e| io::Error::other(e.to_string()))?;

        // Run loop
        let res = (|| -> io::Result<()> {
            loop {
                // Resize based on terminal size
                // Note: backend.render() in ratatui uses the current terminal size,
                // but our backend trait has resize().
                // We should call it so the backend knows the size if it stores it.
                if let Ok(size) = crossterm::terminal::size() {
                    backend.resize(size.0 as u32, size.1 as u32);
                }

                // Render
                backend
                    .render(&scene)
                    .map_err(|e| io::Error::other(e.to_string()))?;

                // Poll events
                if event::poll(Duration::from_millis(16))? {
                    let event = event::read()?;

                    // Default exit on Ctrl+C
                    if let Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) = event
                    {
                        break;
                    }

                    // User update callback
                    // Returns false to exit loop
                    if !update_fn(&mut scene, event) {
                        break;
                    }
                }
            }
            Ok(())
        })();

        // Cleanup
        disable_raw_mode()?;
        execute!(stdout, LeaveAlternateScreen)?;

        if let Err(e) = res {
            eprintln!("Error: {}", e);
            return Err(e);
        }

        Ok(())
    }
}
