//! macOS platform implementation using Cocoa via objc2.

use crate::{Application, PlatformError, Size, Window, WindowConfig, WindowId};
use objc2::ClassType;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{CGFloat, MainThreadMarker, NSPoint, NSRect, NSSize, NSString};
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, DisplayHandle, HasDisplayHandle, HasWindowHandle,
    RawDisplayHandle, RawWindowHandle, WindowHandle,
};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);

pub struct EventLoopImpl {
    app: Retained<NSApplication>,
    mtm: MainThreadMarker,
}

impl EventLoopImpl {
    pub fn new() -> std::result::Result<Self, PlatformError> {
        // SAFETY: This must be called on the main thread
        let mtm = MainThreadMarker::new()
            .ok_or_else(|| PlatformError::Initialization("Not on main thread".into()))?;

        let app = NSApplication::sharedApplication(mtm);

        // Set activation policy to regular app
        app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

        Ok(Self { app, mtm })
    }

    pub fn create_window(
        &self,
        config: WindowConfig,
    ) -> std::result::Result<Window, PlatformError> {
        let window_impl = WindowImpl::new(&config, self.mtm)?;
        Ok(Window { inner: window_impl })
    }
}

pub struct WindowImpl {
    window: Retained<NSWindow>,
    id: WindowId,
    mtm: MainThreadMarker,
}

impl WindowImpl {
    fn new(
        config: &WindowConfig,
        mtm: MainThreadMarker,
    ) -> std::result::Result<Self, PlatformError> {
        let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::SeqCst));

        // Create window style mask
        let mut style_mask = NSWindowStyleMask::empty();
        if config.decorations {
            style_mask |= NSWindowStyleMask::Titled
                | NSWindowStyleMask::Closable
                | NSWindowStyleMask::Miniaturizable;
        } else {
            style_mask |= NSWindowStyleMask::Borderless;
        }

        if config.resizable {
            style_mask |= NSWindowStyleMask::Resizable;
        }

        // Create content rect
        let content_rect = NSRect {
            origin: if let Some(pos) = config.position {
                NSPoint {
                    x: pos.x as CGFloat,
                    y: pos.y as CGFloat,
                }
            } else {
                // Center on screen
                NSPoint { x: 0.0, y: 0.0 }
            },
            size: NSSize {
                width: config.size.width as CGFloat,
                height: config.size.height as CGFloat,
            },
        };

        // Create window
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                mtm.alloc(),
                content_rect,
                style_mask,
                NSBackingStoreType::Buffered,
                false,
            )
        };

        // Set title
        let title = NSString::from_str(&config.title);
        window.setTitle(&title);

        // Center window if no position specified
        if config.position.is_none() {
            window.center();
        }

        // Set visibility
        if config.visible {
            window.makeKeyAndOrderFront(None);
        }

        Ok(Self { window, id, mtm })
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn inner_size(&self) -> Size<u32> {
        let content_rect = self.window.contentRectForFrameRect(self.window.frame());
        Size {
            width: content_rect.size.width as u32,
            height: content_rect.size.height as u32,
        }
    }

    pub fn set_title(&self, title: &str) {
        let title = NSString::from_str(title);
        self.window.setTitle(&title);
    }

    pub fn request_redraw(&self) {
        // On macOS, we can mark the content view as needing display
        if let Some(content_view) = self.window.contentView() {
            content_view.setNeedsDisplay(true);
        }
    }

    pub fn scale_factor(&self) -> f64 {
        self.window.backingScaleFactor() as f64
    }

    pub fn set_visible(&self, visible: bool) {
        if visible {
            self.window.makeKeyAndOrderFront(None);
        } else {
            self.window.orderOut(None);
        }
    }
}

impl HasWindowHandle for WindowImpl {
    fn window_handle(
        &self,
    ) -> std::result::Result<WindowHandle<'_>, raw_window_handle::HandleError> {
        let ns_view = self
            .window
            .contentView()
            .ok_or(raw_window_handle::HandleError::Unavailable)?;
        let view_ptr = Retained::as_ptr(&ns_view) as *mut std::ffi::c_void;

        let non_null = NonNull::new(view_ptr).ok_or(raw_window_handle::HandleError::Unavailable)?;
        let handle = AppKitWindowHandle::new(non_null);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AppKit(handle)) })
    }
}

impl HasDisplayHandle for WindowImpl {
    fn display_handle(
        &self,
    ) -> std::result::Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
        let handle = AppKitDisplayHandle::new();
        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::AppKit(handle)) })
    }
}

pub fn run<A: Application>() -> std::result::Result<(), PlatformError> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| PlatformError::Initialization("Not on main thread".into()))?;

    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    // Create event loop
    let event_loop = crate::EventLoop {
        inner: EventLoopImpl {
            app: app.clone(),
            mtm,
        },
    };

    // Create application
    let mut user_app = A::new(&event_loop);

    // Activate app
    app.activateIgnoringOtherApps(true);

    // Simple run loop - in a real implementation, we'd handle events properly
    // For Phase 1, this is a minimal stub to allow window creation
    Ok(())
}
