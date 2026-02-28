pub mod figma_pipeline_generated {

use arthropod::figma::{FigmaImportError, ImportedFigmaDocument, import_figma_document};
use arthropod::figma_runtime::{FigmaRuntime, FigmaRuntimeError};

pub const SOURCE_BYTES: usize = 1122;
pub const SOURCE_FNV64: u64 = 0xc1878be6ff904914;
pub const NODE_COUNT: usize = 3;
pub const PROTOTYPE_EDGE_COUNT: usize = 3;

pub const FIGMA_JSON: &str = r#"{
  "nodes": [
    {
      "id": "screen-a",
      "type": "FRAME",
      "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 200 },
      "fills": [{ "type": "SOLID", "color": [0.2, 0.3, 0.4, 1.0] }],
      "prototypeInteractions": [
        {
          "trigger": "ON_CLICK",
          "actions": [{ "type": "NAVIGATE", "destinationId": "screen-b" }]
        }
      ]
    },
    {
      "id": "screen-b",
      "type": "FRAME",
      "absoluteBoundingBox": { "x": 320, "y": 0, "width": 300, "height": 200 },
      "fills": [{ "type": "SOLID", "color": [0.3, 0.5, 0.7, 1.0] }],
      "prototypeInteractions": [
        {
          "trigger": "ON_CLICK",
          "actions": [{ "type": "OPEN_OVERLAY", "destinationId": "overlay" }]
        }
      ]
    },
    {
      "id": "overlay",
      "type": "FRAME",
      "absoluteBoundingBox": { "x": 360, "y": 40, "width": 220, "height": 120 },
      "fills": [{ "type": "SOLID", "color": [0.8, 0.8, 0.9, 1.0] }],
      "prototypeInteractions": [
        {
          "trigger": "ON_CLICK",
          "actions": [{ "type": "BACK" }]
        }
      ]
    }
  ]
}
"#;

pub fn document() -> Result<ImportedFigmaDocument, FigmaImportError> {
import_figma_document(FIGMA_JSON)
}

pub fn runtime() -> Result<FigmaRuntime, FigmaRuntimeError> {
FigmaRuntime::from_figma_json(FIGMA_JSON)
}
}
