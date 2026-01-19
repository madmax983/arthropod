//! Layer 3: Style API - CSS-like component styling
//!
//! Provides a declarative API for styling components using design tokens.
//! Currently a placeholder for future CSS-like macro implementation.

use crate::{Color, TokenValue};

/// Style properties for UI components
///
/// Will be expanded with the style! macro in future iterations.
/// For now, provides a builder API for common properties.
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub background: Option<TokenValue>,
    pub padding: Option<Padding>,
    pub border_radius: Option<f32>,
    pub color: Option<Color>,
}

/// Padding values (top, right, bottom, left)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    /// Create uniform padding
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create symmetric padding (vertical, horizontal)
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create padding with individual values
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl Style {
    /// Create a new empty style
    pub fn new() -> Self {
        Self::default()
    }

    /// Set background
    pub fn background(mut self, background: TokenValue) -> Self {
        self.background = Some(background);
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }

    /// Set text color
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

// Placeholder for future style! macro implementation
// This will be expanded to support CSS-like syntax:
//
// ```rust
// let style = style! {
//     background: tokens.surface_elevated;
//     padding: tokens.space_md;
//     border_radius: tokens.radius_lg;
//     color: tokens.text_primary;
// };
// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_uniform() {
        let padding = Padding::uniform(16.0);
        assert_eq!(padding.top, 16.0);
        assert_eq!(padding.right, 16.0);
        assert_eq!(padding.bottom, 16.0);
        assert_eq!(padding.left, 16.0);
    }

    #[test]
    fn test_padding_symmetric() {
        let padding = Padding::symmetric(8.0, 16.0);
        assert_eq!(padding.top, 8.0);
        assert_eq!(padding.right, 16.0);
        assert_eq!(padding.bottom, 8.0);
        assert_eq!(padding.left, 16.0);
    }

    #[test]
    fn test_style_builder() {
        let style = Style::new()
            .background(TokenValue::Color(Color::new(1.0, 1.0, 1.0, 1.0)))
            .padding(Padding::uniform(16.0))
            .border_radius(8.0);

        assert!(style.background.is_some());
        assert!(style.padding.is_some());
        assert_eq!(style.border_radius, Some(8.0));
    }
}
