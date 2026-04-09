//! Window types and traits.

use crate::materials::BackdropMaterial;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// Opaque window identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WindowId(pub(crate) u64);

impl WindowId {
    /// Create a test WindowId (for unit tests only)
    #[cfg(test)]
    pub fn test_id() -> Self {
        WindowId(0)
    }
}

/// A 2D spatial dimension container representing a bounding box's physical limits.
///
/// We decouple `Size` from native OS APIs (like `winit`'s PhysicalSize) so our core
/// platform traits can remain independent of backend windowing implementations.
///
/// ## Examples
/// ```
/// use plat_core::Size;
/// let screen_bounds = Size::new(1920, 1080);
/// assert_eq!(screen_bounds.width, 1920);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(missing_docs)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    /// Creates a new `Size` with the given width and height.
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

/// A 2D integer-based spatial coordinate representing physical pixels on a monitor.
///
/// Used for OS-level window placement where sub-pixel rendering is impossible
/// (e.g., "Place the window at exactly monitor pixel 100, 100").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(missing_docs)]
pub struct Position<T> {
    pub x: T,
    pub y: T,
}

impl<T> Position<T> {
    /// Creates a new `Position` with the given X and Y coordinates.
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

/// A 2D floating-point coordinate for sub-pixel precision.
///
/// Used internally for high-DPI (Retina) cursor tracking and gesture math
/// where a physical mouse movement might cross fractional logic points.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(missing_docs)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    /// Creates a new `Point` with the given X and Y coordinates.
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

/// A 2D bounding box representing an axis-aligned rectangle.
///
/// Used extensively in hit-testing, layout clipping, and dirty region tracking.
/// By maintaining our own `Rect` type, we ensure it maps exactly to our rendering
/// backend's coordinate system (origin Top-Left).
///
/// ## Examples
/// ```
/// use plat_core::Rect;
/// let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
/// assert!(bounds.contains(50.0, 50.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[allow(missing_docs)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Creates a new `Rect` with the given position and size.
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

/// A declarative builder payload for configuring an OS window before creation.
///
/// Because OS windows are expensive and context-heavy (requiring COM on Windows
/// or NSWindow on Mac), we package all desired state into this configuration struct
/// rather than making dozens of individual `set_foo()` calls over the FFI boundary
/// after creation.
///
/// ## Examples
///
/// ```
/// use plat_core::{WindowConfig, Size};
///
/// let config = WindowConfig {
///     title: "Arthropod App".into(),
///     size: Size::new(1024, 768),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct WindowConfig {
    pub title: String,
    pub size: Size<u32>,
    pub position: Option<Position<i32>>,
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub visible: bool,
    /// Enable DirectComposition mode for selective transparency.
    ///
    /// When true, the window uses DirectComposition visual trees instead of
    /// standard wgpu swap chains. This enables per-region backdrop materials
    /// but requires Windows 10 version 1803 or newer.
    pub composition_mode: bool,
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
            composition_mode: false, // Opt-in for now
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

impl Window {
    /// Modifies the OS-level rendering material for the window background.
    ///
    /// This is what allows effects like "Mica" or "Acrylic" on Windows, or
    /// "Vibrancy" on macOS. The material is composited by the desktop window
    /// manager behind the application's rendered content.
    pub fn set_backdrop_material(&self, material: BackdropMaterial) {
        self.inner.set_backdrop_material(material);
    }

    /// Gets the current backdrop material configured for the window.
    pub fn backdrop_material(&self) -> BackdropMaterial {
        self.inner.backdrop_material()
    }
}
