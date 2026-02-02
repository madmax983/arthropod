use super::core::{App, AppError};
use plat_core::{EventLoop, WindowConfig};
use render_engine::backend::WgpuBackend;

/// Arthropod application builder
///
/// Use this to configure and create an `App` instance.
///
/// # Example
///
/// ```
/// use arthropod::prelude::*;
///
/// let config = WindowConfig {
///     title: "My App".to_string(),
///     size: Size { width: 800, height: 600 },
///     ..Default::default()
/// };
///
/// // For testing/headless mode
/// let app = AppBuilder::new()
///     .with_window_config(config)
///     .build_headless()
///     .expect("Failed to create app");
/// ```
pub struct AppBuilder {
    window_config: Option<WindowConfig>,
}

impl AppBuilder {
    /// Create a new AppBuilder
    pub fn new() -> Self {
        Self {
            window_config: None,
        }
    }

    /// Set the window configuration
    pub fn with_window_config(mut self, config: WindowConfig) -> Self {
        self.window_config = Some(config);
        self
    }

    /// Build the application with a real window and GPU backend
    ///
    /// Requires an EventLoop to create the window.
    /// Use this for real applications.
    pub fn build(self, event_loop: &EventLoop) -> Result<App, AppError> {
        let config = self.window_config.unwrap_or_default();

        // Create window
        let window = event_loop
            .create_window(config)
            .map_err(|e| AppError::WindowCreation(e.to_string()))?;

        let size = window.inner_size();

        // Create GPU backend (standard mode - not using DirectComposition)
        // SAFETY: Safe because `App` struct guarantees correct drop order (context before window).
        let backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .map_err(|e| AppError::BackendCreation(e.to_string()))?;

        // Create app with backend
        Ok(App::new_with_backend(Some(window), Some(backend)))
    }

    /// Build the application in headless mode (no window, no GPU)
    ///
    /// Use this for testing and benchmarking.
    /// render() will collect instances but not actually render to GPU.
    pub fn build_headless(self) -> Result<App, AppError> {
        Ok(App::new_with_backend(None, None))
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}
