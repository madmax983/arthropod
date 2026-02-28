use arthropod::figma_codegen::{FigmaCodegenOptions, generate_rust_module_from_json};

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
