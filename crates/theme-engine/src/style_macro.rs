//! Layer 3: Style API - CSS-like component styling
//!
//! Provides a declarative API for styling components using design tokens.
//! Includes the `style!` macro for CSS-like syntax with pseudo-state support.
//!
//! # Example
//!
//! ```ignore
//! use theme_engine::{style, DesignTokens, SystemTheme};
//!
//! let theme = SystemTheme::query()?;
//! let tokens = DesignTokens::from_system(&theme);
//!
//! let button_style = style! {
//!     background: tokens.surface_primary.clone();
//!     padding: tokens.space_md;
//!     border_radius: tokens.radius_lg;
//!     color: tokens.text_primary;
//!     opacity: 1.0;
//!
//!     &:hover {
//!         background: tokens.surface_secondary.clone();
//!         opacity: 0.9;
//!     }
//!
//!     &:focus {
//!         border_radius: tokens.radius_xl;
//!     }
//!
//!     &:disabled {
//!         opacity: 0.5;
//!     }
//! };
//!
//! // Resolve style for current widget state
//! let resolved = button_style.resolve(hover, focus, active, disabled);
//! ```

use crate::{Color, TokenValue};

/// Style properties for UI components with pseudo-state support
///
/// Supports CSS-like pseudo-states:
/// - `:hover` - Mouse over the element
/// - `:focus` - Element has keyboard focus
/// - `:active` - Element is being pressed
/// - `:disabled` - Element is disabled (overrides other states)
#[derive(Debug, Clone, Default)]
pub struct Style {
    // Base properties
    /// Background color or material
    pub background: Option<TokenValue>,
    /// Padding around content
    pub padding: Option<Padding>,
    /// Border radius for rounded corners
    pub border_radius: Option<f32>,
    /// Text/foreground color
    pub color: Option<Color>,
    /// Opacity (0.0 - 1.0)
    pub opacity: Option<f32>,

    // Pseudo-state styles
    /// Styles applied on hover
    pub hover: Option<Box<StyleOverrides>>,
    /// Styles applied on focus
    pub focus: Option<Box<StyleOverrides>>,
    /// Styles applied when active (pressed)
    pub active: Option<Box<StyleOverrides>>,
    /// Styles applied when disabled (overrides other states)
    pub disabled: Option<Box<StyleOverrides>>,
}

/// Partial style overrides for pseudo-states
///
/// Only properties that are `Some` will override the base style.
#[derive(Debug, Clone, Default)]
pub struct StyleOverrides {
    /// Override background color or material
    pub background: Option<TokenValue>,
    /// Override text/foreground color
    pub color: Option<Color>,
    /// Override opacity
    pub opacity: Option<f32>,
    /// Override border radius
    pub border_radius: Option<f32>,
}

/// Fully resolved style values after applying pseudo-states
///
/// All optional values are resolved to concrete values.
#[derive(Debug, Clone)]
pub struct ResolvedStyle {
    /// Resolved background
    pub background: Option<TokenValue>,
    /// Resolved text color
    pub color: Option<Color>,
    /// Resolved padding
    pub padding: Option<Padding>,
    /// Resolved border radius
    pub border_radius: Option<f32>,
    /// Resolved opacity (defaults to 1.0)
    pub opacity: f32,
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

impl StyleOverrides {
    /// Apply overrides to a resolved style
    fn apply_to(&self, resolved: &mut ResolvedStyle) {
        if let Some(bg) = &self.background {
            resolved.background = Some(bg.clone());
        }
        if let Some(c) = self.color {
            resolved.color = Some(c);
        }
        if let Some(o) = self.opacity {
            resolved.opacity = o;
        }
        if let Some(r) = self.border_radius {
            resolved.border_radius = Some(r);
        }
    }
}

impl Style {
    /// Create a new empty style
    pub fn new() -> Self {
        Self::default()
    }

    /// Set background (builder pattern)
    pub fn background(mut self, background: TokenValue) -> Self {
        self.background = Some(background);
        self
    }

    /// Set padding (builder pattern)
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set border radius (builder pattern)
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }

    /// Set text color (builder pattern)
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set opacity (builder pattern)
    pub fn set_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    /// Resolve style for current widget state
    ///
    /// Applies pseudo-state overrides in order:
    /// 1. If disabled, only disabled overrides apply (ignores hover/focus/active)
    /// 2. Otherwise, hover, focus, and active all apply (last write wins for overlapping properties)
    ///
    /// # Arguments
    ///
    /// * `hover` - Mouse is over the element
    /// * `focus` - Element has keyboard focus
    /// * `active` - Element is being pressed
    /// * `disabled` - Element is disabled
    ///
    /// # Returns
    ///
    /// A `ResolvedStyle` with all properties resolved to concrete values.
    pub fn resolve(&self, hover: bool, focus: bool, active: bool, disabled: bool) -> ResolvedStyle {
        let mut resolved = ResolvedStyle {
            background: self.background.clone(),
            color: self.color,
            padding: self.padding,
            border_radius: self.border_radius,
            opacity: self.opacity.unwrap_or(1.0),
        };

        if disabled {
            // Disabled state overrides everything else
            if let Some(d) = &self.disabled {
                d.apply_to(&mut resolved);
            }
        } else {
            // Apply states in order (later states override earlier for same properties)
            if hover {
                if let Some(h) = &self.hover {
                    h.apply_to(&mut resolved);
                }
            }
            if focus {
                if let Some(f) = &self.focus {
                    f.apply_to(&mut resolved);
                }
            }
            if active {
                if let Some(a) = &self.active {
                    a.apply_to(&mut resolved);
                }
            }
        }

        resolved
    }
}

// ============================================================================
// From implementations for TokenValue
// ============================================================================

impl From<Color> for TokenValue {
    fn from(color: Color) -> Self {
        TokenValue::Color(color)
    }
}

// ============================================================================
// style! macro
// ============================================================================

/// CSS-like style macro for declarative UI styling
///
/// Supports:
/// - Base properties: `background`, `color`, `padding`, `border_radius`, `opacity`
/// - Pseudo-states: `&:hover`, `&:focus`, `&:active`, `&:disabled`
///
/// # Example
///
/// ```ignore
/// let s = style! {
///     background: tokens.surface_primary.clone();
///     color: tokens.text_primary;
///     padding: tokens.space_md;
///     opacity: 1.0;
///
///     &:hover {
///         background: tokens.surface_secondary.clone();
///         opacity: 0.9;
///     }
///
///     &:disabled {
///         opacity: 0.5;
///     }
/// };
/// ```
#[macro_export]
macro_rules! style {
    ( $($body:tt)* ) => {{
        let mut style = $crate::Style::new();
        $crate::__style_impl!(style, $($body)*);
        style
    }};
}

/// Internal macro for processing style body
#[macro_export]
#[doc(hidden)]
macro_rules! __style_impl {
    // Base case: empty
    ($style:ident,) => {};
    ($style:ident) => {};

    // ========================================================================
    // Pseudo-states
    // ========================================================================

    // &:hover { ... }
    ($style:ident, &:hover { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.hover = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };

    // &:focus { ... }
    ($style:ident, &:focus { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.focus = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };

    // &:active { ... }
    ($style:ident, &:active { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.active = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };

    // &:disabled { ... }
    ($style:ident, &:disabled { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.disabled = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };

    // ========================================================================
    // Base properties
    // ========================================================================

    // background: expr;
    ($style:ident, background: $val:expr; $($rest:tt)*) => {
        $style.background = Some($crate::TokenValue::from($val));
        $crate::__style_impl!($style, $($rest)*);
    };

    // color: expr;
    ($style:ident, color: $val:expr; $($rest:tt)*) => {
        $style.color = Some($val);
        $crate::__style_impl!($style, $($rest)*);
    };

    // padding: expr;
    ($style:ident, padding: $val:expr; $($rest:tt)*) => {
        $style.padding = Some($crate::Padding::uniform($val));
        $crate::__style_impl!($style, $($rest)*);
    };

    // border_radius: expr;
    ($style:ident, border_radius: $val:expr; $($rest:tt)*) => {
        $style.border_radius = Some($val);
        $crate::__style_impl!($style, $($rest)*);
    };

    // opacity: expr;
    ($style:ident, opacity: $val:expr; $($rest:tt)*) => {
        $style.opacity = Some($val);
        $crate::__style_impl!($style, $($rest)*);
    };
}

/// Internal macro for processing style overrides (pseudo-state bodies)
#[macro_export]
#[doc(hidden)]
macro_rules! __style_overrides {
    // Base case: empty
    ($overrides:ident,) => {};
    ($overrides:ident) => {};

    // background: expr;
    ($overrides:ident, background: $val:expr; $($rest:tt)*) => {
        $overrides.background = Some($crate::TokenValue::from($val));
        $crate::__style_overrides!($overrides, $($rest)*);
    };

    // color: expr;
    ($overrides:ident, color: $val:expr; $($rest:tt)*) => {
        $overrides.color = Some($val);
        $crate::__style_overrides!($overrides, $($rest)*);
    };

    // opacity: expr;
    ($overrides:ident, opacity: $val:expr; $($rest:tt)*) => {
        $overrides.opacity = Some($val);
        $crate::__style_overrides!($overrides, $($rest)*);
    };

    // border_radius: expr;
    ($overrides:ident, border_radius: $val:expr; $($rest:tt)*) => {
        $overrides.border_radius = Some($val);
        $crate::__style_overrides!($overrides, $($rest)*);
    };
}

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

    #[test]
    fn test_style_default() {
        let style = Style::default();
        assert!(style.background.is_none());
        assert!(style.padding.is_none());
        assert!(style.border_radius.is_none());
        assert!(style.color.is_none());
        assert!(style.opacity.is_none());
        assert!(style.hover.is_none());
        assert!(style.focus.is_none());
        assert!(style.active.is_none());
        assert!(style.disabled.is_none());
    }

    #[test]
    fn test_style_overrides_default() {
        let overrides = StyleOverrides::default();
        assert!(overrides.background.is_none());
        assert!(overrides.color.is_none());
        assert!(overrides.opacity.is_none());
        assert!(overrides.border_radius.is_none());
    }

    #[test]
    fn test_resolve_base_only() {
        let style = Style {
            background: Some(TokenValue::Color(Color::new(1.0, 0.0, 0.0, 1.0))),
            opacity: Some(0.8),
            border_radius: Some(4.0),
            ..Default::default()
        };

        let resolved = style.resolve(false, false, false, false);
        assert!(resolved.background.is_some());
        assert_eq!(resolved.opacity, 0.8);
        assert_eq!(resolved.border_radius, Some(4.0));
    }

    #[test]
    fn test_resolve_hover() {
        let style = Style {
            opacity: Some(1.0),
            hover: Some(Box::new(StyleOverrides {
                opacity: Some(0.9),
                ..Default::default()
            })),
            ..Default::default()
        };

        let resolved = style.resolve(true, false, false, false);
        assert_eq!(resolved.opacity, 0.9);
    }

    #[test]
    fn test_resolve_disabled_overrides_hover() {
        let style = Style {
            opacity: Some(1.0),
            hover: Some(Box::new(StyleOverrides {
                opacity: Some(0.9),
                ..Default::default()
            })),
            disabled: Some(Box::new(StyleOverrides {
                opacity: Some(0.5),
                ..Default::default()
            })),
            ..Default::default()
        };

        // With both hover and disabled, disabled takes precedence
        let resolved = style.resolve(true, false, false, true);
        assert_eq!(resolved.opacity, 0.5);
    }

    #[test]
    fn test_token_value_from_color() {
        let color = Color::new(1.0, 0.0, 0.0, 1.0);
        let token: TokenValue = color.into();
        assert!(matches!(token, TokenValue::Color(_)));
    }

    #[test]
    fn test_token_value_identity() {
        // Test that cloned TokenValue equals original (basic identity check)
        let original = TokenValue::Color(Color::new(1.0, 0.0, 0.0, 1.0));
        let cloned = original.clone();
        assert_eq!(cloned, original);
    }
}
