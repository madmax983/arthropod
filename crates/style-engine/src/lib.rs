//! Style Engine for Arthropod
//!
//! This crate provides unified styling primitives for the Figma rendering pipeline,
//! including corner radii, blend modes, paints, strokes, effects, and text styling.

/// Defines how colors from an upper layer blend into the background.
/// These map directly to Figma/W3C standard blend modes (e.g., Multiply, Overlay, Screen).
pub mod blend;
/// Defines independent border radii for the four corners of a rectangle.
/// Used to create pills, circles, or asymmetric rounded containers.
pub mod corner;
/// Defines visual post-processing effects applied to layers.
/// Includes Box Shadows (inner/outer) and Blur filters (layer/background).
pub mod effect;
/// Defines how geometry is filled.
/// Supports solid colors, procedural gradients (linear, radial, angular), and Image textures.
pub mod paint;
/// Defines arbitrary vector geometry.
/// Provides an SVG path parser, a programmatic builder API, and boolean geometry operations (Union/Subtract).
pub mod path;
/// Defines how the borders of geometry are drawn.
/// Controls line thickness, dash patterns, and how line segments join (Miter/Bevel/Round).
pub mod stroke;
/// Defines typographic styling.
/// Controls text alignment, decoration (underline/strikethrough), line height, and text truncation/overflow.
pub mod text;
/// The universal styling container.
/// A `VisualStyle` aggregates fills, strokes, effects, and text rules into a single portable payload.
pub mod visual;

pub use blend::BlendMode;
pub use corner::CornerRadii;
pub use effect::{BackgroundBlur, ColorFilter, DropShadow, Effect, InnerShadow, LayerBlur};
pub use paint::{
    AngularGradient, Color, ColorStop, DiamondGradient, GradientInterpolationMode, ImageFill,
    ImageId, ImageScaleMode, LinearGradient, Paint, RadialGradient,
};
pub use path::{BooleanOp, PathCommand, SvgParseOptions, VectorPath, VectorPathError, WindingRule};
pub use stroke::{SideWeights, StrokeAlign, StrokeCap, StrokeJoin, StrokeStyle};
pub use text::{
    FontStyle, LineHeight, TextAlign, TextAlignVertical, TextAutoResize, TextCase, TextContent,
    TextDecoration, TextOverflow,
};
pub use visual::{MaskType, VisualStyle};
