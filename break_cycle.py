import re

with open("crates/arthropod/src/figma/models.rs", "r") as f:
    models = f.read()

# Extract FigmaImportError and import_figma_document from models.rs
import_figma_idx = models.find("pub fn import_figma_document")
if import_figma_idx != -1:
    import_figma_end = models.find("fn parse_figma_document") # It doesn't exist here
    # Actually wait, import_figma_document goes to end of file

    models_rest = models[import_figma_idx:]
    models = models[:import_figma_idx]

# Extract FigmaImportError
error_start = models.find("#[derive(Debug, Error)]\npub enum FigmaImportError")
if error_start != -1:
    error_end = models.find("}\n", error_start) + 2
    error_str = models[error_start:error_end]
    models = models[:error_start] + models[error_end:]

with open("crates/arthropod/src/figma/models.rs", "w") as f:
    f.write(models)

with open("crates/arthropod/src/figma/mod.rs", "w") as f:
    f.write("""pub mod models;
pub(crate) mod schema;

pub use models::*;

use std::collections::HashMap;
use render_engine::{NodeContent, NodeId, Scene, SceneNode, Transform2D, Vec2, Vec4};
use style_engine::{
    BackgroundBlur, BlendMode, ColorFilter, ColorStop, CornerRadii, DropShadow, Effect, FontStyle,
    ImageFill, ImageId, ImageScaleMode, InnerShadow, LayerBlur, LineHeight, LinearGradient,
    MaskType, Paint, RadialGradient, SideWeights, StrokeAlign, StrokeCap, StrokeJoin, StrokeStyle,
    TextAlign, TextAlignVertical, TextAutoResize, TextCase, TextContent, TextDecoration,
    TextOverflow, VectorPath, VisualStyle, WindingRule,
};
use layout_engine::{
    FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle, FlexWrap, ItemAlignSelf,
};
use plat_core::Rect;
use thiserror::Error;

""")
    f.write(error_str)
    f.write("\n")
    f.write(models_rest)

# Clean up use super::schema::* from models.rs
models = models.replace("use super::schema::*;\n", "")
with open("crates/arthropod/src/figma/models.rs", "w") as f:
    f.write(models)

# Update schema.rs to use super::* instead of use super::models::* so it gets error
with open("crates/arthropod/src/figma/schema.rs", "r") as f:
    schema = f.read()

schema = schema.replace("use super::models::*;", "use super::*;")
with open("crates/arthropod/src/figma/schema.rs", "w") as f:
    f.write(schema)
