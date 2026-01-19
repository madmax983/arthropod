//! Window types and traits.

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// Opaque window identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub(crate) u64);

/// Size in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

/// Position in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position<T> {
    pub x: T,
    pub y: T,
}

impl<T> Position<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

/// Point (position with floating point coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

/// Rectangle with position and size.
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if a point is inside this rectangle.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    /// Compute the union of this rectangle with another.
    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);
        Rect::new(x, y, right - x, bottom - y)
    }
}

/// Window configuration for creation.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub size: Size<u32>,
    pub position: Option<Position<i32>>,
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub visible: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Arthropod".into(),
            size: Size::new(800, 600),
            position: None,
            resizable: true,
            decorations: true,
            transparent: false,
            visible: true,
        }
    }
}

/// A platform window.
pub struct Window {
    pub(crate) inner: crate::platform::WindowImpl,
}

impl Window {
    /// Get the window's unique identifier.
    pub fn id(&self) -> WindowId {
        self.inner.id()
    }

    /// Get current inner size (content area) in physical pixels.
    pub fn inner_size(&self) -> Size<u32> {
        self.inner.inner_size()
    }

    /// Set the window title.
    pub fn set_title(&self, title: &str) {
        self.inner.set_title(title);
    }

    /// Request a redraw of the window.
    pub fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    /// Get the scale factor (DPI scaling).
    pub fn scale_factor(&self) -> f64 {
        self.inner.scale_factor()
    }

    /// Set window visibility.
    pub fn set_visible(&self, visible: bool) {
        self.inner.set_visible(visible);
    }
}

impl HasWindowHandle for Window {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.inner.window_handle()
    }
}

impl HasDisplayHandle for Window {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.inner.display_handle()
    }
}
