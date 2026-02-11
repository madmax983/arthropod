use crate::{
    Color, blend::BlendMode, corner::CornerRadii, effect::Effect, paint::Paint, path::VectorPath,
    stroke::StrokeStyle, text::TextContent,
};
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// VisualStyle - unified styling for all visual primitives
///
/// This is the central type that maps 1:1 to Figma's visual properties.
/// Every visible primitive (rect, path, text) has a VisualStyle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualStyle {
    /// Fill paints (applied bottom-to-top)
    pub fills: Vec<Paint>,
    /// Stroke style (optional)
    pub stroke: Option<StrokeStyle>,
    /// Visual effects (shadows, blur)
    pub effects: Vec<Effect>,
    /// Corner radii (for rectangles)
    pub corner_radii: CornerRadii,
    /// Corner smoothing (0.0 = circular corners, 1.0 = superellipse-like)
    pub corner_smoothing: f32,
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
    /// Blend mode
    pub blend_mode: BlendMode,
    /// Whether children should be clipped to this node's bounds
    pub clips_content: bool,
    /// Text content (for text nodes)
    pub text: Option<TextContent>,
    /// Fill geometry for custom vector shapes (None = rectangle)
    pub fill_geometry: Option<Vec<VectorPath>>,
    /// Stroke geometry for custom vector shapes (None = use fill geometry)
    pub stroke_geometry: Option<Vec<VectorPath>>,
}

impl Default for VisualStyle {
    fn default() -> Self {
        Self {
            fills: Vec::new(),
            stroke: None,
            effects: Vec::new(),
            corner_radii: CornerRadii::ZERO,
            corner_smoothing: 0.0,
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            clips_content: false,
            text: None,
            fill_geometry: None,
            stroke_geometry: None,
        }
    }
}

impl VisualStyle {
    /// Create a new empty visual style
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a solid fill
    pub fn solid_fill(mut self, color: Color) -> Self {
        self.fills.push(Paint::solid(color));
        self
    }

    /// Add a fill paint
    pub fn fill(mut self, paint: Paint) -> Self {
        self.fills.push(paint);
        self
    }

    /// Set corner radius (uniform)
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radii = CornerRadii::uniform(radius);
        self
    }

    /// Set corner radii (per-corner)
    pub fn corner_radii(mut self, radii: CornerRadii) -> Self {
        self.corner_radii = radii;
        self
    }

    /// Set corner smoothing amount
    pub fn corner_smoothing(mut self, amount: f32) -> Self {
        self.corner_smoothing = amount.clamp(0.0, 1.0);
        self
    }

    /// Add a drop shadow effect
    pub fn drop_shadow(mut self, offset: Vec2, blur: f32, color: Color) -> Self {
        self.effects.push(Effect::drop_shadow(offset, blur, color));
        self
    }

    /// Add an effect
    pub fn effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }

    /// Set stroke style
    pub fn stroke(mut self, stroke: StrokeStyle) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Set text content
    pub fn text(mut self, text: TextContent) -> Self {
        self.text = Some(text);
        self
    }

    /// Set opacity
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set blend mode
    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    /// Set whether this node clips child content to its bounds
    pub fn clips_content(mut self, clips: bool) -> Self {
        self.clips_content = clips;
        self
    }

    /// Set fill geometry paths
    pub fn fill_geometry(mut self, geometry: Vec<VectorPath>) -> Self {
        self.fill_geometry = Some(geometry);
        self
    }

    /// Set stroke geometry paths
    pub fn stroke_geometry(mut self, geometry: Vec<VectorPath>) -> Self {
        self.stroke_geometry = Some(geometry);
        self
    }

    /// Set vector path
    pub fn path(mut self, path: VectorPath) -> Self {
        self.fill_geometry = Some(vec![path]);
        self
    }
}

// Migration helpers for old node types
impl VisualStyle {
    /// Create style from old Rect (solid color only)
    pub fn from_rect_color(color: Color) -> Self {
        Self::new().solid_fill(color)
    }

    /// Create style from old RoundedRect
    pub fn from_rounded_rect(color: Color, corner_radius: f32) -> Self {
        Self::new().solid_fill(color).corner_radius(corner_radius)
    }

    /// Create style from old Text node
    pub fn from_text(text: String, font_size: f32, color: Color) -> Self {
        Self::new()
            .solid_fill(color)
            .text(TextContent::new(text, font_size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlendMode, stroke::StrokeAlign};
    use glam::{Vec2, Vec4};

    #[test]
    fn test_default_visual_style() {
        let style = VisualStyle::default();

        assert!(style.fills.is_empty(), "should have no fills by default");
        assert!(style.stroke.is_none(), "should have no stroke by default");
        assert!(
            style.effects.is_empty(),
            "should have no effects by default"
        );
        assert_eq!(style.corner_radii, CornerRadii::ZERO);
        assert_eq!(style.corner_smoothing, 0.0);
        assert_eq!(style.opacity, 1.0);
        assert_eq!(style.blend_mode, BlendMode::Normal);
        assert!(!style.clips_content);
        assert!(style.text.is_none());
        assert!(style.fill_geometry.is_none());
        assert!(style.stroke_geometry.is_none());
    }

    #[test]
    fn test_builder_multiple_fills() {
        let red = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let blue = Vec4::new(0.0, 0.0, 1.0, 1.0);

        let style = VisualStyle::new().solid_fill(red).solid_fill(blue);

        assert_eq!(style.fills.len(), 2);
        assert_eq!(style.fills[0], Paint::solid(red));
        assert_eq!(style.fills[1], Paint::solid(blue));
    }

    #[test]
    fn test_builder_corner_radius() {
        let style = VisualStyle::new().corner_radius(12.0);

        assert_eq!(style.corner_radii, CornerRadii::uniform(12.0));
        assert!(style.corner_radii.is_uniform());
    }

    #[test]
    fn test_builder_corner_smoothing_clamps() {
        let style = VisualStyle::new().corner_smoothing(1.5);
        assert_eq!(style.corner_smoothing, 1.0);
    }

    #[test]
    fn test_builder_drop_shadow() {
        let offset = Vec2::new(2.0, 2.0);
        let blur = 4.0;
        let color = Vec4::new(0.0, 0.0, 0.0, 0.5);

        let style = VisualStyle::new().drop_shadow(offset, blur, color);

        assert_eq!(style.effects.len(), 1);
    }

    #[test]
    fn test_builder_stroke() {
        let black = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let stroke = StrokeStyle::solid(Paint::solid(black), 2.0, StrokeAlign::Inside);

        let style = VisualStyle::new().stroke(stroke.clone());

        assert!(style.stroke.is_some());
        assert_eq!(style.stroke.unwrap(), stroke);
    }

    #[test]
    fn test_builder_text() {
        let text_content = TextContent::new("Hello", 16.0);

        let style = VisualStyle::new().text(text_content.clone());

        assert!(style.text.is_some());
        assert_eq!(style.text.unwrap(), text_content);
    }

    #[test]
    fn test_builder_clips_content() {
        let style = VisualStyle::new().clips_content(true);
        assert!(style.clips_content);
    }

    #[test]
    fn test_gradient_text() {
        use crate::paint::{ColorStop, LinearGradient};

        let gradient = Paint::Linear(LinearGradient {
            start: Vec2::ZERO,
            end: Vec2::new(100.0, 0.0),
            stops: vec![
                ColorStop::new(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)),
                ColorStop::new(1.0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
            ],
        });

        let text_content = TextContent::new("Gradient Text", 16.0);

        let style = VisualStyle::new().fill(gradient).text(text_content);

        assert_eq!(style.fills.len(), 1);
        assert!(style.text.is_some());
    }

    #[test]
    fn test_migration_from_rect() {
        let color = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let style = VisualStyle::from_rect_color(color);

        assert_eq!(style.fills.len(), 1);
        assert_eq!(style.fills[0], Paint::solid(color));
        assert_eq!(style.corner_radii, CornerRadii::ZERO);
    }

    #[test]
    fn test_migration_from_rounded_rect() {
        let color = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let style = VisualStyle::from_rounded_rect(color, 8.0);

        assert_eq!(style.fills.len(), 1);
        assert_eq!(style.fills[0], Paint::solid(color));
        assert_eq!(style.corner_radii, CornerRadii::uniform(8.0));
    }

    #[test]
    fn test_migration_from_text() {
        let color = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let style = VisualStyle::from_text("Hello World".to_string(), 16.0, color);

        assert_eq!(style.fills.len(), 1);
        assert_eq!(style.fills[0], Paint::solid(color));
        assert!(style.text.is_some());

        let text = style.text.unwrap();
        assert_eq!(text.text, "Hello World");
        assert_eq!(text.font_size, 16.0);
    }

    #[test]
    fn test_serde_roundtrip() {
        let style = VisualStyle::new()
            .solid_fill(Vec4::new(1.0, 0.0, 0.0, 1.0))
            .corner_radius(12.0)
            .opacity(0.8);

        let json = serde_json::to_string(&style).expect("serialize failed");
        let deserialized: VisualStyle = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(style, deserialized);
    }
}
