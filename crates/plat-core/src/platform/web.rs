//! Web platform backend (wasm32 + web feature).

use crate::{
    Application, BackdropMaterial, ControlFlow, ElementState, Event, LifecycleEvent, PlatformError,
    Size, Window, WindowConfig, WindowEvent, WindowId,
};
use raw_window_handle::{DisplayHandle, HasDisplayHandle, HasWindowHandle, WindowHandle};
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, Window as WebWindow};

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);
static ACTIVE_WINDOW_ID: AtomicU64 = AtomicU64::new(0);

fn current_window_id() -> WindowId {
    WindowId(ACTIVE_WINDOW_ID.load(Ordering::SeqCst))
}

fn dispatch_window_event<A: Application>(
    app: &Rc<RefCell<A>>,
    control_flow: &Rc<RefCell<ControlFlow>>,
    pending_events: &Rc<RefCell<VecDeque<WindowEvent>>>,
    is_draining_events: &Rc<Cell<bool>>,
    event: WindowEvent,
) {
    {
        let mut pending = pending_events.borrow_mut();
        pending.push_back(event);
    }

    if !is_draining_events.get() {
        drain_window_events(app, control_flow, pending_events, is_draining_events);
    }
}

fn drain_window_events<A: Application>(
    app: &Rc<RefCell<A>>,
    control_flow: &Rc<RefCell<ControlFlow>>,
    pending_events: &Rc<RefCell<VecDeque<WindowEvent>>>,
    is_draining_events: &Rc<Cell<bool>>,
) {
    if is_draining_events.replace(true) {
        return;
    }

    loop {
        let event = {
            let mut pending = pending_events.borrow_mut();
            pending.pop_front()
        };

        let Some(event) = event else {
            break;
        };

        let mut app_ref = match app.try_borrow_mut() {
            Ok(app_ref) => app_ref,
            Err(_) => {
                {
                    let mut pending = pending_events.borrow_mut();
                    pending.push_front(event);
                }
                break;
            }
        };

        let mut cf = *control_flow.borrow();
        app_ref.on_event(
            Event::Window {
                window_id: current_window_id(),
                event,
            },
            &mut cf,
        );
        drop(app_ref);

        *control_flow.borrow_mut() = cf;
    }

    is_draining_events.set(false);
}

pub struct EventLoopImpl {
    window: WebWindow,
}

impl EventLoopImpl {
    pub fn new() -> Result<Self, PlatformError> {
        let Some(window) = web_sys::window() else {
            return Err(PlatformError::Initialization(
                "web_sys::window() was unavailable".into(),
            ));
        };
        Ok(Self { window })
    }

    pub fn create_window(&self, config: WindowConfig) -> Result<Window, PlatformError> {
        let inner = WindowImpl::new(&self.window, config)?;
        Ok(Window { inner })
    }
}

pub struct WindowImpl {
    window: WebWindow,
    canvas: HtmlCanvasElement,
    id: WindowId,
    backdrop_material: Cell<BackdropMaterial>,
}

impl WindowImpl {
    fn new(window: &WebWindow, config: WindowConfig) -> Result<Self, PlatformError> {
        let Some(document) = window.document() else {
            return Err(PlatformError::Initialization(
                "window.document() was unavailable".into(),
            ));
        };

        let canvas = if let Some(existing) = document.get_element_by_id("arthropod-canvas") {
            existing.dyn_into::<HtmlCanvasElement>().map_err(|_| {
                PlatformError::WindowCreation("Element #arthropod-canvas was not a canvas".into())
            })?
        } else {
            let created = document.create_element("canvas").map_err(|e| {
                PlatformError::WindowCreation(format!("Failed to create canvas: {e:?}"))
            })?;
            created
                .set_attribute("id", "arthropod-canvas")
                .map_err(|e| {
                    PlatformError::WindowCreation(format!("Failed to set canvas id: {e:?}"))
                })?;
            let canvas = created.dyn_into::<HtmlCanvasElement>().map_err(|_| {
                PlatformError::WindowCreation("Created element was not a canvas".into())
            })?;
            if let Some(body) = document.body() {
                body.append_child(&canvas).map_err(|e| {
                    PlatformError::WindowCreation(format!("Failed to append canvas: {e:?}"))
                })?;
            }
            canvas
        };

        canvas.set_width(config.size.width);
        canvas.set_height(config.size.height);
        document.set_title(&config.title);

        let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::SeqCst));
        ACTIVE_WINDOW_ID.store(id.0, Ordering::SeqCst);

        Ok(Self {
            window: window.clone(),
            canvas,
            id,
            backdrop_material: Cell::new(BackdropMaterial::None),
        })
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn inner_size(&self) -> Size<u32> {
        Size::new(self.canvas.width(), self.canvas.height())
    }

    pub fn set_title(&self, title: &str) {
        if let Some(document) = self.window.document() {
            document.set_title(title);
        }
    }

    pub fn request_redraw(&self) {}

    pub fn scale_factor(&self) -> f64 {
        self.window.device_pixel_ratio()
    }

    pub fn set_visible(&self, _visible: bool) {}

    pub fn set_backdrop_material(&self, material: BackdropMaterial) {
        self.backdrop_material.set(material);
    }

    pub fn backdrop_material(&self) -> BackdropMaterial {
        self.backdrop_material.get()
    }
}

impl HasWindowHandle for WindowImpl {
    fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
        let js_value: &wasm_bindgen::JsValue = self.canvas.as_ref();
        let obj = std::ptr::NonNull::from(js_value).cast();
        let raw = raw_window_handle::WebCanvasWindowHandle::new(obj);

        // SAFETY: The JsValue pointer is borrowed from `self.canvas` and remains valid
        // for the lifetime of the returned `WindowHandle`.
        Ok(unsafe { WindowHandle::borrow_raw(raw.into()) })
    }
}

impl HasDisplayHandle for WindowImpl {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(DisplayHandle::web())
    }
}

pub fn run<A: Application>() -> Result<(), PlatformError> {
    let event_loop = crate::EventLoop::new()?;
    let app = Rc::new(RefCell::new(A::new(&event_loop)));
    let control_flow = Rc::new(RefCell::new(ControlFlow::Poll));
    let scheduler = Rc::new(RefCell::new(super::web_runtime::RedrawScheduler::default()));
    let pending_events: Rc<RefCell<VecDeque<WindowEvent>>> = Rc::new(RefCell::new(VecDeque::new()));
    let draining_events = Rc::new(Cell::new(false));

    {
        let mut cf = *control_flow.borrow();
        app.borrow_mut()
            .on_event(Event::Lifecycle(LifecycleEvent::Resumed), &mut cf);
        *control_flow.borrow_mut() = cf;
    }
    if *control_flow.borrow() == ControlFlow::Exit {
        return Ok(());
    }

    scheduler.borrow_mut().request();

    let Some(window) = web_sys::window() else {
        return Err(PlatformError::Initialization(
            "web_sys::window() was unavailable".into(),
        ));
    };
    let Some(document) = window.document() else {
        return Err(PlatformError::Initialization(
            "window.document() was unavailable".into(),
        ));
    };
    let canvas = document
        .get_element_by_id("arthropod-canvas")
        .ok_or_else(|| {
            PlatformError::Initialization(
                "Missing #arthropod-canvas. Create a window before entering run()".into(),
            )
        })?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| {
            PlatformError::Initialization("Element #arthropod-canvas was not a canvas".into())
        })?;

    // Allow canvas to receive keyboard focus.
    let _ = canvas.set_attribute("tabindex", "0");

    {
        let dpr = window.device_pixel_ratio();
        let css_width = f64::from(canvas.client_width().max(1));
        let css_height = f64::from(canvas.client_height().max(1));
        let (resized, scale_changed) =
            super::web_runtime::map_resize_events(css_width, css_height, dpr);

        if let WindowEvent::Resized(size) = resized {
            canvas.set_width(size.width);
            canvas.set_height(size.height);
            dispatch_window_event(
                &app,
                &control_flow,
                &pending_events,
                &draining_events,
                WindowEvent::Resized(size),
            );
            dispatch_window_event(
                &app,
                &control_flow,
                &pending_events,
                &draining_events,
                WindowEvent::ScaleFactorChanged {
                    scale_factor: dpr,
                    new_inner_size: size,
                },
            );
        } else {
            dispatch_window_event(
                &app,
                &control_flow,
                &pending_events,
                &draining_events,
                resized,
            );
            dispatch_window_event(
                &app,
                &control_flow,
                &pending_events,
                &draining_events,
                scale_changed,
            );
        }
    }

    {
        let app_for_mouse_down = Rc::clone(&app);
        let control_for_mouse_down = Rc::clone(&control_flow);
        let pending_for_mouse_down = Rc::clone(&pending_events);
        let draining_for_mouse_down = Rc::clone(&draining_events);
        let scheduler_for_mouse_down = Rc::clone(&scheduler);
        let mouse_down =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                let modifiers = super::web_runtime::map_modifiers(
                    event.shift_key(),
                    event.ctrl_key(),
                    event.alt_key(),
                    event.meta_key(),
                );
                let mapped = super::web_runtime::map_pointer_down_with_modifiers(
                    event.button(),
                    f64::from(event.offset_x()),
                    f64::from(event.offset_y()),
                    modifiers,
                );
                dispatch_window_event(
                    &app_for_mouse_down,
                    &control_for_mouse_down,
                    &pending_for_mouse_down,
                    &draining_for_mouse_down,
                    mapped,
                );
                let _ = scheduler_for_mouse_down.borrow_mut().request();
            })
                as Box<dyn FnMut(web_sys::MouseEvent)>);
        canvas
            .add_event_listener_with_callback("mousedown", mouse_down.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::EventLoop(format!("Failed to add mousedown: {e:?}")))?;
        mouse_down.forget();
    }

    {
        let app_for_mouse_up = Rc::clone(&app);
        let control_for_mouse_up = Rc::clone(&control_flow);
        let pending_for_mouse_up = Rc::clone(&pending_events);
        let draining_for_mouse_up = Rc::clone(&draining_events);
        let scheduler_for_mouse_up = Rc::clone(&scheduler);
        let mouse_up =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                let modifiers = super::web_runtime::map_modifiers(
                    event.shift_key(),
                    event.ctrl_key(),
                    event.alt_key(),
                    event.meta_key(),
                );
                let mapped = super::web_runtime::map_pointer_up_with_modifiers(
                    event.button(),
                    f64::from(event.offset_x()),
                    f64::from(event.offset_y()),
                    modifiers,
                );
                dispatch_window_event(
                    &app_for_mouse_up,
                    &control_for_mouse_up,
                    &pending_for_mouse_up,
                    &draining_for_mouse_up,
                    mapped,
                );
                let _ = scheduler_for_mouse_up.borrow_mut().request();
            })
                as Box<dyn FnMut(web_sys::MouseEvent)>);
        canvas
            .add_event_listener_with_callback("mouseup", mouse_up.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::EventLoop(format!("Failed to add mouseup: {e:?}")))?;
        mouse_up.forget();
    }

    {
        let app_for_mouse_move = Rc::clone(&app);
        let control_for_mouse_move = Rc::clone(&control_flow);
        let pending_for_mouse_move = Rc::clone(&pending_events);
        let draining_for_mouse_move = Rc::clone(&draining_events);
        let scheduler_for_mouse_move = Rc::clone(&scheduler);
        let mouse_move =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                let mapped = super::web_runtime::map_pointer_move(
                    f64::from(event.offset_x()),
                    f64::from(event.offset_y()),
                );
                dispatch_window_event(
                    &app_for_mouse_move,
                    &control_for_mouse_move,
                    &pending_for_mouse_move,
                    &draining_for_mouse_move,
                    mapped,
                );
                let _ = scheduler_for_mouse_move.borrow_mut().request();
            })
                as Box<dyn FnMut(web_sys::MouseEvent)>);
        canvas
            .add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::EventLoop(format!("Failed to add mousemove: {e:?}")))?;
        mouse_move.forget();
    }

    {
        let app_for_enter = Rc::clone(&app);
        let control_for_enter = Rc::clone(&control_flow);
        let pending_for_enter = Rc::clone(&pending_events);
        let draining_for_enter = Rc::clone(&draining_events);
        let enter = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
            dispatch_window_event(
                &app_for_enter,
                &control_for_enter,
                &pending_for_enter,
                &draining_for_enter,
                WindowEvent::CursorEntered(true),
            );
        })
            as Box<dyn FnMut(web_sys::Event)>);
        canvas
            .add_event_listener_with_callback("mouseenter", enter.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::EventLoop(format!("Failed to add mouseenter: {e:?}")))?;
        enter.forget();
    }

    {
        let app_for_leave = Rc::clone(&app);
        let control_for_leave = Rc::clone(&control_flow);
        let pending_for_leave = Rc::clone(&pending_events);
        let draining_for_leave = Rc::clone(&draining_events);
        let leave = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
            dispatch_window_event(
                &app_for_leave,
                &control_for_leave,
                &pending_for_leave,
                &draining_for_leave,
                WindowEvent::CursorEntered(false),
            );
        })
            as Box<dyn FnMut(web_sys::Event)>);
        canvas
            .add_event_listener_with_callback("mouseleave", leave.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::EventLoop(format!("Failed to add mouseleave: {e:?}")))?;
        leave.forget();
    }

    {
        let app_for_wheel = Rc::clone(&app);
        let control_for_wheel = Rc::clone(&control_flow);
        let pending_for_wheel = Rc::clone(&pending_events);
        let draining_for_wheel = Rc::clone(&draining_events);
        let scheduler_for_wheel = Rc::clone(&scheduler);
        let wheel =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
                event.prevent_default();
                let mapped = super::web_runtime::wheel_event_from_input(
                    event.delta_x(),
                    event.delta_y(),
                    event.delta_mode(),
                );
                dispatch_window_event(
                    &app_for_wheel,
                    &control_for_wheel,
                    &pending_for_wheel,
                    &draining_for_wheel,
                    mapped,
                );
                let _ = scheduler_for_wheel.borrow_mut().request();
            })
                as Box<dyn FnMut(web_sys::WheelEvent)>);
        canvas
            .add_event_listener_with_callback("wheel", wheel.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::EventLoop(format!("Failed to add wheel: {e:?}")))?;
        wheel.forget();
    }

    {
        let app_for_keydown = Rc::clone(&app);
        let control_for_keydown = Rc::clone(&control_flow);
        let pending_for_keydown = Rc::clone(&pending_events);
        let draining_for_keydown = Rc::clone(&draining_events);
        let scheduler_for_keydown = Rc::clone(&scheduler);
        let keydown =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                let input = super::web_runtime::map_key_input_with_modifiers(
                    &event.code(),
                    &event.key(),
                    ElementState::Pressed,
                    event.repeat(),
                    super::web_runtime::map_modifiers(
                        event.shift_key(),
                        event.ctrl_key(),
                        event.alt_key(),
                        event.meta_key(),
                    ),
                );
                dispatch_window_event(
                    &app_for_keydown,
                    &control_for_keydown,
                    &pending_for_keydown,
                    &draining_for_keydown,
                    WindowEvent::KeyboardInput(input),
                );
                let _ = scheduler_for_keydown.borrow_mut().request();
            })
                as Box<dyn FnMut(web_sys::KeyboardEvent)>);
        window
            .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::EventLoop(format!("Failed to add keydown: {e:?}")))?;
        keydown.forget();
    }

    {
        let app_for_keyup = Rc::clone(&app);
        let control_for_keyup = Rc::clone(&control_flow);
        let pending_for_keyup = Rc::clone(&pending_events);
        let draining_for_keyup = Rc::clone(&draining_events);
        let scheduler_for_keyup = Rc::clone(&scheduler);
        let keyup =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                let input = super::web_runtime::map_key_input_with_modifiers(
                    &event.code(),
                    &event.key(),
                    ElementState::Released,
                    event.repeat(),
                    super::web_runtime::map_modifiers(
                        event.shift_key(),
                        event.ctrl_key(),
                        event.alt_key(),
                        event.meta_key(),
                    ),
                );
                dispatch_window_event(
                    &app_for_keyup,
                    &control_for_keyup,
                    &pending_for_keyup,
                    &draining_for_keyup,
                    WindowEvent::KeyboardInput(input),
                );
                let _ = scheduler_for_keyup.borrow_mut().request();
            })
                as Box<dyn FnMut(web_sys::KeyboardEvent)>);
        window
            .add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::EventLoop(format!("Failed to add keyup: {e:?}")))?;
        keyup.forget();
    }

    {
        let app_for_resize = Rc::clone(&app);
        let control_for_resize = Rc::clone(&control_flow);
        let pending_for_resize = Rc::clone(&pending_events);
        let draining_for_resize = Rc::clone(&draining_events);
        let scheduler_for_resize = Rc::clone(&scheduler);
        let canvas_for_resize = canvas.clone();
        let window_for_resize = window.clone();
        let resize = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let dpr = window_for_resize.device_pixel_ratio();
            let css_width = f64::from(canvas_for_resize.client_width().max(1));
            let css_height = f64::from(canvas_for_resize.client_height().max(1));
            let (resized, scale_changed) =
                super::web_runtime::map_resize_events(css_width, css_height, dpr);
            if let WindowEvent::Resized(size) = resized {
                canvas_for_resize.set_width(size.width);
                canvas_for_resize.set_height(size.height);
                dispatch_window_event(
                    &app_for_resize,
                    &control_for_resize,
                    &pending_for_resize,
                    &draining_for_resize,
                    WindowEvent::Resized(size),
                );
                dispatch_window_event(
                    &app_for_resize,
                    &control_for_resize,
                    &pending_for_resize,
                    &draining_for_resize,
                    WindowEvent::ScaleFactorChanged {
                        scale_factor: dpr,
                        new_inner_size: size,
                    },
                );
            } else {
                dispatch_window_event(
                    &app_for_resize,
                    &control_for_resize,
                    &pending_for_resize,
                    &draining_for_resize,
                    resized,
                );
                dispatch_window_event(
                    &app_for_resize,
                    &control_for_resize,
                    &pending_for_resize,
                    &draining_for_resize,
                    scale_changed,
                );
            }
            let _ = scheduler_for_resize.borrow_mut().request();
        })
            as Box<dyn FnMut(web_sys::Event)>);
        window
            .add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::EventLoop(format!("Failed to add resize: {e:?}")))?;
        resize.forget();
    }

    let window_for_loop = window.clone();

    let raf_cb: Rc<RefCell<Option<wasm_bindgen::closure::Closure<dyn FnMut()>>>> =
        Rc::new(RefCell::new(None));
    let raf_cb_for_loop = Rc::clone(&raf_cb);
    let app_for_loop = Rc::clone(&app);
    let control_for_loop = Rc::clone(&control_flow);
    let scheduler_for_loop = Rc::clone(&scheduler);
    let pending_for_loop = Rc::clone(&pending_events);
    let draining_for_loop = Rc::clone(&draining_events);

    *raf_cb.borrow_mut() = Some(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        if *control_for_loop.borrow() == ControlFlow::Exit {
            return;
        }

        if scheduler_for_loop.borrow_mut().consume() {
            drain_window_events(
                &app_for_loop,
                &control_for_loop,
                &pending_for_loop,
                &draining_for_loop,
            );

            let mut cf = *control_for_loop.borrow();
            let window_id = current_window_id();
            app_for_loop.borrow_mut().on_event(
                Event::Window {
                    window_id,
                    event: WindowEvent::RedrawRequested,
                },
                &mut cf,
            );
            if cf != ControlFlow::Exit {
                app_for_loop.borrow_mut().on_redraw(window_id);
            }
            *control_for_loop.borrow_mut() = cf;

            // Process any events queued re-entrantly during on_event/on_redraw.
            drain_window_events(
                &app_for_loop,
                &control_for_loop,
                &pending_for_loop,
                &draining_for_loop,
            );
        }

        if *control_for_loop.borrow() != ControlFlow::Exit {
            let _ = scheduler_for_loop.borrow_mut().request();
            if let Some(cb) = raf_cb_for_loop.borrow().as_ref() {
                let _ = window_for_loop.request_animation_frame(cb.as_ref().unchecked_ref());
            }
        }
    })
        as Box<dyn FnMut()>));

    if let Some(cb) = raf_cb.borrow().as_ref() {
        window
            .request_animation_frame(cb.as_ref().unchecked_ref())
            .map_err(|e| {
                PlatformError::EventLoop(format!("requestAnimationFrame failed: {e:?}"))
            })?;
    }

    // Keep RAF callback alive for the lifetime of the page.
    std::mem::forget(raf_cb);

    Ok(())
}
