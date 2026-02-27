use serde::{Deserialize, Serialize};

/// Font style (normal, italic, oblique)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FontStyle {
    /// Normal font style
    #[default]
    Normal,
    /// Italic font style
    Italic,
    /// Oblique font style
    Oblique,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextAlign {
    /// Left alignment
    #[default]
    Left,
    /// Center alignment
    Center,
    /// Right alignment
    Right,
    /// Justified alignment
    Justified,
}

/// Line height (auto or fixed)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum LineHeight {
    /// Automatic line height
    #[default]
    Auto,
    /// Fixed line height in pixels
    Fixed(f32),
    /// Relative line height (multiplier)
    Relative(f32),
}

/// Text decoration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextDecoration {
    /// No decoration
    #[default]
    None,
    /// Underline
    Underline,
    /// Line through (strikethrough)
    LineThrough,
}

/// Text case transform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextCase {
    /// Preserve source casing
    #[default]
    Original,
    /// Uppercase transform
    Upper,
    /// Lowercase transform
    Lower,
    /// Title-case transform
    Title,
    /// Small-caps style (renderer may fallback to upper-case)
    SmallCaps,
    /// Forced small-caps style
    SmallCapsForced,
}

/// Vertical text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextAlignVertical {
    /// Align to top
    #[default]
    Top,
    /// Center vertically
    Center,
    /// Align to bottom
    Bottom,
}

/// Text auto-resize behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextAutoResize {
    /// Fixed text box
    #[default]
    None,
    /// Grow width and height to fit content
    WidthAndHeight,
    /// Grow height only
    Height,
    /// Grow width only
    Width,
    /// Truncate within fixed box
    Truncate,
}

/// Text overflow behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextOverflow {
    /// Clip overflow content
    #[default]
    Clip,
    /// Show ellipsis when truncated
    Ellipsis,
}

/// Text content with styling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextContent {
    /// The text string
    pub text: String,
    /// Font size in pixels
    pub font_size: f32,
    /// Font weight (100-900)
    pub font_weight: u16,
    /// Font style
    pub font_style: FontStyle,
    /// Text alignment
    pub align: TextAlign,
    /// Line height
    pub line_height: LineHeight,
    /// Font family name
    pub font_family: Option<String>,
    /// Letter spacing
    pub letter_spacing: f32,
    /// Text decoration
    pub decoration: TextDecoration,
    /// Text case transformation semantics
    pub text_case: TextCase,
    /// Vertical alignment in text bounds
    pub align_vertical: TextAlignVertical,
    /// Auto-resize behavior for text frame
    pub auto_resize: TextAutoResize,
    /// Max visible lines for truncation/wrapping
    pub max_lines: Option<u32>,
    /// Overflow handling mode
    pub overflow: TextOverflow,
    /// Additional spacing after each paragraph
    pub paragraph_spacing: f32,
    /// Paragraph first-line indent
    pub paragraph_indent: f32,
}

impl TextContent {
    /// Create new text content with default styling
    pub fn new(text: impl Into<String>, font_size: f32) -> Self {
        Self {
            text: text.into(),
            font_size,
            font_weight: 400,
            font_style: FontStyle::default(),
            align: TextAlign::default(),
            line_height: LineHeight::default(),
            font_family: None,
            letter_spacing: 0.0,
            decoration: TextDecoration::default(),
            text_case: TextCase::default(),
            align_vertical: TextAlignVertical::default(),
            auto_resize: TextAutoResize::default(),
            max_lines: None,
            overflow: TextOverflow::default(),
            paragraph_spacing: 0.0,
            paragraph_indent: 0.0,
        }
    }

    /// Set font weight to bold (700)
    pub fn bold(mut self) -> Self {
        self.font_weight = 700;
        self
    }

    /// Set font style to italic
    pub fn italic(mut self) -> Self {
        self.font_style = FontStyle::Italic;
        self
    }

    /// Set text alignment
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Set font family
    pub fn family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    /// Set font weight
    pub fn weight(mut self, weight: u16) -> Self {
        self.font_weight = weight;
        self
    }

    /// Set line height
    pub fn line_height(mut self, line_height: LineHeight) -> Self {
        self.line_height = line_height;
        self
    }

    /// Set letter spacing
    pub fn letter_spacing(mut self, spacing: f32) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// Set text decoration
    pub fn decoration(mut self, decoration: TextDecoration) -> Self {
        self.decoration = decoration;
        self
    }

    /// Set text case behavior
    pub fn text_case(mut self, text_case: TextCase) -> Self {
        self.text_case = text_case;
        self
    }

    /// Set vertical text alignment
    pub fn align_vertical(mut self, align: TextAlignVertical) -> Self {
        self.align_vertical = align;
        self
    }

    /// Set text auto-resize behavior
    pub fn auto_resize(mut self, auto_resize: TextAutoResize) -> Self {
        self.auto_resize = auto_resize;
        self
    }

    /// Set maximum line count
    pub fn max_lines(mut self, max_lines: Option<u32>) -> Self {
        self.max_lines = max_lines;
        self
    }

    /// Set overflow mode
    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    /// Set paragraph spacing
    pub fn paragraph_spacing(mut self, spacing: f32) -> Self {
        self.paragraph_spacing = spacing;
        self
    }

    /// Set paragraph indent
    pub fn paragraph_indent(mut self, indent: f32) -> Self {
        self.paragraph_indent = indent;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_content_defaults() {
        let text = TextContent::new("Hello", 16.0);

        assert_eq!(text.text, "Hello");
        assert_eq!(text.font_size, 16.0);
        assert_eq!(text.font_weight, 400);
        assert_eq!(text.font_style, FontStyle::Normal);
        assert_eq!(text.align, TextAlign::Left);
        assert_eq!(text.line_height, LineHeight::Auto);
        assert_eq!(text.font_family, None);
        assert_eq!(text.text_case, TextCase::Original);
        assert_eq!(text.align_vertical, TextAlignVertical::Top);
        assert_eq!(text.auto_resize, TextAutoResize::None);
        assert_eq!(text.max_lines, None);
        assert_eq!(text.overflow, TextOverflow::Clip);
        assert_eq!(text.paragraph_spacing, 0.0);
        assert_eq!(text.paragraph_indent, 0.0);
    }

    #[test]
    fn test_builder_chain() {
        let text = TextContent::new("Hello", 16.0)
            .bold()
            .italic()
            .align(TextAlign::Center)
            .family("Inter");

        assert_eq!(text.font_weight, 700);
        assert_eq!(text.font_style, FontStyle::Italic);
        assert_eq!(text.align, TextAlign::Center);
        assert_eq!(text.font_family, Some("Inter".to_string()));
    }

    #[test]
    fn test_line_height_default() {
        let text = TextContent::new("Hello", 16.0);
        assert_eq!(text.line_height, LineHeight::Auto);
    }

    #[test]
    fn test_line_height_fixed() {
        let text = TextContent::new("Hello", 16.0).line_height(LineHeight::Fixed(24.0));
        assert_eq!(text.line_height, LineHeight::Fixed(24.0));
    }

    #[test]
    fn test_line_height_relative() {
        let text = TextContent::new("Hello", 16.0).line_height(LineHeight::Relative(1.5));
        assert_eq!(text.line_height, LineHeight::Relative(1.5));
    }

    #[test]
    fn test_serde_roundtrip() {
        let text = TextContent::new("Hello", 16.0)
            .bold()
            .italic()
            .align(TextAlign::Center)
            .text_case(TextCase::Upper)
            .align_vertical(TextAlignVertical::Center)
            .auto_resize(TextAutoResize::Height)
            .max_lines(Some(2))
            .overflow(TextOverflow::Ellipsis)
            .paragraph_spacing(6.0)
            .paragraph_indent(8.0);

        let json = serde_json::to_string(&text).expect("serialize failed");
        let deserialized: TextContent = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(text, deserialized);
    }
}
