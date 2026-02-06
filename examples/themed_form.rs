//! Example: Themed Form
//!
//! Demonstrates layered rendering to create visual depth without shader changes.
//! Shows a sign-in form with faked shadows, borders, accent elements, and Mica backdrop.
//!
//! **Interactive features:**
//! - Click input fields to focus (highlighted border)
//! - Type text into focused fields (appears live)
//! - Tab / Shift+Tab to cycle between fields
//! - Click "Sign In" or press Enter to submit
//! - Green toast notification auto-dismisses after 2.5 seconds
//!
//! Run with: cargo run --example themed_form
//!
//! Techniques demonstrated:
//! - Layered RoundedRects for drop-shadow, border, and accent effects
//! - Custom vibrant color palette (independent of system accent)
//! - Mica backdrop material (Windows 11)
//! - SDF-based corner rounding with pill-shaped button
//! - Text rendering via cosmic-text pipeline
//! - Hit testing for click-to-focus
//! - Keyboard input handling

use std::time::{Duration, Instant};

use plat_core::{
    Application, BackdropMaterial, ControlFlow, ElementState, Event, EventLoop,
    HasBackdropMaterial, Key, MouseButton, Rect, Size, Window, WindowConfig, WindowEvent, WindowId,
};
use render_engine::{
    Color, NodeContent, NodeId, Scene, SceneNode,
    backend::{RenderBackend, WgpuBackend},
};
use theme_engine::{DesignTokens, SystemTheme};

/// IDs returned from scene construction for interactive elements
struct FormLayout {
    scene: Scene,
    field_text_ids: [NodeId; 3],
    field_border_ids: [NodeId; 3],
    input_fill_ids: [NodeId; 3],
    button_id: NodeId,
    toast_bg_id: NodeId,
    toast_text_id: NodeId,
}

/// Application state for the themed form demo
struct ThemedFormApp {
    // Backend must be dropped before window
    backend: WgpuBackend,
    window: Window,
    scene: Scene,
    #[allow(dead_code)]
    tokens: DesignTokens,
    // Interactive state
    focused_field: Option<usize>,
    field_texts: [String; 3],
    field_text_node_ids: [NodeId; 3],
    field_border_ids: [NodeId; 3],
    input_fill_ids: [NodeId; 3],
    button_id: NodeId,
    // Toast node IDs (created at scene build time, toggled via visible)
    toast_bg_id: NodeId,
    toast_text_id: NodeId,
    toast_created_at: Option<Instant>,
}

/// Colors used for focus highlight transitions
const FOCUS_BORDER: Color = Color::rgba(124.0 / 255.0, 58.0 / 255.0, 237.0 / 255.0, 0.5);
const UNFOCUS_BORDER: Color = Color::rgba(70.0 / 255.0, 70.0 / 255.0, 100.0 / 255.0, 0.6);

impl ThemedFormApp {
    fn set_focus(&mut self, field: Option<usize>) {
        // Restore previous focus border
        if let Some(prev) = self.focused_field
            && let Some(node) = self.scene.get_node_mut(self.field_border_ids[prev])
            && let NodeContent::RoundedRect { ref mut color, .. } = node.content
        {
            *color = UNFOCUS_BORDER;
        }

        self.focused_field = field;

        // Highlight new focus border
        if let Some(idx) = field
            && let Some(node) = self.scene.get_node_mut(self.field_border_ids[idx])
            && let NodeContent::RoundedRect { ref mut color, .. } = node.content
        {
            *color = FOCUS_BORDER;
        }
    }

    fn update_field_text(&mut self, idx: usize) {
        let display = if idx == 2 {
            // Password field: show dots
            "\u{2022}".repeat(self.field_texts[idx].len())
        } else {
            self.field_texts[idx].clone()
        };

        if let Some(node) = self.scene.get_node_mut(self.field_text_node_ids[idx])
            && let NodeContent::Text { ref mut text, .. } = node.content
        {
            *text = display;
        }
    }

    fn show_toast(&mut self) {
        if let Some(node) = self.scene.get_node_mut(self.toast_bg_id) {
            node.visible = true;
        }
        if let Some(node) = self.scene.get_node_mut(self.toast_text_id) {
            node.visible = true;
        }
        self.toast_created_at = Some(Instant::now());
    }

    fn hide_toast(&mut self) {
        if let Some(node) = self.scene.get_node_mut(self.toast_bg_id) {
            node.visible = false;
        }
        if let Some(node) = self.scene.get_node_mut(self.toast_text_id) {
            node.visible = false;
        }
        self.toast_created_at = None;
    }
}

impl Application for ThemedFormApp {
    fn new(event_loop: &EventLoop) -> Self {
        // Query system theme for design tokens
        let theme = SystemTheme::query().unwrap_or_else(|_| SystemTheme {
            accent_color: glam::Vec4::new(0.0, 0.47, 0.84, 1.0),
            is_dark_mode: false,
            supports_transparency: false,
            available_materials: vec![],
            text_color: glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
            text_secondary_color: glam::Vec4::new(0.4, 0.4, 0.4, 1.0),
        });
        let tokens = DesignTokens::from_system(&theme);

        println!("=== Themed Form Demo ===");
        println!("System theme:");
        println!("  - Dark mode: {}", theme.is_dark_mode);
        println!("  - Supports transparency: {}", theme.supports_transparency);
        println!(
            "  - Accent color: ({:.2}, {:.2}, {:.2})",
            theme.accent_color.x, theme.accent_color.y, theme.accent_color.z
        );
        println!(
            "  - Text color: ({:.2}, {:.2}, {:.2})",
            theme.text_color.x, theme.text_color.y, theme.text_color.z
        );
        println!("  - Available materials: {:?}", theme.available_materials);
        println!();

        // Create window with transparent config for backdrop effect
        let config = WindowConfig {
            title: "Themed Form Demo - Mica Backdrop".to_string(),
            size: Size::new(500, 620),
            resizable: true,
            decorations: true,
            transparent: true,
            visible: true,
            ..Default::default()
        };

        let window = event_loop
            .create_window(config)
            .expect("Failed to create window");

        // Apply Mica backdrop material (Windows 11)
        window.set_backdrop_material(BackdropMaterial::Mica);
        println!(
            "Applied backdrop material: {:?}",
            window.backdrop_material()
        );

        // Create GPU backend
        let size = window.inner_size();
        // SAFETY: Safe because backend is dropped before window (struct field order)
        let mut backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .expect("Failed to create backend");

        // Transparent clear color to show Mica backdrop through
        backend.set_clear_color(Color::rgba(0.0, 0.0, 0.0, 0.0));

        // Create scene with themed form elements
        let layout = create_themed_form_scene(&tokens);

        println!();
        println!("You should see:");
        println!("  - Desktop wallpaper bleeding through (Windows 11 Mica)");
        println!("  - Drop shadow behind the form card");
        println!("  - Vibrant purple accent strip across top of card");
        println!("  - Input fields with visible borders (layered rects)");
        println!("  - Pill-shaped vibrant purple 'Sign In' button");
        println!("  - Footer text below button");
        println!();
        println!("Interactive features:");
        println!("  - Click an input field to focus it");
        println!("  - Type to enter text");
        println!("  - Tab / Shift+Tab to cycle fields");
        println!("  - Click 'Sign In' or press Enter to submit");

        Self {
            backend,
            window,
            scene: layout.scene,
            tokens,
            focused_field: None,
            field_texts: [String::new(), String::new(), String::new()],
            field_text_node_ids: layout.field_text_ids,
            field_border_ids: layout.field_border_ids,
            input_fill_ids: layout.input_fill_ids,
            button_id: layout.button_id,
            toast_bg_id: layout.toast_bg_id,
            toast_text_id: layout.toast_text_id,
            toast_created_at: None,
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window {
                event: WindowEvent::Resized(size),
                ..
            } => {
                self.backend.resize(size.width, size.height);
                self.window.request_redraw();
            }
            Event::Window {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::Window {
                event: WindowEvent::KeyboardInput(input),
                ..
            } => {
                if input.state == ElementState::Pressed {
                    if let Some(idx) = self.focused_field {
                        match input.key {
                            Key::Backspace => {
                                self.field_texts[idx].pop();
                                self.update_field_text(idx);
                                self.window.request_redraw();
                            }
                            Key::Tab => {
                                let next = if input.modifiers.shift {
                                    if idx == 0 { 2 } else { idx - 1 }
                                } else {
                                    (idx + 1) % 3
                                };
                                self.set_focus(Some(next));
                                self.window.request_redraw();
                            }
                            Key::Enter => {
                                self.show_toast();
                                self.window.request_redraw();
                            }
                            Key::Escape => {
                                self.set_focus(None);
                                self.window.request_redraw();
                            }
                            key => {
                                if let Some(ch) = key.to_char(input.modifiers.shift) {
                                    self.field_texts[idx].push(ch);
                                    self.update_field_text(idx);
                                    self.window.request_redraw();
                                }
                            }
                        }
                    } else if input.key == Key::Tab {
                        self.set_focus(Some(0));
                        self.window.request_redraw();
                    }
                }
            }
            Event::Window {
                event: WindowEvent::MouseInput(mouse),
                ..
            } => {
                if mouse.button == MouseButton::Left && mouse.state == ElementState::Pressed {
                    let x = mouse.position.x as f32;
                    let y = mouse.position.y as f32;

                    // Check button hit first
                    if let Some(btn_node) = self.scene.get_node(self.button_id)
                        && btn_node.bounds.contains(x, y)
                    {
                        self.show_toast();
                        self.window.request_redraw();
                        return;
                    }

                    // Check input fields
                    let mut clicked_field = None;
                    for (i, &fill_id) in self.input_fill_ids.iter().enumerate() {
                        if let Some(fill_node) = self.scene.get_node(fill_id)
                            && fill_node.bounds.contains(x, y)
                        {
                            clicked_field = Some(i);
                            break;
                        }
                    }

                    self.set_focus(clicked_field);
                    self.window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        // Check toast timeout
        if let Some(created_at) = self.toast_created_at {
            if created_at.elapsed() >= Duration::from_millis(2500) {
                self.hide_toast();
            } else {
                // Keep redraws flowing so the timer check fires
                self.window.request_redraw();
            }
        }

        if let Err(e) = self.backend.render(&self.scene) {
            eprintln!("Render error: {}", e);
        }
    }
}

/// Helper: add a rounded rect node and set its bounds
#[allow(clippy::too_many_arguments)]
fn add_rounded_rect(
    scene: &mut Scene,
    parent: NodeId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: Color,
    corner_radius: f32,
) -> NodeId {
    let node = SceneNode::new(NodeContent::RoundedRect {
        color,
        corner_radius,
    });
    let id = scene.add_node(parent, node);
    if let Some(n) = scene.get_node_mut(id) {
        n.bounds = Rect::new(x, y, w, h);
    }
    id
}

/// Helper: add a text node and set its bounds
#[allow(clippy::too_many_arguments)]
fn add_text(
    scene: &mut Scene,
    parent: NodeId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &str,
    font_size: f32,
    color: Color,
) -> NodeId {
    let node = SceneNode::new(NodeContent::Text {
        text: text.to_string(),
        font_size,
        color,
    });
    let id = scene.add_node(parent, node);
    if let Some(n) = scene.get_node_mut(id) {
        n.bounds = Rect::new(x, y, w, h);
    }
    id
}

/// Helper: add a bordered input field (border rect + fill rect + text node).
/// Returns (border_id, fill_id, text_id).
#[allow(clippy::too_many_arguments)]
fn add_input_field(
    scene: &mut Scene,
    parent: NodeId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    border_color: Color,
    fill_color: Color,
    text_color: Color,
    corner_radius: f32,
) -> (NodeId, NodeId, NodeId) {
    // Border rect (slightly larger, drawn first = behind)
    let border_inset = 1.0;
    let border_id = add_rounded_rect(
        scene,
        parent,
        x - border_inset,
        y - border_inset,
        w + border_inset * 2.0,
        h + border_inset * 2.0,
        border_color,
        corner_radius + border_inset,
    );
    // Fill rect (on top)
    let fill_id = add_rounded_rect(scene, parent, x, y, w, h, fill_color, corner_radius);

    // Text node inside the input
    let text_id = add_text(
        scene,
        parent,
        x + 12.0,
        y + 13.0,
        w - 24.0,
        18.0,
        "",
        14.0,
        text_color,
    );

    (border_id, fill_id, text_id)
}

/// Create a themed form scene with layered visual effects.
fn create_themed_form_scene(_tokens: &DesignTokens) -> FormLayout {
    let mut scene = Scene::new();
    let root = scene.root();

    // ── Custom vibrant palette (independent of system accent) ──
    let accent = Color::rgba(124.0 / 255.0, 58.0 / 255.0, 237.0 / 255.0, 1.0); // #7C3AED
    let card_bg = Color::rgba(25.0 / 255.0, 25.0 / 255.0, 40.0 / 255.0, 0.92);
    let card_shadow = Color::rgba(0.0, 0.0, 0.0, 0.35);
    let input_fill = Color::rgba(45.0 / 255.0, 45.0 / 255.0, 65.0 / 255.0, 0.9);
    let input_border = UNFOCUS_BORDER;
    let text_primary = Color::rgba(0.92, 0.92, 0.96, 1.0);
    let text_secondary = Color::rgba(0.58, 0.60, 0.70, 1.0);
    let text_tertiary = Color::rgba(0.45, 0.46, 0.54, 1.0);
    let toast_green = Color::rgba(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0, 0.95); // #22C55E

    // ── Layout constants ──
    let card_x = 36.0;
    let card_y = 36.0;
    let card_w = 428.0;
    let card_h = 540.0;
    let card_r = 14.0;
    let pad = 36.0;
    let field_h = 44.0;
    let field_r = 8.0;
    let label_gap = 6.0;
    let field_gap = 20.0;
    let shadow_offset = 6.0;

    // ── 1. Drop shadow (dark rect offset behind card) ──
    add_rounded_rect(
        &mut scene,
        root,
        card_x + shadow_offset,
        card_y + shadow_offset,
        card_w,
        card_h,
        card_shadow,
        card_r,
    );

    // ── 2. Card background ──
    add_rounded_rect(
        &mut scene, root, card_x, card_y, card_w, card_h, card_bg, card_r,
    );

    // ── 3. Accent strip across top of card ──
    let strip_h = 4.0;
    let strip_inset = 1.0;
    add_rounded_rect(
        &mut scene,
        root,
        card_x + strip_inset,
        card_y,
        card_w - strip_inset * 2.0,
        strip_h,
        accent,
        2.0,
    );

    // ── Content area ──
    let cx = card_x + pad;
    let cw = card_w - pad * 2.0;
    let mut y = card_y + strip_h + pad;

    // ── Title ──
    add_text(
        &mut scene,
        root,
        cx,
        y,
        cw,
        36.0,
        "Welcome Back",
        28.0,
        text_primary,
    );
    y += 36.0 + 4.0;

    // ── Subtitle ──
    add_text(
        &mut scene,
        root,
        cx,
        y,
        cw,
        20.0,
        "Sign in to your account",
        14.0,
        text_secondary,
    );
    y += 20.0 + field_gap + 4.0;

    // ── Input fields ──
    let fields = ["Name", "Email", "Password"];
    let mut field_text_ids = [NodeId(0); 3];
    let mut field_border_ids = [NodeId(0); 3];
    let mut input_fill_ids = [NodeId(0); 3];

    for (i, label_text) in fields.iter().enumerate() {
        add_text(
            &mut scene,
            root,
            cx,
            y,
            cw,
            18.0,
            label_text,
            13.0,
            text_secondary,
        );
        y += 18.0 + label_gap;

        let (border_id, fill_id, text_id) = add_input_field(
            &mut scene,
            root,
            cx,
            y,
            cw,
            field_h,
            input_border,
            input_fill,
            text_primary,
            field_r,
        );
        field_border_ids[i] = border_id;
        input_fill_ids[i] = fill_id;
        field_text_ids[i] = text_id;

        y += field_h + field_gap;
    }

    y += 8.0;

    // ── Submit button (pill-shaped) ──
    let button_h = 48.0;
    let button_r = 24.0;
    let button_id = add_rounded_rect(&mut scene, root, cx, y, cw, button_h, accent, button_r);

    add_text(
        &mut scene,
        root,
        cx + cw / 2.0 - 30.0,
        y + 14.0,
        60.0,
        20.0,
        "Sign In",
        16.0,
        Color::WHITE,
    );
    y += button_h + 20.0;

    // ── Footer text ──
    add_text(
        &mut scene,
        root,
        cx,
        y,
        cw,
        18.0,
        "Don't have an account? Sign up",
        13.0,
        text_tertiary,
    );

    // ── Toast notification (initially invisible) ──
    let toast_x = card_x + 10.0;
    let toast_y = card_y + 10.0;
    let toast_w = card_w - 20.0;
    let toast_h = 44.0;

    let toast_bg_id = add_rounded_rect(
        &mut scene,
        root,
        toast_x,
        toast_y,
        toast_w,
        toast_h,
        toast_green,
        8.0,
    );
    if let Some(node) = scene.get_node_mut(toast_bg_id) {
        node.visible = false;
    }

    let toast_text_id = add_text(
        &mut scene,
        root,
        toast_x + toast_w / 2.0 - 80.0,
        toast_y + 13.0,
        160.0,
        18.0,
        "Signed in successfully!",
        14.0,
        Color::WHITE,
    );
    if let Some(node) = scene.get_node_mut(toast_text_id) {
        node.visible = false;
    }

    FormLayout {
        scene,
        field_text_ids,
        field_border_ids,
        input_fill_ids,
        button_id,
        toast_bg_id,
        toast_text_id,
    }
}

fn main() {
    env_logger::init();
    plat_core::run::<ThemedFormApp>().expect("Failed to run application");
}
