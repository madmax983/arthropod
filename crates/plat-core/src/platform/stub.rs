//! Stub implementation for unsupported platforms.

use crate::{Application, BackdropMaterial, PlatformError, Size, Window, WindowConfig, WindowId};
use raw_window_handle::{DisplayHandle, HasDisplayHandle, HasWindowHandle, WindowHandle};

pub struct EventLoopImpl;

impl EventLoopImpl {
    pub fn new() -> Result<Self, PlatformError> {
        Err(PlatformError::Initialization(
            "The current platform is not supported. Arthropod currently supports Windows and macOS.".into(),
        ))
    }

    pub fn create_window(&self, _config: WindowConfig) -> Result<Window, PlatformError> {
        Err(PlatformError::WindowCreation(
            "The current platform is not supported. Arthropod currently supports Windows and macOS.".into(),
        ))
    }
}

pub struct WindowImpl;

impl WindowImpl {
    pub fn id(&self) -> WindowId {
        WindowId(0)
    }

    pub fn inner_size(&self) -> Size<u32> {
        Size::new(0, 0)
    }

    pub fn set_title(&self, _title: &str) {}

    pub fn request_redraw(&self) {}

    pub fn scale_factor(&self) -> f64 {
        1.0
    }

    pub fn set_visible(&self, _visible: bool) {}

    pub fn set_backdrop_material(&self, _material: BackdropMaterial) {}

    pub fn backdrop_material(&self) -> BackdropMaterial {
        BackdropMaterial::None
    }
}

impl HasWindowHandle for WindowImpl {
    fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
        Err(raw_window_handle::HandleError::Unavailable)
    }
}

impl HasDisplayHandle for WindowImpl {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
        Err(raw_window_handle::HandleError::Unavailable)
    }
}

#[allow(clippy::extra_unused_type_parameters)]
pub fn run<A: Application>() -> Result<(), PlatformError> {
    Err(PlatformError::Initialization(
        "The current platform is not supported. Arthropod currently supports Windows and macOS.".into(),
    ))
}
