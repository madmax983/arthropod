import os
import re

with open("crates/arthropod/src/figma/models.rs", "r") as f:
    models = f.read()

with open("crates/arthropod/src/figma/schema.rs", "r") as f:
    schema = f.read()

# Instead of blindly doing use super::schema::*, let's only import what we need or restructure.
# Wait, models depends on schema to parse and map, and schema depends on models to map to them.
# The cleaner way is to make `schema` map to `models` and `models` import `schema` just to run `parse_figma_document`.
# Actually, if `schema` depends on `models`, `models` should NOT depend on `schema`.
# But `models.rs` currently contains `import_figma_document` which calls `parse_figma_document` which is in `schema.rs`.
# So `models` uses `schema`.
# Does `schema` use `models`? Yes, because it maps to types in `models.rs` like `VisualStyle`, `TextContent`, `FigmaImportError` which is in `models.rs`.
# This is indeed a circular dependency!
# We must break it.
# `import_figma_document` and `FigmaImportError` should be in the root `mod.rs`!
# `models.rs` should ONLY contain the pure data models.
# `schema.rs` should map from raw JSON to `models.rs`.
# Then `mod.rs` glues them together by defining `import_figma_document` and `FigmaImportError`.
