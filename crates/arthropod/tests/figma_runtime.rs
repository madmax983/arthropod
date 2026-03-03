use arthropod::figma_runtime::FigmaRuntime;
use arthropod::prototype_runtime::{PrototypeRuntimeEffect, PrototypeRuntimeEvent};

#[test]
fn figma_runtime_applies_layout_and_collects_render_instances() {
    let json = r#"{
        "nodes": [
            {
                "id": "root",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 220 },
                "layoutMode": "VERTICAL",
                "fills": [{ "type": "SOLID", "color": [0.1, 0.2, 0.3, 1.0] }]
            },
            {
                "id": "child",
                "parentId": "root",
                "type": "RECTANGLE",
                "absoluteBoundingBox": { "x": 20, "y": 24, "width": 120, "height": 44 },
                "fills": [{ "type": "SOLID", "color": [0.7, 0.8, 0.9, 1.0] }]
            }
        ]
    }"#;

    let mut runtime = FigmaRuntime::from_figma_json(json).expect("runtime should initialize");
    runtime.apply_layout(800.0, 600.0);

    let instances = runtime.collect_render_instances();
    assert!(
        !instances.is_empty(),
        "styled imported nodes should produce render instances"
    );
}

#[test]
fn figma_runtime_without_prototype_keeps_all_top_level_nodes_visible() {
    let json = r#"{
        "nodes": [
            {
                "id": "layer-a",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 120, "height": 80 },
                "fills": [{ "type": "SOLID", "color": [1.0, 0.0, 0.0, 1.0] }]
            },
            {
                "id": "layer-b",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 12, "y": 8, "width": 120, "height": 80 },
                "fills": [{ "type": "SOLID", "color": [0.0, 1.0, 0.0, 1.0] }]
            }
        ]
    }"#;

    let runtime = FigmaRuntime::from_figma_json(json).expect("runtime should initialize");
    let layer_a = runtime
        .node_for_figma_id("layer-a")
        .expect("layer-a should resolve");
    let layer_b = runtime
        .node_for_figma_id("layer-b")
        .expect("layer-b should resolve");

    assert!(
        runtime.is_visible(layer_a),
        "top-level layer-a should remain visible without prototype graph"
    );
    assert!(
        runtime.is_visible(layer_b),
        "top-level layer-b should remain visible without prototype graph"
    );
}

#[test]
fn figma_runtime_dispatch_navigate_switches_top_level_visibility() {
    let json = r#"{
        "nodes": [
            {
                "id": "screen-a",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 200 },
                "fills": [{ "type": "SOLID", "color": [0.2, 0.2, 0.2, 1.0] }],
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
                "fills": [{ "type": "SOLID", "color": [0.1, 0.4, 0.7, 1.0] }]
            }
        ]
    }"#;

    let mut runtime = FigmaRuntime::from_figma_json(json).expect("runtime should initialize");
    let screen_a = runtime
        .node_for_figma_id("screen-a")
        .expect("screen-a should resolve");
    let screen_b = runtime
        .node_for_figma_id("screen-b")
        .expect("screen-b should resolve");
    runtime.set_current_screen(screen_a);

    assert!(
        runtime.is_visible(screen_a),
        "active screen should be visible"
    );
    assert!(
        !runtime.is_visible(screen_b),
        "inactive screen should be hidden"
    );

    let effects = runtime.dispatch(PrototypeRuntimeEvent::Click { node: screen_a });
    assert!(effects.iter().any(|effect| matches!(
        effect,
        PrototypeRuntimeEffect::Navigate { to, .. } if *to == screen_b
    )));
    assert!(
        !runtime.is_visible(screen_a),
        "source screen should be hidden"
    );
    assert!(
        runtime.is_visible(screen_b),
        "destination screen should be visible"
    );
}

#[test]
fn figma_runtime_overlay_visibility_tracks_open_and_close() {
    let json = r#"{
        "nodes": [
            {
                "id": "screen",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 200 },
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
                "absoluteBoundingBox": { "x": 20, "y": 20, "width": 160, "height": 100 },
                "prototypeInteractions": [
                    { "trigger": "ON_CLICK", "actions": [{ "type": "BACK" }] }
                ]
            }
        ]
    }"#;

    let mut runtime = FigmaRuntime::from_figma_json(json).expect("runtime should initialize");
    let screen = runtime
        .node_for_figma_id("screen")
        .expect("screen should resolve");
    let overlay = runtime
        .node_for_figma_id("overlay")
        .expect("overlay should resolve");
    runtime.set_current_screen(screen);

    assert!(!runtime.is_visible(overlay), "overlay starts hidden");

    runtime.dispatch(PrototypeRuntimeEvent::Click { node: screen });
    assert!(runtime.is_visible(overlay), "open_overlay should show node");

    runtime.dispatch(PrototypeRuntimeEvent::Click { node: overlay });
    assert!(
        !runtime.is_visible(overlay),
        "back from overlay should hide overlay node"
    );
}
