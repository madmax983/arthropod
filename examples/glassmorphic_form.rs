//! Canonical Glassmorphic Form example with interactive fields and toast feedback.
//!
//! Run with:
//! cargo run --example glassmorphic_form

use std::time::{Duration, Instant};

use glam::Vec4;
use plat_core::{
    Application, BackdropMaterial, ControlFlow, ElementState, Event, EventLoop, Key, MouseButton,
    Rect, Size, Window, WindowConfig, WindowEvent, WindowId,
};
use render_engine::{
    Color, NodeContent, NodeId, Paint, Scene, SceneNode, TextContent, backend::WgpuBackend,
};
use style_engine::{
    ColorStop, Effect, ImageFill, ImageId, ImageScaleMode, LinearGradient, Paint as StylePaint,
    StrokeAlign, StrokeStyle, VisualStyle,
};

const FORM_BACKGROUND_ID: ImageId = ImageId(80_001);
const FORM_NOISE_ID: ImageId = ImageId(80_002);

const FIELD_COUNT: usize = 3;
const FIELD_LABELS: [&str; FIELD_COUNT] = ["Full Name", "Email", "Password"];
const FIELD_PLACEHOLDERS: [&str; FIELD_COUNT] = ["Ada Lovelace", "ada@curie.ui", "**********"];
const TOAST_DURATION: Duration = Duration::from_millis(2_200);
const MAX_FIELD_LEN: usize = 64;

const FOCUS_BORDER: Color = Color::rgba(0.46, 0.78, 1.0, 0.98);
const UNFOCUS_BORDER: Color = Color::rgba(1.0, 1.0, 1.0, 0.28);

fn c(r: f32, g: f32, b: f32, a: f32) -> Vec4 {
    Color::rgba(r, g, b, a).as_vec4()
}

struct FormLayout {
    scene: Scene,
    field_box_ids: [NodeId; FIELD_COUNT],
    field_text_ids: [NodeId; FIELD_COUNT],
    button_id: NodeId,
    toast_bg_id: NodeId,
    toast_text_id: NodeId,
}

struct GlassmorphicFormApp {
    backend: WgpuBackend,
    window: Window,
    scene: Scene,
    focused_field: Option<usize>,
    field_values: [String; FIELD_COUNT],
    field_box_ids: [NodeId; FIELD_COUNT],
    field_text_ids: [NodeId; FIELD_COUNT],
    button_id: NodeId,
    toast_bg_id: NodeId,
    toast_text_id: NodeId,
    toast_created_at: Option<Instant>,
}

fn make_hero_image(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let fx = x as f32 / width as f32;
            let fy = y as f32 / height as f32;
            let base = 60.0 + 35.0 * (fx * 7.0).sin() * (fy * 9.0).cos();
            out[idx] = (base + 150.0).clamp(0.0, 255.0) as u8;
            out[idx + 1] = (base + 110.0 + 20.0 * (fy * 11.0).sin()).clamp(0.0, 255.0) as u8;
            out[idx + 2] = (base + 170.0 + 40.0 * (fx * 13.0).cos()).clamp(0.0, 255.0) as u8;
            out[idx + 3] = 232;
        }
    }
    out
}

fn make_noise(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let v =
                ((x.wrapping_mul(17) + y.wrapping_mul(73) + x.wrapping_mul(y + 13)) % 255) as u8;
            out[idx] = v;
            out[idx + 1] = v;
            out[idx + 2] = v;
            out[idx + 3] = 32;
        }
    }
    out
}

fn add_styled_node(scene: &mut Scene, parent: NodeId, bounds: Rect, style: VisualStyle) -> NodeId {
    let node = SceneNode::new(NodeContent::Styled {
        style: Box::new(style),
    });
    let id = scene.add_node(parent, node);
    if let Some(n) = scene.get_node_mut(id) {
        n.bounds = bounds;
    }
    id
}

fn add_empty_node(scene: &mut Scene, parent: NodeId, bounds: Rect) -> NodeId {
    let node = SceneNode::new(NodeContent::Empty);
    let id = scene.add_node(parent, node);
    if let Some(n) = scene.get_node_mut(id) {
        n.bounds = bounds;
    }
    id
}

#[allow(clippy::too_many_arguments)]
fn add_text(
    scene: &mut Scene,
    parent: NodeId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &str,
    size: f32,
    color: Color,
) -> NodeId {
    add_styled_node(
        scene,
        parent,
        Rect::new(x, y, w, h),
        VisualStyle::new()
            .solid_fill(color.as_vec4())
            .text(TextContent::new(text.to_string(), size)),
    )
}

#[allow(clippy::too_many_arguments)]
fn add_field(
    scene: &mut Scene,
    parent: NodeId,
    label: &str,
    placeholder: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> (NodeId, NodeId) {
    add_text(
        scene,
        parent,
        x,
        y - 22.0,
        w,
        14.0,
        label,
        12.0,
        Color::rgba(0.95, 0.98, 1.0, 0.95),
    );
    let field_id = add_styled_node(
        scene,
        parent,
        Rect::new(x, y, w, h),
        VisualStyle::new()
            .solid_fill(c(0.03, 0.07, 0.14, 0.92))
            .corner_radius(12.0)
            .clips_content(true)
            .stroke(StrokeStyle::solid(
                Paint::Solid(UNFOCUS_BORDER.as_vec4()),
                1.1,
                StrokeAlign::Inside,
            )),
    );
    let text_id = add_text(
        scene,
        field_id,
        x + 12.0,
        y + 11.0,
        w - 24.0,
        16.0,
        placeholder,
        14.0,
        Color::rgba(0.84, 0.90, 0.98, 0.68),
    );
    (field_id, text_id)
}

fn display_value(field_idx: usize, raw: &str) -> String {
    if raw.is_empty() {
        return FIELD_PLACEHOLDERS[field_idx].to_string();
    }
    if field_idx == 2 {
        return "*".repeat(raw.chars().count());
    }
    raw.to_string()
}

fn next_focus(current: Option<usize>, reverse: bool) -> usize {
    match (current, reverse) {
        (None, _) => 0,
        (Some(0), true) => FIELD_COUNT - 1,
        (Some(idx), true) => idx - 1,
        (Some(idx), false) => (idx + 1) % FIELD_COUNT,
    }
}

fn build_scene(width: f32, height: f32) -> FormLayout {
    let mut scene = Scene::new();
    let root = scene.root();

    add_styled_node(
        &mut scene,
        root,
        Rect::new(0.0, 0.0, width, height),
        VisualStyle::new().fill(Paint::Linear(LinearGradient {
            start: glam::Vec2::new(0.0, 0.0),
            end: glam::Vec2::new(1.0, 1.0),
            stops: vec![
                ColorStop::new(0.0, c(0.005, 0.02, 0.06, 1.0)),
                ColorStop::new(0.55, c(0.02, 0.06, 0.13, 1.0)),
                ColorStop::new(1.0, c(0.01, 0.04, 0.10, 1.0)),
            ],
        })),
    );

    add_styled_node(
        &mut scene,
        root,
        Rect::new(0.0, 0.0, width, height),
        VisualStyle::new()
            .fill(StylePaint::Image(ImageFill {
                image_id: FORM_BACKGROUND_ID,
                scale_mode: ImageScaleMode::Crop,
                transform: None,
            }))
            .opacity(0.08),
    );

    add_styled_node(
        &mut scene,
        root,
        Rect::new(-180.0, -110.0, 360.0, 360.0),
        VisualStyle::new()
            .solid_fill(c(0.16, 0.35, 0.98, 0.14))
            .corner_radius(180.0),
    );

    add_styled_node(
        &mut scene,
        root,
        Rect::new(width - 260.0, height - 280.0, 340.0, 340.0),
        VisualStyle::new()
            .solid_fill(c(0.76, 0.18, 0.82, 0.14))
            .corner_radius(180.0),
    );

    let card_w = (width - 160.0).clamp(760.0, 940.0);
    let card_h = (height - 180.0).clamp(420.0, 540.0);
    let card_x = (width - card_w) / 2.0;
    let card_y = (height - card_h) / 2.0;
    let side_label_y = card_y + 0.62 * card_h;
    let side_blurb_y = side_label_y + 28.0;

    let panel_w = (card_w * 0.42).clamp(290.0, 380.0);
    let form_x = card_x + panel_w + 40.0;
    let form_w = card_w - panel_w - 84.0;
    let form_top = card_y + 42.0;

    let card_background = add_styled_node(
        &mut scene,
        root,
        Rect::new(card_x, card_y, card_w, card_h),
        VisualStyle::new()
            .solid_fill(c(0.03, 0.06, 0.12, 0.82))
            .opacity(1.0)
            .stroke(StrokeStyle::solid(
                Paint::Solid(c(0.95, 0.98, 1.0, 0.34)),
                1.1,
                StrokeAlign::Inside,
            ))
            .corner_radius(28.0)
            .effect(Effect::drop_shadow(
                glam::Vec2::new(0.0, 14.0),
                44.0,
                c(0.0, 0.0, 0.0, 0.64),
            )),
    );

    let card = add_empty_node(&mut scene, root, Rect::new(card_x, card_y, card_w, card_h));

    add_styled_node(
        &mut scene,
        card_background,
        Rect::new(card_x + 22.0, card_y + 22.0, card_w - 44.0, card_h - 44.0),
        VisualStyle::new()
            .fill(StylePaint::Image(ImageFill {
                image_id: FORM_NOISE_ID,
                scale_mode: ImageScaleMode::Crop,
                transform: None,
            }))
            .opacity(0.004),
    );

    let side_base_x = card_x + 22.0;
    let side_base_y = card_y + 22.0;
    let side_badge_y = card_y + card_h - 136.0;

    let side_base = add_styled_node(
        &mut scene,
        card,
        Rect::new(side_base_x, side_base_y, panel_w, card_h - 44.0),
        VisualStyle::new()
            .solid_fill(c(0.06, 0.10, 0.18, 0.64))
            .corner_radius(22.0)
            .stroke(StrokeStyle::solid(
                Paint::Solid(c(0.96, 0.98, 1.0, 0.34)),
                1.0,
                StrokeAlign::Inside,
            )),
    );

    add_styled_node(
        &mut scene,
        side_base,
        Rect::new(
            side_base_x + 2.0,
            side_base_y + 18.0,
            panel_w - 16.0,
            (card_h - 96.0) * 0.58,
        ),
        VisualStyle::new()
            .fill(StylePaint::Image(ImageFill {
                image_id: FORM_BACKGROUND_ID,
                scale_mode: ImageScaleMode::Fill,
                transform: None,
            }))
            .corner_radius(14.0)
            .opacity(0.52),
    );

    add_text(
        &mut scene,
        side_base,
        side_base_x + 20.0,
        side_label_y,
        panel_w - 52.0,
        32.0,
        "Glassmorphic Asset Layer",
        16.0,
        Color::rgba(0.98, 0.99, 1.0, 0.92),
    );
    add_text(
        &mut scene,
        side_base,
        side_base_x + 20.0,
        side_blurb_y,
        panel_w - 52.0,
        20.0,
        "Image-backed fills + tint + noise",
        11.0,
        Color::rgba(0.76, 0.87, 0.98, 0.84),
    );

    add_styled_node(
        &mut scene,
        side_base,
        Rect::new(side_base_x + 20.0, side_badge_y, 108.0, 36.0),
        VisualStyle::new()
            .solid_fill(c(0.18, 0.38, 1.0, 0.42))
            .corner_radius(12.0),
    );
    add_text(
        &mut scene,
        side_base,
        side_base_x + 34.0,
        side_badge_y + 10.0,
        80.0,
        16.0,
        "Frosted",
        11.0,
        Color::rgba(1.0, 1.0, 1.0, 0.94),
    );

    add_text(
        &mut scene,
        card,
        form_x,
        form_top,
        form_w,
        40.0,
        "Create your account",
        36.0,
        Color::rgba(1.0, 1.0, 1.0, 0.98),
    );
    add_text(
        &mut scene,
        card,
        form_x,
        form_top + 50.0,
        form_w,
        28.0,
        "Type in fields and click Create Account to trigger toast.",
        14.0,
        Color::rgba(0.84, 0.91, 0.98, 0.86),
    );

    let field_w = form_w - 8.0;
    let mut field_box_ids = [NodeId(0); FIELD_COUNT];
    let mut field_text_ids = [NodeId(0); FIELD_COUNT];
    for (idx, label) in FIELD_LABELS.iter().enumerate() {
        let field_y = form_top + 92.0 + 72.0 * idx as f32;
        let (box_id, text_id) = add_field(
            &mut scene,
            card,
            label,
            FIELD_PLACEHOLDERS[idx],
            form_x,
            field_y,
            field_w,
            44.0,
        );
        field_box_ids[idx] = box_id;
        field_text_ids[idx] = text_id;
    }

    let button_id = add_styled_node(
        &mut scene,
        card,
        Rect::new(form_x, form_top + 318.0, 184.0, 46.0),
        VisualStyle::new()
            .solid_fill(c(0.19, 0.49, 0.84, 0.96))
            .stroke(StrokeStyle::solid(
                Paint::Solid(c(1.0, 1.0, 1.0, 0.42)),
                1.1,
                StrokeAlign::Inside,
            ))
            .effect(Effect::drop_shadow(
                glam::Vec2::new(0.0, 10.0),
                18.0,
                c(0.0, 0.0, 0.0, 0.46),
            ))
            .corner_radius(14.0),
    );
    add_text(
        &mut scene,
        card,
        form_x + 32.0,
        form_top + 331.5,
        128.0,
        20.0,
        "Create Account",
        14.0,
        Color::rgba(0.98, 0.99, 1.0, 0.98),
    );

    add_text(
        &mut scene,
        card,
        form_x,
        form_top + 376.0,
        form_w,
        16.0,
        "By creating an account you agree to the terms and privacy policy.",
        11.0,
        Color::rgba(0.82, 0.90, 0.98, 0.80),
    );

    let toast_bg_id = add_styled_node(
        &mut scene,
        root,
        Rect::new(card_x + card_w - 300.0, card_y + 16.0, 280.0, 40.0),
        VisualStyle::new()
            .solid_fill(c(0.10, 0.75, 0.44, 0.96))
            .corner_radius(10.0),
    );
    if let Some(node) = scene.get_node_mut(toast_bg_id) {
        node.visible = false;
    }

    let toast_text_id = add_text(
        &mut scene,
        root,
        card_x + card_w - 284.0,
        card_y + 28.0,
        248.0,
        18.0,
        "Account created (demo)",
        13.0,
        Color::WHITE,
    );
    if let Some(node) = scene.get_node_mut(toast_text_id) {
        node.visible = false;
    }

    FormLayout {
        scene,
        field_box_ids,
        field_text_ids,
        button_id,
        toast_bg_id,
        toast_text_id,
    }
}

impl GlassmorphicFormApp {
    fn set_focus_border(&mut self, idx: usize, color: Color) {
        if let Some(node) = self.scene.get_node_mut(self.field_box_ids[idx])
            && let NodeContent::Styled { style } = &mut node.content
            && let Some(stroke) = &mut style.stroke
        {
            if stroke.paints.is_empty() {
                stroke.paints.push(Paint::Solid(color.as_vec4()));
            } else {
                stroke.paints[0] = Paint::Solid(color.as_vec4());
            }
        }
    }

    fn set_focus(&mut self, next: Option<usize>) {
        if let Some(prev) = self.focused_field {
            self.set_focus_border(prev, UNFOCUS_BORDER);
        }
        self.focused_field = next;
        if let Some(idx) = next {
            self.set_focus_border(idx, FOCUS_BORDER);
        }
    }

    fn update_field_text_node(&mut self, idx: usize) {
        let display = display_value(idx, &self.field_values[idx]);
        let is_placeholder = self.field_values[idx].is_empty();
        let color = if is_placeholder {
            Color::rgba(0.84, 0.90, 0.98, 0.68)
        } else {
            Color::rgba(0.99, 1.0, 1.0, 0.97)
        };

        if let Some(node) = self.scene.get_node_mut(self.field_text_ids[idx])
            && let NodeContent::Styled { style } = &mut node.content
        {
            if let Some(text) = &mut style.text {
                text.text = display;
            }
            if style.fills.is_empty() {
                style.fills.push(Paint::Solid(color.as_vec4()));
            } else {
                style.fills[0] = Paint::Solid(color.as_vec4());
            }
        }
    }

    fn update_all_field_text_nodes(&mut self) {
        for idx in 0..FIELD_COUNT {
            self.update_field_text_node(idx);
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

    fn rebuild_scene(&mut self, width: f32, height: f32) {
        let preserve_focus = self.focused_field;
        let preserve_toast = self.toast_created_at.is_some();
        let layout = build_scene(width, height);
        self.scene = layout.scene;
        self.field_box_ids = layout.field_box_ids;
        self.field_text_ids = layout.field_text_ids;
        self.button_id = layout.button_id;
        self.toast_bg_id = layout.toast_bg_id;
        self.toast_text_id = layout.toast_text_id;
        self.focused_field = None;
        self.update_all_field_text_nodes();
        self.set_focus(preserve_focus);
        if preserve_toast {
            self.show_toast();
        }
    }
}

impl Application for GlassmorphicFormApp {
    fn new(event_loop: &EventLoop) -> Self {
        let config = WindowConfig {
            title: "Arthropod Glassmorphic Form".to_string(),
            size: Size::new(1160, 760),
            transparent: true,
            visible: true,
            ..Default::default()
        };
        let window = event_loop
            .create_window(config)
            .expect("Failed to create window");
        window.set_backdrop_material(BackdropMaterial::Mica);

        let size = window.inner_size();
        // SAFETY: The window drops after the backend in GlassmorphicFormApp, satisfying the wgpu::Surface 'static lifetime requirement.
        let mut backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .expect("Failed to create backend");
        backend.set_clear_color(Color::rgba(0.03, 0.05, 0.12, 1.0));

        backend
            .register_image_rgba8(FORM_BACKGROUND_ID, 256, 256, make_hero_image(256, 256))
            .expect("Failed to register hero image");
        backend
            .register_image_rgba8(FORM_NOISE_ID, 512, 512, make_noise(512, 512))
            .expect("Failed to register noise image");

        let layout = build_scene(size.width as f32, size.height as f32);

        let mut app = Self {
            backend,
            window,
            scene: layout.scene,
            focused_field: None,
            field_values: [String::new(), String::new(), String::new()],
            field_box_ids: layout.field_box_ids,
            field_text_ids: layout.field_text_ids,
            button_id: layout.button_id,
            toast_bg_id: layout.toast_bg_id,
            toast_text_id: layout.toast_text_id,
            toast_created_at: None,
        };
        app.update_all_field_text_nodes();
        app
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::Window {
                event: WindowEvent::Resized(size),
                ..
            } => {
                self.backend.resize(size.width, size.height);
                self.rebuild_scene(size.width as f32, size.height as f32);
                self.window.request_redraw();
            }
            Event::Window {
                event: WindowEvent::KeyboardInput(input),
                ..
            } => {
                if input.state != ElementState::Pressed {
                    return;
                }

                if input.key == Key::Escape {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if input.key == Key::Tab {
                    let next = next_focus(self.focused_field, input.modifiers.shift);
                    self.set_focus(Some(next));
                    self.window.request_redraw();
                    return;
                }

                if input.key == Key::Enter {
                    self.show_toast();
                    self.window.request_redraw();
                    return;
                }

                let Some(idx) = self.focused_field else {
                    return;
                };

                match input.key {
                    Key::Backspace => {
                        self.field_values[idx].pop();
                        self.update_field_text_node(idx);
                        self.window.request_redraw();
                    }
                    key => {
                        if let Some(ch) = key.to_char(input.modifiers.shift)
                            && !ch.is_control()
                            && self.field_values[idx].chars().count() < MAX_FIELD_LEN
                        {
                            self.field_values[idx].push(ch);
                            self.update_field_text_node(idx);
                            self.window.request_redraw();
                        }
                    }
                }
            }
            Event::Window {
                event: WindowEvent::MouseInput(mouse),
                ..
            } => {
                if mouse.button != MouseButton::Left || mouse.state != ElementState::Pressed {
                    return;
                }
                let x = mouse.position.x as f32;
                let y = mouse.position.y as f32;

                if let Some(button) = self.scene.get_node(self.button_id)
                    && button.bounds.contains(x, y)
                {
                    self.show_toast();
                    self.window.request_redraw();
                    return;
                }

                let mut clicked = None;
                for (idx, node_id) in self.field_box_ids.iter().enumerate() {
                    if let Some(field) = self.scene.get_node(*node_id)
                        && field.bounds.contains(x, y)
                    {
                        clicked = Some(idx);
                        break;
                    }
                }
                self.set_focus(clicked);
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        if let Some(created_at) = self.toast_created_at {
            if created_at.elapsed() >= TOAST_DURATION {
                self.hide_toast();
            } else {
                self.window.request_redraw();
            }
        }

        if let Err(e) = self.backend.render(&self.scene) {
            eprintln!("Render error: {e}");
        }
    }
}

fn main() {
    if let Err(e) = plat_core::run::<GlassmorphicFormApp>() {
        eprintln!("Failed to run glassmorphic form example: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::{display_value, next_focus};

    #[test]
    fn test_next_focus_forward_wraps() {
        assert_eq!(next_focus(None, false), 0);
        assert_eq!(next_focus(Some(0), false), 1);
        assert_eq!(next_focus(Some(2), false), 0);
    }

    #[test]
    fn test_next_focus_reverse_wraps() {
        assert_eq!(next_focus(None, true), 0);
        assert_eq!(next_focus(Some(0), true), 2);
        assert_eq!(next_focus(Some(2), true), 1);
    }

    #[test]
    fn test_display_value_uses_placeholder_for_empty() {
        assert_eq!(display_value(0, ""), "Ada Lovelace");
        assert_eq!(display_value(1, ""), "ada@curie.ui");
    }

    #[test]
    fn test_display_value_masks_password() {
        assert_eq!(display_value(2, "hunter2"), "*******");
    }
}
