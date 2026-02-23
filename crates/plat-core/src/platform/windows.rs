//! Windows platform implementation using Win32 APIs.

pub mod composition;

use crate::{
    Application, ControlFlow, Event, PlatformError, Point, Size, Window, WindowConfig, WindowEvent,
    WindowId,
    materials::{BackdropMaterial, HasBackdropMaterial},
};
use raw_window_handle::{
    DisplayHandle, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
    Win32WindowHandle, WindowHandle, WindowsDisplayHandle,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Once;
use std::sync::atomic::{AtomicU8, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::System::Threading::INFINITE, Win32::UI::Accessibility::*,
    Win32::UI::WindowsAndMessaging::*, core::*,
};

#[cfg(target_os = "windows")]
thread_local! {
    static A11Y_PROVIDERS: RefCell<HashMap<isize, IRawElementProviderSimple>> = RefCell::new(HashMap::new());
}

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

/// Helper to safely retrieve WindowId from HWND user data, handling 32-bit sign extension correctly.
#[inline]
unsafe fn get_window_id(hwnd: HWND) -> WindowId {
    // Cast to usize first to ensure zero-extension on 32-bit systems where isize is negative
    // but the original ID was a large u32.
    WindowId(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as usize as u64)
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
    thread_id: thread::ThreadId,
    /// Current backdrop material (stored as u8: 0=None, 1=Mica, 2=MicaAlt, 3=Acrylic)
    backdrop_material: AtomicU8,
    /// DirectComposition integration (only if composition_mode enabled)
    #[cfg(target_os = "windows")]
    composition: Option<std::sync::Arc<WindowComposition>>,
}

/// DirectComposition state for a window.
#[cfg(target_os = "windows")]
pub struct WindowComposition {
    pub device: composition::CompositionDevice,
    #[allow(dead_code)] // Future API surface
    pub target: composition::CompositionTarget,
    pub root_visual: composition::CompositionVisual,
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
                    hbrBackground: HBRUSH(0 as _), // Transparent background for composition
                    ..Default::default()
                };

                let _ = RegisterClassW(&wc);
            });

            let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::SeqCst));

            // Ensure ID fits in pointer for 32-bit systems where usize < u64
            // This prevents ID truncation when passing via lpParam
            if std::mem::size_of::<usize>() < 8 && id.0 > usize::MAX as u64 {
                return Err(PlatformError::Initialization(
                    "Window ID overflow on 32-bit system".into(),
                ));
            }

            let title: Vec<u16> = config
                .title
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            // Note: WS_EX_NOREDIRECTIONBITMAP is required for DirectComposition to work correctly
            // with transparent swapchains. It tells the DWM not to allocate a redirection bitmap,
            // relying entirely on the application's swapchain for content.
            let mut ex_style = WINDOW_EX_STYLE::default();
            if config.composition_mode {
                ex_style |= WS_EX_NOREDIRECTIONBITMAP;
            }

            // Pass WindowId via lpParam so WM_NCCREATE can set it in GWLP_USERDATA
            let hwnd: HWND = CreateWindowExW(
                ex_style,
                class_name,
                PCWSTR(title.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                config.size.width as i32,
                config.size.height as i32,
                None, // Parent
                None, // Menu
                Some(hinstance),
                Some(id.0 as *const std::ffi::c_void),
            )
            .map_err(|e| PlatformError::WindowCreation(format!("CreateWindowExW failed: {}", e)))?;

            if config.visible {
                let _ = ShowWindow(hwnd, SW_SHOW);
            }

            // Extend frame into client area to ensure transparency works
            if config.composition_mode {
                use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
                let margins = windows::Win32::UI::Controls::MARGINS {
                    cxLeftWidth: -1, // -1 extends to entire client area
                    cxRightWidth: -1,
                    cyTopHeight: -1,
                    cyBottomHeight: -1,
                };
                let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
            }

            // Increment window count
            WINDOW_COUNT.fetch_add(1, Ordering::SeqCst);

            // Initialize DirectComposition if requested
            let composition = if config.composition_mode {
                // Initialize COM for DirectComposition
                let _ = windows::Win32::System::Com::CoInitializeEx(
                    None,
                    windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
                );

                let device = composition::CompositionDevice::new().map_err(|e| {
                    PlatformError::Initialization(format!("DirectComposition device: {}", e))
                })?;
                // Topmost=false ensures composition content is BEHIND wgpu swapchain
                let target = device.create_target_for_hwnd(hwnd, false).map_err(|e| {
                    PlatformError::Initialization(format!("Composition target: {}", e))
                })?;
                let root_visual = device
                    .create_visual()
                    .map_err(|e| PlatformError::Initialization(format!("Root visual: {}", e)))?;

                target
                    .set_root(&root_visual)
                    .map_err(|e| PlatformError::Initialization(format!("Set root: {}", e)))?;

                // Windows COM DirectComposition wrappers are inherently single-threaded, Arc is for ref counting
                #[allow(clippy::arc_with_non_send_sync)]
                Some(std::sync::Arc::new(WindowComposition {
                    device,
                    target,
                    root_visual,
                }))
            } else {
                None
            };

            Ok(Self {
                hwnd,
                hinstance,
                id,
                thread_id: thread::current().id(),
                backdrop_material: AtomicU8::new(0), // BackdropMaterial::None
                composition,
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

    #[cfg(target_os = "windows")]
    pub fn composition(&self) -> Option<std::sync::Arc<WindowComposition>> {
        self.composition.clone()
    }

    #[cfg(target_os = "windows")]
    #[allow(dead_code)] // Future API for accessibility integration
    pub fn register_a11y_provider(&self, provider: IRawElementProviderSimple) {
        // Ensure we are on the correct thread, as A11Y_PROVIDERS is thread-local
        // and wndproc expects to find the provider in its own thread-local map.
        if thread::current().id() != self.thread_id {
            panic!(
                "register_a11y_provider must be called from the same thread that created the window"
            );
        }

        // Register the provider for this window's HWND
        A11Y_PROVIDERS.with(|providers| {
            // Cast *mut c_void to isize
            providers
                .borrow_mut()
                .insert(self.hwnd.0 as isize, provider);
        });
    }
}

impl Drop for WindowImpl {
    fn drop(&mut self) {
        // If we initialized COM for DirectComposition, uninitialize it
        // Must be done before destroying the window to ensure clean teardown
        #[cfg(target_os = "windows")]
        if self.composition.is_some() {
            // Drop composition resources first (while COM is still active)
            self.composition = None;
            // Now uninitialize COM
            unsafe {
                windows::Win32::System::Com::CoUninitialize();
            }
        }

        // Decrement window count BEFORE destroying the window
        // This ensures that when WM_DESTROY fires (synchronously inside DestroyWindow),
        // the count reflects the remaining windows (excluding this one).
        WINDOW_COUNT.fetch_sub(1, Ordering::SeqCst);

        unsafe {
            // Ensure the window is destroyed when the Rust struct is dropped
            // This handles cases where the app drops the window manually
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

impl HasBackdropMaterial for WindowImpl {
    fn set_backdrop_material(&self, material: BackdropMaterial) {
        // Use window-vibrancy crate for full-window backdrop effects
        // This applies the effect to the entire window via DWM, allowing
        // semi-transparent wgpu content to show the backdrop through.
        //
        // Note: This is a window-level effect. For selective transparency
        // (Mica in some areas, solid in others), we'll need DirectComposition
        // integration in the future.
        let result = match material {
            BackdropMaterial::None => {
                // Clear all effects
                window_vibrancy::clear_mica(self).and_then(|_| window_vibrancy::clear_acrylic(self))
            }
            BackdropMaterial::Mica | BackdropMaterial::MicaAlt => {
                // Apply Mica effect (Windows 11)
                // None means use default system theme colors
                window_vibrancy::apply_mica(self, None)
            }
            BackdropMaterial::Acrylic => {
                // Apply Acrylic effect with subtle dark tint
                // RGBA: (18, 18, 18, 200) = dark gray with 78% opacity
                window_vibrancy::apply_acrylic(self, Some((18, 18, 18, 200)))
            }
        };

        // Log errors but don't fail - graceful degradation on older Windows
        if let Err(e) = result {
            log::warn!("Failed to apply backdrop material {:?}: {}", material, e);
        }

        // Store the material value (0=None, 1=Mica, 2=MicaAlt, 3=Acrylic)
        let material_value = match material {
            BackdropMaterial::None => 0u8,
            BackdropMaterial::Mica => 1u8,
            BackdropMaterial::MicaAlt => 2u8,
            BackdropMaterial::Acrylic => 3u8,
        };
        self.backdrop_material
            .store(material_value, Ordering::SeqCst);
    }

    fn backdrop_material(&self) -> BackdropMaterial {
        match self.backdrop_material.load(Ordering::SeqCst) {
            1 => BackdropMaterial::Mica,
            2 => BackdropMaterial::MicaAlt,
            3 => BackdropMaterial::Acrylic,
            _ => BackdropMaterial::None,
        }
    }
}

impl HasWindowHandle for WindowImpl {
    fn window_handle(
        &self,
    ) -> std::result::Result<WindowHandle<'_>, raw_window_handle::HandleError> {
        use std::num::NonZeroIsize;
        let handle =
            Win32WindowHandle::new(NonZeroIsize::new(self.hwnd.0 as usize as isize).unwrap());
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
            let window_id = unsafe { get_window_id(hwnd) };
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
            let window_id = unsafe { get_window_id(hwnd) };

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
        WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN
        | WM_MBUTTONUP => {
            use crate::{ElementState, MouseButton, MouseInput};

            let window_id = unsafe { get_window_id(hwnd) };
            let x = get_x_lparam(lparam);
            let y = get_y_lparam(lparam);

            let (button, state) = match msg {
                WM_LBUTTONDOWN => (MouseButton::Left, ElementState::Pressed),
                WM_LBUTTONUP => (MouseButton::Left, ElementState::Released),
                WM_RBUTTONDOWN => (MouseButton::Right, ElementState::Pressed),
                WM_RBUTTONUP => (MouseButton::Right, ElementState::Released),
                WM_MBUTTONDOWN => (MouseButton::Middle, ElementState::Pressed),
                WM_MBUTTONUP => (MouseButton::Middle, ElementState::Released),
                _ => unreachable!(),
            };

            // TODO: Extract modifiers from wparam
            let modifiers = crate::Modifiers::default();

            EVENT_SENDER.with(|sender| {
                if let Some(sender) = sender.borrow().as_ref() {
                    let _ = sender.send(Event::Window {
                        window_id,
                        event: WindowEvent::MouseInput(MouseInput {
                            button,
                            state,
                            position: Point { x, y },
                            modifiers,
                        }),
                    });
                }
            });

            LRESULT(0)
        }
        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {
            use crate::{ElementState, Key, KeyboardInput};

            let window_id = unsafe { get_window_id(hwnd) };
            let vk = wparam.0 as u32;
            let state = if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                ElementState::Pressed
            } else {
                ElementState::Released
            };

            // Map virtual key codes to Key enum
            let key = match vk {
                0x41..=0x5A => {
                    // A-Z
                    Key::from_vk(vk)
                }
                0x30..=0x39 => {
                    // 0-9
                    Key::from_vk(vk)
                }
                0x08 => Key::Backspace,
                0x09 => Key::Tab,
                0x0D => Key::Enter,
                0x10 => Key::Shift,
                0x11 => Key::Control,
                0x12 => Key::Alt,
                0x1B => Key::Escape,
                0x20 => Key::Space,
                0x21 => Key::PageUp,
                0x22 => Key::PageDown,
                0x23 => Key::End,
                0x24 => Key::Home,
                0x25 => Key::Left,
                0x26 => Key::Up,
                0x27 => Key::Right,
                0x28 => Key::Down,
                0x2C => Key::PrintScreen,
                0x2D => Key::Insert,
                0x2E => Key::Delete,
                0x70..=0x87 => Key::from_vk(vk), // F1-F24
                _ => Key::Unknown,
            };

            // TODO: Extract modifiers from GetKeyState
            let modifiers = crate::Modifiers::default();
            let repeat = (lparam.0 & 0x40000000) != 0;

            EVENT_SENDER.with(|sender| {
                if let Some(sender) = sender.borrow().as_ref() {
                    let _ = sender.send(Event::Window {
                        window_id,
                        event: WindowEvent::KeyboardInput(KeyboardInput {
                            key,
                            state,
                            modifiers,
                            repeat,
                        }),
                    });
                }
            });

            LRESULT(0)
        }
        WM_PAINT => {
            // Mark window as needing redraw
            let window_id = unsafe { get_window_id(hwnd) };
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
        WM_CLOSE => {
            let window_id = unsafe { get_window_id(hwnd) };
            EVENT_SENDER.with(|sender| {
                if let Some(sender) = sender.borrow().as_ref() {
                    let _ = sender.send(Event::Window {
                        window_id,
                        event: WindowEvent::CloseRequested,
                    });
                }
            });
            // We do NOT call DestroyWindow here. We let the application decide.
            // If the app wants to close, it should drop the Window or return ControlFlow::Exit.
            LRESULT(0)
        }
        WM_DESTROY => unsafe {
            // Only quit the application when the last window is destroyed
            if WINDOW_COUNT.load(Ordering::SeqCst) == 0 {
                PostQuitMessage(0);
            }
            LRESULT(0)
        },
        WM_ERASEBKGND => {
            // Return 1 (TRUE) to tell Windows we handled background erasure (by doing nothing)
            // This prevents GDI from clearing the window to the class background color
            LRESULT(1)
        }
        WM_GETOBJECT => {
            // Handle UIA provider request
            if lparam.0 == UiaRootObjectId as isize {
                let mut result = LRESULT(0);
                A11Y_PROVIDERS.with(|providers| {
                    if let Some(provider) = providers.borrow().get(&(hwnd.0 as isize)) {
                        unsafe {
                            result = UiaReturnRawElementProvider(hwnd, wparam, lparam, provider);
                        }
                    }
                });
                result
            } else {
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
        }
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
                    None,        // No handles to wait for
                    INFINITE,    // Wait forever
                    QS_ALLINPUT, // Wake on any input
                    MWMO_INPUTAVAILABLE,
                )
            } else {
                // Poll mode: Non-blocking, let wgpu swapchain handle V-Sync
                // Don't use sleep timeout - it adds to frame time, not replaces it
                MsgWaitForMultipleObjectsEx(
                    None,
                    0, // Non-blocking (wgpu present will V-Sync)
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
            let dirty_windows: Vec<WindowId> =
                DIRTY_WINDOWS.with(|dirty| dirty.borrow_mut().drain().collect());

            for window_id in dirty_windows {
                app.on_redraw(window_id);
            }
        }
    }

    Ok(())
}
