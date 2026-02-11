//! Style Engine for Arthropod
//!
//! This crate provides unified styling primitives for the Figma rendering pipeline,
//! including corner radii, blend modes, paints, strokes, effects, and text styling.

pub mod blend;
pub mod corner;
pub mod effect;
pub mod paint;
pub mod path;
pub mod stroke;
pub mod text;
pub mod visual;

pub use blend::BlendMode;
pub use corner::CornerRadii;
pub use effect::{BackgroundBlur, DropShadow, Effect, InnerShadow, LayerBlur};
pub use paint::{
    AngularGradient, Color, ColorStop, DiamondGradient, GradientInterpolationMode, ImageFill,
    ImageId, ImageScaleMode, LinearGradient, Paint, RadialGradient,
};
pub use path::{BooleanOp, PathCommand, SvgParseOptions, VectorPath, VectorPathError, WindingRule};
pub use stroke::{SideWeights, StrokeAlign, StrokeCap, StrokeJoin, StrokeStyle};
pub use text::{FontStyle, LineHeight, TextAlign, TextContent, TextDecoration};
pub use visual::VisualStyle;
