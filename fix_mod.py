import re

with open("crates/arthropod/src/figma/mod.rs", "r") as f:
    content = f.read()

# Instead of blindly rewriting mod.rs, let's keep it simple.
# The `schema` module creates the data model `FigmaDocument`, `models` module needs to be completely agnostic of schema.
# But `models.rs` has functions that depend on `schema`! No, `FigmaNode` etc are in `schema`.
# So `FigmaNode` methods like `to_node_transform`, `to_visual_style` are in `schema`.
# But they return `models` types. This is the correct unidirectional dependency:
# schema -> models.
# The entrypoint `import_figma_document` takes `json` string, calls `parse_figma_document` from `schema`, gets `FigmaDocument` from `schema`, and maps it to `ImportedFigmaDocument` from `models`.
# Where should `import_figma_document` live? In `mod.rs`!
# Let's fix the `import_figma_document` in `mod.rs` to use `schema::*`.

content = content.replace("use std::collections::HashMap;\n", "use std::collections::HashMap;\nuse schema::*;\n")

# Remove unused imports from mod.rs
unused = [
    'NodeId, Transform2D, Vec2, Vec4', 'BackgroundBlur, BlendMode, ColorFilter, ColorStop, CornerRadii, DropShadow, Effect, FontStyle,',
    'ImageFill, ImageId, ImageScaleMode, InnerShadow, LayerBlur, LineHeight, LinearGradient,',
    'MaskType, Paint, RadialGradient, SideWeights, StrokeAlign, StrokeCap, StrokeJoin, StrokeStyle,',
    'TextAlign, TextAlignVertical, TextAutoResize, TextCase, TextContent, TextDecoration,',
    'TextOverflow, VectorPath, VisualStyle, WindingRule,',
    'FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle, FlexWrap, ItemAlignSelf,',
    'use plat_core::Rect;\n'
]

for u in unused:
    content = content.replace(u, '')

content = content.replace('use render_engine::{NodeContent, Scene, SceneNode, };\n', 'use render_engine::{NodeContent, Scene, SceneNode};\n')
content = content.replace('use style_engine::{\n    \n    \n    \n    \n    \n};\n', '')
content = content.replace('use layout_engine::{\n    \n};\n', '')

with open("crates/arthropod/src/figma/mod.rs", "w") as f:
    f.write(content)

with open("crates/arthropod/src/figma/models.rs", "r") as f:
    models = f.read()

models = models.replace("use render_engine::{NodeContent, Scene, SceneNode};\n", "")
models = models.replace("use thiserror::Error;\n", "")

with open("crates/arthropod/src/figma/models.rs", "w") as f:
    f.write(models)
