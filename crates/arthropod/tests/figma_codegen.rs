use std::time::{SystemTime, UNIX_EPOCH};

use arthropod::figma_codegen::{
    FigmaCodegenOptions, generate_rust_module_from_json, write_rust_module_from_json,
};

fn sample_json() -> &'static str {
    r#"{
        "nodes": [
            {
                "id": "10",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 200 },
                "prototypeInteractions": [
                    {
                        "trigger": "ON_CLICK",
                        "actions": [{ "type": "NAVIGATE", "destinationId": "11" }]
                    }
                ]
            },
            {
                "id": "11",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 320, "y": 0, "width": 300, "height": 200 }
            }
        ]
    }"#
}

#[test]
fn figma_codegen_generates_runtime_and_document_entrypoints() {
    let options = FigmaCodegenOptions {
        module_name: "riotwaves_codegen".to_string(),
        document_fn: "document".to_string(),
        runtime_fn: "runtime".to_string(),
    };
    let generated = generate_rust_module_from_json(sample_json(), &options)
        .expect("codegen should succeed for valid figma json");

    assert!(generated.contains("pub mod riotwaves_codegen"));
    assert!(generated.contains("pub const NODE_COUNT: usize = 2;"));
    assert!(generated.contains("pub const PROTOTYPE_EDGE_COUNT: usize = 1;"));
    assert!(generated.contains("pub fn document() -> Result<ImportedFigmaDocument"));
    assert!(generated.contains("pub fn runtime() -> Result<FigmaRuntime"));
    assert!(generated.contains("\"id\": \"10\""));
}

#[test]
fn figma_codegen_is_deterministic_for_same_input() {
    let options = FigmaCodegenOptions::default();
    let first = generate_rust_module_from_json(sample_json(), &options).expect("first run");
    let second = generate_rust_module_from_json(sample_json(), &options).expect("second run");
    assert_eq!(first, second, "codegen output should be stable");
}

#[test]
fn figma_codegen_writes_module_file() {
    let options = FigmaCodegenOptions {
        module_name: "riotwaves_codegen_file".to_string(),
        document_fn: "doc_fn".to_string(),
        runtime_fn: "runtime_fn".to_string(),
    };
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let file_path = std::env::temp_dir().join(format!(
        "arthropod_figma_codegen_{}_{}.rs",
        std::process::id(),
        nonce
    ));

    write_rust_module_from_json(&file_path, sample_json(), &options)
        .expect("writing generated module should succeed");

    let written = std::fs::read_to_string(&file_path).expect("generated file should be readable");
    assert!(written.contains("pub mod riotwaves_codegen_file"));
    assert!(written.contains("pub fn doc_fn()"));
    assert!(written.contains("pub fn runtime_fn()"));

    std::fs::remove_file(&file_path).expect("temporary codegen file should be removable");
}
