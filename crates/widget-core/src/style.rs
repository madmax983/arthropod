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

use layout_engine::{FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle, FlexWrap};
use style_engine::{CornerRadii, Paint, StrokeStyle, VisualStyle};
use theme_engine::{Color, TokenValue};

/// Style properties for UI components with pseudo-state support
///
/// Supports CSS-like pseudo-states:
/// - `:hover` - Mouse over the element
/// - `:focus` - Element has keyboard focus
/// - `:active` - Element is being pressed
/// - `:disabled` - Element is disabled (overrides other states)
#[derive(Debug, Clone, Default)]
pub struct Style {
    // === Visual Properties ===
    /// Background color or material
    pub background: Option<TokenValue>,
    /// Border radius for rounded corners
    pub border_radius: Option<CornerRadii>,
    /// Text/foreground color
    pub color: Option<Color>,
    /// Opacity (0.0 - 1.0)
    pub opacity: Option<f32>,
    /// Corner smoothing (0.0 - 1.0)
    pub corner_smoothing: Option<f32>,
    /// Stroke style
    pub stroke: Option<StrokeStyle>,
    /// Shadow effects
    pub effects: Vec<style_engine::Effect>,

    // === Layout Properties ===
    /// Direction of the main axis
    pub direction: Option<FlexDirection>,
    /// Main-axis distribution
    pub justify_content: Option<FlexJustifyContent>,
    /// Cross-axis alignment
    pub align_items: Option<FlexAlign>,
    /// Padding around content
    pub padding: Option<Padding>,
    /// Gap between children
    pub gap: Option<f32>,
    /// Fixed width
    pub width: Option<f32>,
    /// Fixed height
    pub height: Option<f32>,
    /// Flex grow factor
    pub flex_grow: Option<f32>,
    /// Flex shrink factor
    pub flex_shrink: Option<f32>,
    /// Flex wrap
    pub wrap: Option<FlexWrap>,

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
    pub border_radius: Option<CornerRadii>,
    /// Override stroke
    pub stroke: Option<StrokeStyle>,
}

/// Fully resolved style values after applying pseudo-states
///
/// All optional values are resolved to concrete values.
#[derive(Debug, Clone)]
pub struct ResolvedStyle {
    // Visual
    /// Resolved background color or material
    pub background: Option<TokenValue>,
    /// Resolved text/foreground color
    pub color: Option<Color>,
    /// Resolved border radius
    pub border_radius: CornerRadii,
    /// Resolved opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Resolved corner smoothing (0.0 - 1.0)
    pub corner_smoothing: f32,
    /// Resolved stroke style
    pub stroke: Option<StrokeStyle>,
    /// Resolved shadow and visual effects
    pub effects: Vec<style_engine::Effect>,

    // Layout
    /// Resolved direction of the main axis
    pub direction: FlexDirection,
    /// Resolved main-axis distribution
    pub justify_content: FlexJustifyContent,
    /// Resolved cross-axis alignment
    pub align_items: FlexAlign,
    /// Resolved padding around content
    pub padding: Padding,
    /// Resolved gap between children
    pub gap: f32,
    /// Resolved fixed width
    pub width: Option<f32>,
    /// Resolved fixed height
    pub height: Option<f32>,
    /// Resolved flex grow factor
    pub flex_grow: f32,
    /// Resolved flex shrink factor
    pub flex_shrink: f32,
    /// Resolved flex wrap mode
    pub wrap: FlexWrap,
}

/// Padding values (top, right, bottom, left)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Padding {
    /// Top padding
    pub top: f32,
    /// Right padding
    pub right: f32,
    /// Bottom padding
    pub bottom: f32,
    /// Left padding
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

impl From<f32> for Padding {
    fn from(value: f32) -> Self {
        Self::uniform(value)
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
            resolved.border_radius = r;
        }
        if let Some(s) = &self.stroke {
            resolved.stroke = Some(s.clone());
        }
    }
}

impl Style {
    /// Create a new empty style
    pub fn new() -> Self {
        Self::default()
    }

    // === Visual Builders ===

    /// Set the background color or material.
    pub fn background(mut self, background: impl Into<TokenValue>) -> Self {
        self.background = Some(background.into());
        self
    }

    /// Set the border radius for rounded corners.
    pub fn border_radius(mut self, radius: impl Into<CornerRadii>) -> Self {
        self.border_radius = Some(radius.into());
        self
    }

    /// Set the text/foreground color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the opacity level (0.0 to 1.0).
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    /// Set the corner smoothing factor (iOS style "squircle" effect, 0.0 to 1.0).
    pub fn corner_smoothing(mut self, smoothing: f32) -> Self {
        self.corner_smoothing = Some(smoothing);
        self
    }

    /// Set the stroke (outline/border) style.
    pub fn stroke(mut self, stroke: StrokeStyle) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Add a visual effect (like a drop shadow or inner shadow) to the style.
    pub fn effect(mut self, effect: style_engine::Effect) -> Self {
        self.effects.push(effect);
        self
    }

    // === Layout Builders ===

    /// Set the primary flex layout direction (e.g. `Column` or `Row`).
    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Set how items are distributed along the main axis.
    pub fn justify_content(mut self, justify: FlexJustifyContent) -> Self {
        self.justify_content = Some(justify);
        self
    }

    /// Set how items are aligned along the cross axis.
    pub fn align_items(mut self, align: FlexAlign) -> Self {
        self.align_items = Some(align);
        self
    }

    /// Set the padding around the inner content.
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set the spacing gap between flex items.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Set an explicit, fixed width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set an explicit, fixed height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set the flex grow factor (how much remaining space to consume).
    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.flex_grow = Some(grow);
        self
    }

    /// Set the flex shrink factor (how much to shrink when space is constrained).
    pub fn flex_shrink(mut self, shrink: f32) -> Self {
        self.flex_shrink = Some(shrink);
        self
    }

    /// Set the flex wrap behavior (whether children can wrap to new lines).
    pub fn wrap(mut self, wrap: FlexWrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Resolve style for current widget state
    pub fn resolve(&self, hover: bool, focus: bool, active: bool, disabled: bool) -> ResolvedStyle {
        let mut resolved = ResolvedStyle {
            background: self.background.clone(),
            color: self.color,
            border_radius: self.border_radius.unwrap_or(CornerRadii::ZERO),
            opacity: self.opacity.unwrap_or(1.0),
            corner_smoothing: self.corner_smoothing.unwrap_or(0.0),
            stroke: self.stroke.clone(),
            effects: self.effects.clone(),

            direction: self.direction.unwrap_or(FlexDirection::Column),
            justify_content: self.justify_content.unwrap_or(FlexJustifyContent::Start),
            align_items: self.align_items.unwrap_or(FlexAlign::Stretch),
            padding: self.padding.unwrap_or_default(),
            gap: self.gap.unwrap_or(0.0),
            width: self.width,
            height: self.height,
            flex_grow: self.flex_grow.unwrap_or(0.0),
            flex_shrink: self.flex_shrink.unwrap_or(1.0),
            wrap: self.wrap.unwrap_or(FlexWrap::NoWrap),
        };

        if disabled {
            if let Some(d) = &self.disabled {
                d.apply_to(&mut resolved);
            }
        } else {
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

impl ResolvedStyle {
    /// Convert to style-engine VisualStyle
    pub fn to_visual_style(&self) -> VisualStyle {
        let mut style = VisualStyle::new();

        if let Some(bg) = &self.background {
            style.fills.push(Paint::Solid(bg.as_color()));
        }

        style.corner_radii = self.border_radius;
        style.corner_smoothing = self.corner_smoothing;
        style.opacity = self.opacity;
        style.stroke = self.stroke.clone();
        style.effects = self.effects.clone();

        style
    }

    /// Convert to layout-engine FlexStyle
    pub fn to_flex_style(&self) -> FlexStyle {
        FlexStyle {
            direction: self.direction,
            justify_content: self.justify_content,
            align_items: self.align_items,
            padding_left: self.padding.left,
            padding_right: self.padding.right,
            padding_top: self.padding.top,
            padding_bottom: self.padding.bottom,
            gap: self.gap,
            width: self.width,
            height: self.height,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
            wrap: self.wrap,
            ..Default::default()
        }
    }
}

// ============================================================================
// style! macro
// ============================================================================

/// A macro for creating declarative, CSS-like component styles.
///
/// Supports visual properties, layout properties, and pseudo-states (`&:hover`,
/// `&:focus`, `&:active`, `&:disabled`).
///
/// # Example
///
/// ```
/// use widget_core::{style, Style};
/// use theme_engine::Color;
/// use style_engine::CornerRadii;
///
/// let my_style: Style = style! {
///     background: Color::new(1.0, 1.0, 1.0, 1.0);
///     color: Color::new(0.0, 0.0, 0.0, 1.0);
///     border_radius: 8.0;
///     padding: 16.0;
///
///     &:hover {
///         background: Color::new(0.8, 0.8, 0.8, 1.0);
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

#[macro_export]
#[doc(hidden)]
macro_rules! __style_impl {
    ($style:ident,) => {};
    ($style:ident) => {};

    // Pseudo-states
    ($style:ident, &:hover { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.hover = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, &:focus { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.focus = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, &:active { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.active = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, &:disabled { $($inner:tt)* } $($rest:tt)*) => {
        {
            let mut overrides = $crate::StyleOverrides::default();
            $crate::__style_overrides!(overrides, $($inner)*);
            $style.disabled = Some(Box::new(overrides));
        }
        $crate::__style_impl!($style, $($rest)*);
    };

    // Visual Properties
    ($style:ident, background: $val:expr; $($rest:tt)*) => {
        $style = $style.background($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, color: $val:expr; $($rest:tt)*) => {
        $style = $style.color($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, border_radius: $val:expr; $($rest:tt)*) => {
        $style = $style.border_radius($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, opacity: $val:expr; $($rest:tt)*) => {
        $style = $style.opacity($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, corner_smoothing: $val:expr; $($rest:tt)*) => {
        $style = $style.corner_smoothing($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, stroke: $val:expr; $($rest:tt)*) => {
        $style = $style.stroke($val);
        $crate::__style_impl!($style, $($rest)*);
    };

    // Layout Properties
    ($style:ident, direction: $val:expr; $($rest:tt)*) => {
        $style = $style.direction($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, justify_content: $val:expr; $($rest:tt)*) => {
        $style = $style.justify_content($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, align_items: $val:expr; $($rest:tt)*) => {
        $style = $style.align_items($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, padding: $val:expr; $($rest:tt)*) => {
        $style = $style.padding($val.into());
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, gap: $val:expr; $($rest:tt)*) => {
        $style = $style.gap($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, width: $val:expr; $($rest:tt)*) => {
        $style = $style.width($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, height: $val:expr; $($rest:tt)*) => {
        $style = $style.height($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, flex_grow: $val:expr; $($rest:tt)*) => {
        $style = $style.flex_grow($val);
        $crate::__style_impl!($style, $($rest)*);
    };
    ($style:ident, flex_shrink: $val:expr; $($rest:tt)*) => {
        $style = $style.flex_shrink($val);
        $crate::__style_impl!($style, $($rest)*);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __style_overrides {
    ($overrides:ident,) => {};
    ($overrides:ident) => {};

    ($overrides:ident, background: $val:expr; $($rest:tt)*) => {
        $overrides.background = Some($crate::TokenValue::from($val));
        $crate::__style_overrides!($overrides, $($rest)*);
    };
    ($overrides:ident, color: $val:expr; $($rest:tt)*) => {
        $overrides.color = Some($val);
        $crate::__style_overrides!($overrides, $($rest)*);
    };
    ($overrides:ident, opacity: $val:expr; $($rest:tt)*) => {
        $overrides.opacity = Some($val);
        $crate::__style_overrides!($overrides, $($rest)*);
    };
    ($overrides:ident, border_radius: $val:expr; $($rest:tt)*) => {
        $overrides.border_radius = Some($val.into());
        $crate::__style_overrides!($overrides, $($rest)*);
    };
}
