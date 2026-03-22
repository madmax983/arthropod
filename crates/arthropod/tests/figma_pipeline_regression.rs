use std::path::{Path, PathBuf};

use arthropod::figma_codegen::{FigmaCodegenOptions, generate_rust_module_from_json};
use arthropod::figma_runtime::FigmaRuntime;
use arthropod::prototype_runtime::{PrototypeRuntimeEffect, PrototypeRuntimeEvent};

const FIXTURE_JSON_PATH: &str = "tests/fixtures/figma/figma_pipeline_fixture.json";
const CODEGEN_GOLDEN_PATH: &str = "tests/fixtures/figma/figma_pipeline_codegen.golden.rs";
const RUNTIME_GOLDEN_PATH: &str = "tests/fixtures/figma/figma_pipeline_runtime.golden.txt";

#[test]
fn figma_codegen_snapshot_matches_golden() {
    let fixture = load_text(&manifest_path(FIXTURE_JSON_PATH));
    let options = FigmaCodegenOptions {
        module_name: "figma_pipeline_generated".to_string(),
        document_fn: "document".to_string(),
        runtime_fn: "runtime".to_string(),
    };
    let generated = generate_rust_module_from_json(&fixture, &options)
        .expect("codegen should succeed for fixture json");

    assert_snapshot_matches(&manifest_path(CODEGEN_GOLDEN_PATH), &generated);
}

#[test]
fn figma_runtime_visibility_and_instances_match_golden() {
    let fixture = load_text(&manifest_path(FIXTURE_JSON_PATH));
    let mut runtime = FigmaRuntime::from_figma_json(&fixture).expect("runtime should initialize");
    let screen_a = runtime
        .node_for_figma_id("screen-a")
        .expect("screen-a should map");
    let screen_b = runtime
        .node_for_figma_id("screen-b")
        .expect("screen-b should map");
    let overlay = runtime
        .node_for_figma_id("overlay")
        .expect("overlay should map");

    runtime.set_current_screen(screen_a);

    let tracked = [
        ("screen-a", screen_a),
        ("screen-b", screen_b),
        ("overlay", overlay),
    ];
    let mut phases = Vec::new();
    phases.push(snapshot_phase("initial", &runtime, &tracked));

    let navigate = runtime.dispatch(PrototypeRuntimeEvent::Click { node: screen_a });
    assert!(navigate.iter().any(|effect| matches!(
        effect,
        PrototypeRuntimeEffect::Navigate { to, .. } if *to == screen_b
    )));
    phases.push(snapshot_phase("after_navigate", &runtime, &tracked));

    let open_overlay = runtime.dispatch(PrototypeRuntimeEvent::Click { node: screen_b });
    assert!(open_overlay.iter().any(|effect| matches!(
        effect,
        PrototypeRuntimeEffect::OpenOverlay { to, .. } if *to == overlay
    )));
    phases.push(snapshot_phase("after_open_overlay", &runtime, &tracked));

    let close_overlay = runtime.dispatch(PrototypeRuntimeEvent::Click { node: overlay });
    assert!(close_overlay.iter().any(|effect| matches!(
        effect,
        PrototypeRuntimeEffect::CloseOverlay { closed: Some(node) } if *node == overlay
    )));
    phases.push(snapshot_phase("after_overlay_back", &runtime, &tracked));

    let snapshot = phases.join("\n");
    assert_snapshot_matches(&manifest_path(RUNTIME_GOLDEN_PATH), &snapshot);
}

#[test]
fn figma_codegen_normalizes_crlf_and_lf_to_identical_output() {
    let fixture_raw = load_text(&manifest_path(FIXTURE_JSON_PATH));
    // Normalize to LF first so the CRLF conversion is clean on all platforms.
    // On Windows, load_text may return CRLF; a naive replace('\n', "\r\n")
    // would turn existing \r\n into \r\r\n, producing double-newlines after
    // normalize_line_endings strips the \r characters.
    let fixture = fixture_raw.replace("\r\n", "\n");
    let fixture_crlf = fixture.replace('\n', "\r\n");
    let options = FigmaCodegenOptions {
        module_name: "figma_pipeline_generated".to_string(),
        document_fn: "document".to_string(),
        runtime_fn: "runtime".to_string(),
    };

    let from_lf =
        generate_rust_module_from_json(&fixture, &options).expect("LF fixture codegen should pass");
    let from_crlf = generate_rust_module_from_json(&fixture_crlf, &options)
        .expect("CRLF fixture codegen should pass");

    assert_eq!(
        from_lf, from_crlf,
        "codegen output should be line-ending invariant"
    );
}

fn snapshot_phase(
    phase: &str,
    runtime: &FigmaRuntime,
    tracked: &[(&str, render_engine::NodeId)],
) -> String {
    let mut visibility = tracked
        .iter()
        .map(|(name, node)| format!("{name}={}", runtime.is_visible(*node)))
        .collect::<Vec<_>>();
    visibility.sort();

    let mut instances = runtime
        .collect_render_instances()
        .iter()
        .map(|instance| {
            format!(
                "{:.1},{:.1},{:.1},{:.1}|{:.3},{:.3},{:.3},{:.3}|{}",
                instance.pos[0],
                instance.pos[1],
                instance.size[0],
                instance.size[1],
                instance.color[0],
                instance.color[1],
                instance.color[2],
                instance.color[3],
                instance.flags
            )
        })
        .collect::<Vec<_>>();
    instances.sort();

    format!(
        "phase:{phase}\nvisibility:{}\ninstances:{}",
        visibility.join(","),
        instances.join(";")
    )
}

fn manifest_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn load_text(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn assert_snapshot_matches(path: &Path, actual: &str) {
    if should_update_goldens() || !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
        }
        std::fs::write(path, actual)
            .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));

        assert!(
            should_update_goldens(),
            "missing golden snapshot {}. generated from current output; rerun with ARTHROPOD_UPDATE_GOLDENS=1 to accept it.",
            path.display()
        );
    }

    let expected = load_text(path);
    assert_eq!(
        normalize_newlines(&expected),
        normalize_newlines(actual),
        "snapshot mismatch for {}",
        path.display()
    );
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

fn should_update_goldens() -> bool {
    std::env::var("ARTHROPOD_UPDATE_GOLDENS")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}
