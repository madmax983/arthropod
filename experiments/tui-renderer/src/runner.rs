use crate::TuiBackend;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use render_engine::Scene;
use std::io;
use std::time::Duration;

/// Terminal event loop and lifecycle manager.
///
/// `TuiRunner` abstracts away the boilerplate of setting up a raw-mode terminal,
/// managing the alternate screen buffer, and running the main event loop. It captures
/// terminal input events (like keystrokes) and forwards them to a user-provided callback.
///
/// ## Examples
///
/// ```rust,no_run
/// use tui_renderer::TuiRunner;
/// use render_engine::Scene;
/// use crossterm::event::Event;
///
/// # fn main() -> std::io::Result<()> {
/// let scene = Scene::new();
///
/// // Run the event loop, passing input events to the closure
/// TuiRunner::run(scene, |scene_mut, event| {
///     // Handle event and update scene...
///
///     // Return true to continue, false to exit
///     true
/// })?;
/// # Ok(())
/// # }
/// ```
pub struct TuiRunner;

impl TuiRunner {
    /// Starts the terminal event loop and takes over the current thread.
    ///
    /// This method will:
    /// 1. Enter raw mode and the alternate screen buffer.
    /// 2. Initialize a default [`TuiBackend`] using standard output.
    /// 3. Enter a render loop running at approximately 60 FPS (16ms polling).
    /// 4. Intercept terminal events (`crossterm::event::Event`) and pass them to `update_fn`.
    /// 5. Automatically handle graceful shutdown when `update_fn` returns `false` or `Ctrl+C` is pressed.
    /// 6. Restore the terminal to its previous state upon exit.
    ///
    /// ## Arguments
    /// * `scene` - The initial visual [`Scene`] to render.
    /// * `update_fn` - A closure called when an input event occurs. It receives mutable access to the scene
    ///   so it can apply updates based on the event. It must return a boolean: `true` to continue the loop,
    ///   or `false` to gracefully exit.
    ///
    /// ## Errors
    /// Returns an [`io::Error`] if terminal setup, teardown, rendering, or event polling fails.
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
