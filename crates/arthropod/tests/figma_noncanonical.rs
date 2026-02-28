use arthropod::figma_codegen::{FigmaCodegenOptions, generate_rust_module_from_json};
use arthropod::figma_runtime::FigmaRuntime;

#[test]
fn codegen_accepts_enum_wrapped_values() {
    let non_canonical = r#"{
  "nodes": [
    {
      "id": "root",
      "type": { "__enum__": "NodeType", "value": "FRAME" },
      "x": 0,
      "y": 0,
      "width": 120,
      "height": 80,
      "fills": [
        {
          "type": { "__enum__": "PaintType", "value": "SOLID" },
          "color": { "r": 0.4, "g": 0.2, "b": 0.9, "a": 1.0 }
        }
      ]
    }
  ]
}"#;
    let options = FigmaCodegenOptions {
        module_name: "noncanonical_generated".to_string(),
        document_fn: "document".to_string(),
        runtime_fn: "runtime".to_string(),
    };

    let generated = generate_rust_module_from_json(non_canonical, &options)
        .expect("enum-wrapped payload should be accepted");
    assert!(generated.contains("pub mod noncanonical_generated"));
}

#[test]
fn codegen_tolerates_unrecognized_path_geometry_and_image_payloads() {
    let non_canonical = r#"{
  "nodes": [
    {
      "id": "shape",
      "type": "FRAME",
      "x": 0,
      "y": 0,
      "width": 200,
      "height": 120,
      "fillGeometry": [
        {
          "commands": ["M", 0, 0, "L", 200, 0, "Z"],
          "styleID": 0,
          "windingRule": { "__enum__": "WindingRule", "value": "NONZERO" }
        }
      ],
      "fills": [
        {
          "type": "IMAGE",
          "image": { "hash": [1, 2, 3], "name": "preview" },
          "imageScaleMode": { "__enum__": "ImageScaleMode", "value": "STRETCH" }
        }
      ]
    }
  ]
}"#;
    let options = FigmaCodegenOptions {
        module_name: "noncanonical_shapes".to_string(),
        document_fn: "document".to_string(),
        runtime_fn: "runtime".to_string(),
    };

    let generated = generate_rust_module_from_json(non_canonical, &options)
        .expect("unsupported geometry/image payloads should be skipped without parse failure");
    assert!(generated.contains("pub mod noncanonical_shapes"));
}

#[test]
fn runtime_renders_fill_paints_with_transform_and_size_bounds() {
    let non_canonical = r#"{
  "nodes": [
    {
      "id": "hero",
      "type": { "__enum__": "NodeType", "value": "FRAME" },
      "size": { "x": 320, "y": 180 },
      "transform": { "m00": 1, "m01": 0, "m02": 24, "m10": 0, "m11": 1, "m12": 32 },
      "fillPaints": [
        {
          "type": { "__enum__": "PaintType", "value": "SOLID" },
          "color": { "r": 0.2, "g": 0.6, "b": 0.9, "a": 1.0 }
        }
      ]
    }
  ]
}"#;
    let mut runtime = FigmaRuntime::from_figma_json(non_canonical)
        .expect("runtime should parse non-canonical fillPaints payload");
    runtime.apply_layout(1280.0, 720.0);

    let instances = runtime.collect_render_instances();
    assert!(
        !instances.is_empty(),
        "non-canonical fillPaints + transform/size payload should produce render instances"
    );
    assert!(
        instances
            .iter()
            .any(|instance| instance.size[0] > 100.0 && instance.size[1] > 100.0),
        "expected a visible large instance from size/transform-derived bounds"
    );
}

#[test]
fn codegen_accepts_code_snapshot_image_paints() {
    let non_canonical = r#"{
  "nodes": [
    {
      "id": "code-node",
      "type": { "__enum__": "NodeType", "value": "CODE_INSTANCE" },
      "size": { "x": 320, "y": 200 },
      "transform": { "m00": 1, "m01": 0, "m02": 10, "m10": 0, "m11": 1, "m12": 20 },
      "codeSnapshot": {
        "paints": [
          {
            "type": { "__enum__": "PaintType", "value": "IMAGE" },
            "image": { "hash": [239, 155, 77, 97], "name": "preview" },
            "imageScaleMode": { "__enum__": "ImageScaleMode", "value": "STRETCH" }
          }
        ]
      }
    }
  ]
}"#;
    let options = FigmaCodegenOptions {
        module_name: "code_snapshot_generated".to_string(),
        document_fn: "document".to_string(),
        runtime_fn: "runtime".to_string(),
    };

    let generated = generate_rust_module_from_json(non_canonical, &options)
        .expect("codeSnapshot image paints should be accepted in codegen import path");
    assert!(generated.contains("pub mod code_snapshot_generated"));
}
