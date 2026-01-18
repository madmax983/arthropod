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
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Once;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::System::Threading::INFINITE,
    Win32::UI::WindowsAndMessaging::*, core::*,
};

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);
static WINDOW_COUNT: AtomicUsize = AtomicUsize::new(0);
static REGISTER_CLASS: Once = Once::new();

// Thread-local event channel for dispatching Win32 messages as Events
// Uses mpsc to avoid RefCell panic when Win32 API calls trigger synchronous messages
thread_local! {
    static EVENT_SENDER: RefCell<Option<Sender<Event>>> = const { RefCell::new(None) };
    // Track windows that need redraw (marked via WM_PAINT or request_redraw)
    static DIRTY_WINDOWS: RefCell<HashSet<WindowId>> = RefCell::new(HashSet::new());
}

/// Helper to extract X coordinate from lparam (handles negative coords on multi-monitor setups)
#[inline]
fn get_x_lparam(lparam: LPARAM) -> f64 {
    // Cast to i16 first to preserve sign bit for negative coordinates
    (lparam.0 as i16) as f64
}

/// Helper to extract Y coordinate from lparam (handles negative coords on multi-monitor setups)
#[inline]
fn get_y_lparam(lparam: LPARAM) -> f64 {
    // Shift right to get high word, then cast to i16 to preserve sign
    ((lparam.0 >> 16) as i16) as f64
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
            // Register window class only once
            let class_name = w!("ArthropodWindow");

            REGISTER_CLASS.call_once(|| {
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
            });

            let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::SeqCst));

            let title: Vec<u16> = config
                .title
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            // Pass WindowId via lpParam so WM_NCCREATE can set it in GWLP_USERDATA
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
                Some(id.0 as *const _),
            )
            .map_err(|e| PlatformError::WindowCreation(format!("CreateWindowExW failed: {}", e)))?;

            if config.visible {
                let _ = ShowWindow(hwnd, SW_SHOW);
            }

            // Increment window count
            WINDOW_COUNT.fetch_add(1, Ordering::SeqCst);

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
            // InvalidateRect will trigger WM_PAINT, which marks window as dirty
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

impl Drop for WindowImpl {
    fn drop(&mut self) {
        // Decrement window count when window is dropped
        WINDOW_COUNT.fetch_sub(1, Ordering::SeqCst);
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
        WM_NCCREATE => {
            // Extract WindowId from lpCreateParams and store in GWLP_USERDATA
            // SAFETY: lparam is a pointer to CREATESTRUCTW during WM_NCCREATE
            let create_struct = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
            let window_id = create_struct.lpCreateParams as u64;
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_id as isize) };
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_SIZE => {
            // Retrieve WindowId from window user data
            let window_id = unsafe { WindowId(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as u64) };
            // Extract new window size from lparam
            let width = (lparam.0 & 0xFFFF) as u32;
            let height = ((lparam.0 >> 16) & 0xFFFF) as u32;

            // Queue the resize event
            EVENT_SENDER.with(|sender| {
                if let Some(sender) = sender.borrow().as_ref() {
                    let _ = sender.send(Event::Window {
                        window_id,
                        event: WindowEvent::Resized(Size { width, height }),
                    });
                }
            });

            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            // Retrieve WindowId from window user data
            let window_id = unsafe { WindowId(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as u64) };

            // Extract mouse coordinates using helpers (handles negative coords on multi-monitor)
            let x = get_x_lparam(lparam);
            let y = get_y_lparam(lparam);

            // Queue the event
            EVENT_SENDER.with(|sender| {
                if let Some(sender) = sender.borrow().as_ref() {
                    let _ = sender.send(Event::Window {
                        window_id,
                        event: WindowEvent::CursorMoved {
                            position: Point { x, y },
                        },
                    });
                }
            });

            LRESULT(0)
        }
        WM_PAINT => {
            // Mark window as needing redraw
            let window_id = unsafe { WindowId(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as u64) };
            DIRTY_WINDOWS.with(|dirty| {
                dirty.borrow_mut().insert(window_id);
            });

            // Validate the window to prevent Windows from re-sending WM_PAINT
            unsafe {
                let mut ps = PAINTSTRUCT::default();
                let _hdc = BeginPaint(hwnd, &mut ps);
                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        WM_CLOSE => unsafe {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        },
        WM_DESTROY => unsafe {
            // Only quit the application when the last window is destroyed
            if WINDOW_COUNT.load(Ordering::SeqCst) == 0 {
                PostQuitMessage(0);
            }
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

    // Create event channel to avoid RefCell panic during recursive Win32 calls
    let (event_sender, event_receiver) = mpsc::channel();

    // Set thread-local sender for wndproc to use
    EVENT_SENDER.with(|sender| {
        *sender.borrow_mut() = Some(event_sender);
    });

    unsafe {
        let mut msg = MSG::default();

        loop {
            if control_flow == ControlFlow::Exit {
                break;
            }

            // Wait for messages efficiently based on control flow
            let _wait_result = if control_flow == ControlFlow::Wait {
                // Wait indefinitely for messages (0% CPU when idle)
                MsgWaitForMultipleObjectsEx(
                    None,           // No handles to wait for
                    INFINITE,       // Wait forever
                    QS_ALLINPUT,    // Wake on any input
                    MWMO_INPUTAVAILABLE,
                )
            } else {
                // Poll mode: Non-blocking, let wgpu swapchain handle V-Sync
                // Don't use sleep timeout - it adds to frame time, not replaces it
                MsgWaitForMultipleObjectsEx(
                    None,
                    0,              // Non-blocking (wgpu present will V-Sync)
                    QS_ALLINPUT,
                    MWMO_INPUTAVAILABLE,
                )
            };

            // Drain ALL pending messages before rendering (no interleaving)
            // This batches input processing for better performance
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    return Ok(());
                }

                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }

            // Drain all events from channel (won't panic even if wndproc fires during app.on_event)
            while let Ok(event) = event_receiver.try_recv() {
                app.on_event(event, &mut control_flow);
            }

            // Only redraw windows that are actually dirty
            // This prevents unnecessary rendering when nothing has changed
            let dirty_windows: Vec<WindowId> = DIRTY_WINDOWS.with(|dirty| {
                dirty.borrow_mut().drain().collect()
            });

            for window_id in dirty_windows {
                app.on_redraw(window_id);
            }
        }
    }

    Ok(())
}
