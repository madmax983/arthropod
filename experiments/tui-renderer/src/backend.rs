use plat_core::Rect;
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::Rect as TuiRect,
    style::{Color as TuiColor, Style, Stylize},
    widgets::{Block, Borders, Paragraph},
};
use render_engine::{Color, NodeContent, RendererError, Scene};
use std::io::Stdout;

pub struct TuiBackend<B: Backend> {
    terminal: Terminal<B>,
    clear_color: Color,
    #[allow(dead_code)]
    width: u32,
    #[allow(dead_code)]
    height: u32,
}

impl Default for TuiBackend<CrosstermBackend<Stdout>> {
    fn default() -> Self {
        Self::new().expect("Failed to create default TuiBackend")
    }
}

impl TuiBackend<CrosstermBackend<Stdout>> {
    pub fn new() -> Result<Self, RendererError> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;

        Ok(Self {
            terminal,
            clear_color: Color::BLACK,
            width: 0,
            height: 0,
        })
    }
}

impl<B: Backend> TuiBackend<B> {
    pub fn new_with_backend(backend: B) -> Result<Self, RendererError> {
        let terminal = Terminal::new(backend)
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;

        Ok(Self {
            terminal,
            clear_color: Color::BLACK,
            width: 0,
            height: 0,
        })
    }
}

// Helper to map f32 Rect to u16 TuiRect
fn map_rect(rect: Rect) -> TuiRect {
    // Simple rounding
    let x = rect.x.round().max(0.0) as u16;
    let y = rect.y.round().max(0.0) as u16;
    let width = rect.width.round().max(0.0) as u16;
    let height = rect.height.round().max(0.0) as u16;

    TuiRect::new(x, y, width, height)
}

// Helper to map Color to TuiColor
fn map_color(color: Color) -> TuiColor {
    let r = (color.r() * 255.0) as u8;
    let g = (color.g() * 255.0) as u8;
    let b = (color.b() * 255.0) as u8;
    TuiColor::Rgb(r, g, b)
}

impl<B: Backend> TuiBackend<B> {
    pub fn render(&mut self, scene: &Scene) -> Result<(), RendererError> {
        let clear_color = self.clear_color; // Copy for closure

        self.terminal
            .draw(|frame| {
                // Fill background with clear color
                let size = frame.area();
                let block = Block::default().bg(map_color(clear_color));
                frame.render_widget(block, size);

                // Iterate visuals in Z-order
                for (_id, node, _) in scene.iter_visuals() {
                    if !node.visible || node.opacity <= 0.0 {
                        continue;
                    }

                    let rect = map_rect(node.bounds);

                    // Simple intersection check with frame area to avoid out of bounds panic if any
                    if rect.area() == 0 {
                        continue;
                    }

                    match &node.content {
                        NodeContent::Styled { style } => {
                            // Background color
                            let mut bg = TuiColor::Reset;
                            if let Some(style_engine::Paint::Solid(c)) = style.fills.first() {
                                bg = map_color(Color::from_vec4(*c));
                            }

                            // Text Content
                            if let Some(text_content) = &style.text {
                                // Determine text color
                                let mut fg = TuiColor::White;
                                if let Some(style_engine::Paint::Solid(c)) = style.fills.first() {
                                    fg = map_color(Color::from_vec4(*c));
                                }

                                let p = Paragraph::new(text_content.text.clone())
                                    .style(Style::default().fg(fg));

                                frame.render_widget(p, rect);
                            } else {
                                // Rectangle / Shape
                                let mut block = Block::default().bg(bg);

                                // Border / Stroke
                                if let Some(stroke) = &style.stroke
                                    && let Some(style_engine::Paint::Solid(c)) =
                                        stroke.paints.first()
                                {
                                    block = block.borders(Borders::ALL).border_style(
                                        Style::default().fg(map_color(Color::from_vec4(*c))),
                                    );
                                }

                                // Corner radius -> rounded corners?
                                if style.corner_radii.top_left > 0.0 {
                                    block =
                                        block.border_type(ratatui::widgets::BorderType::Rounded);
                                }

                                frame.render_widget(block, rect);
                            }
                        }
                        NodeContent::SolidColor { color } => {
                            let bg = map_color(*color);
                            let block = Block::default().bg(bg);
                            frame.render_widget(block, rect);
                        }
                        NodeContent::Empty => {}
                    }
                }
            })
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }
}
