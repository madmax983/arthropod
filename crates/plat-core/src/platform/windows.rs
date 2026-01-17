//! Windows platform implementation using Win32 APIs.

use crate::{
    Application, ControlFlow, Event, PlatformError, Point, Size, Window, WindowConfig, WindowEvent,
    WindowId,
};
use raw_window_handle::{
    DisplayHandle, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
    Win32WindowHandle, WindowHandle, WindowsDisplayHandle,
};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use windows::{
    Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*, core::*,
};

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);

// Thread-local event queue for dispatching Win32 messages as Events
thread_local! {
    static EVENT_QUEUE: RefCell<Vec<Event>> = const { RefCell::new(Vec::new()) };
}

/// Windows event loop implementation.
pub struct EventLoopImpl {
    hinstance: HINSTANCE,
}

impl EventLoopImpl {
    pub fn new() -> std::result::Result<Self, PlatformError> {
        unsafe {
            let hinstance = GetModuleHandleW(None)
                .map_err(|e| {
                    PlatformError::Initialization(format!("GetModuleHandleW failed: {}", e))
                })?
                .into();
            Ok(Self { hinstance })
        }
    }

    pub fn create_window(
        &self,
        config: WindowConfig,
    ) -> std::result::Result<Window, PlatformError> {
        WindowImpl::new(self.hinstance, config).map(|inner| Window { inner })
    }
}

/// Windows window implementation.
pub struct WindowImpl {
    hwnd: HWND,
    // Keep hinstance alive for window lifetime
    #[allow(dead_code)]
    hinstance: HINSTANCE,
    id: WindowId,
}

// SAFETY: HWND is thread-safe to share across threads (it's just a handle).
// Windows API allows HWND to be used from any thread.
unsafe impl Sync for WindowImpl {}

impl WindowImpl {
    fn new(hinstance: HINSTANCE, config: WindowConfig) -> std::result::Result<Self, PlatformError> {
        unsafe {
            // Register window class
            let class_name = w!("ArthropodWindow");

            let wc = WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                hInstance: hinstance,
                lpszClassName: class_name,
                style: CS_HREDRAW | CS_VREDRAW,
                hCursor: LoadCursorW(None, IDC_ARROW).ok().unwrap_or_default(),
                hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as _),
                ..Default::default()
            };

            let _ = RegisterClassW(&wc);

            let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::SeqCst));

            let title: Vec<u16> = config
                .title
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                PCWSTR(title.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                config.size.width as i32,
                config.size.height as i32,
                None,
                None,
                Some(hinstance),
                None,
            )
            .map_err(|e| PlatformError::WindowCreation(format!("CreateWindowExW failed: {}", e)))?;

            if config.visible {
                let _ = ShowWindow(hwnd, SW_SHOW);
            }

            Ok(Self {
                hwnd,
                hinstance,
                id,
            })
        }
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn inner_size(&self) -> Size<u32> {
        unsafe {
            let mut rect = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut rect);
            Size::new(
                (rect.right - rect.left) as u32,
                (rect.bottom - rect.top) as u32,
            )
        }
    }

    pub fn set_title(&self, title: &str) {
        unsafe {
            let title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = SetWindowTextW(self.hwnd, PCWSTR(title.as_ptr()));
        }
    }

    pub fn request_redraw(&self) {
        unsafe {
            let _ = InvalidateRect(Some(self.hwnd), None, false);
        }
    }

    pub fn scale_factor(&self) -> f64 {
        // For now, return 1.0. Full DPI support will be added later.
        1.0
    }

    pub fn set_visible(&self, visible: bool) {
        unsafe {
            let _ = ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }
}

impl HasWindowHandle for WindowImpl {
    fn window_handle(
        &self,
    ) -> std::result::Result<WindowHandle<'_>, raw_window_handle::HandleError> {
        use std::num::NonZeroIsize;
        let handle = Win32WindowHandle::new(NonZeroIsize::new(self.hwnd.0 as isize).unwrap());
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Win32(handle)) })
    }
}

impl HasDisplayHandle for WindowImpl {
    fn display_handle(
        &self,
    ) -> std::result::Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
        let handle = WindowsDisplayHandle::new();
        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Windows(handle)) })
    }
}

/// Window procedure callback.
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_SIZE => {
            // Extract new window size from lparam
            let width = (lparam.0 & 0xFFFF) as u32;
            let height = ((lparam.0 >> 16) & 0xFFFF) as u32;

            // Queue the resize event
            EVENT_QUEUE.with(|queue| {
                queue.borrow_mut().push(Event::Window {
                    window_id: WindowId(0), // TODO: Get actual window ID
                    event: WindowEvent::Resized(Size { width, height }),
                });
            });

            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            // Extract mouse coordinates from lparam
            let x = (lparam.0 & 0xFFFF) as i16 as f64;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as f64;

            // Queue the event
            EVENT_QUEUE.with(|queue| {
                queue.borrow_mut().push(Event::Window {
                    window_id: WindowId(0), // TODO: Get actual window ID
                    event: WindowEvent::CursorMoved {
                        position: Point { x, y },
                    },
                });
            });

            LRESULT(0)
        }
        WM_CLOSE => unsafe {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        },
        WM_DESTROY => unsafe {
            PostQuitMessage(0);
            LRESULT(0)
        },
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// Run the application.
pub fn run<A: Application>() -> std::result::Result<(), PlatformError> {
    let event_loop = crate::EventLoop::new()?;
    let mut app = A::new(&event_loop);
    let mut control_flow = ControlFlow::Poll;

    // Trigger initial redraw
    // For Phase 1, we'll just continuously redraw
    // TODO: Only redraw when needed based on WM_PAINT or explicit requests

    unsafe {
        let mut msg = MSG::default();
        loop {
            if control_flow == ControlFlow::Exit {
                break;
            }

            let has_msg = if control_flow == ControlFlow::Wait {
                GetMessageW(&mut msg, None, 0, 0).as_bool()
            } else {
                PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool()
            };

            if has_msg {
                if msg.message == WM_QUIT {
                    break;
                }

                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }

            // Dispatch any queued events
            let events: Vec<Event> =
                EVENT_QUEUE.with(|queue| queue.borrow_mut().drain(..).collect());

            for event in events {
                app.on_event(event, &mut control_flow);
            }

            // Always trigger redraw in Poll mode (even if we processed a message)
            // For Phase 1: Call on_redraw continuously
            // TODO: Only redraw when needed
            if control_flow == ControlFlow::Poll {
                app.on_redraw(crate::WindowId(0));
            }
        }
    }

    Ok(())
}
