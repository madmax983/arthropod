use arthropod::figma::import_figma_document;
use arthropod::prototype_runtime::{
    PrototypeRuntime, PrototypeRuntimeEffect, PrototypeRuntimeEvent,
};

#[test]
fn prototype_runtime_click_navigate_updates_current_screen_and_history() {
    let json = r#"{
        "nodes": [
            {
                "id": "1",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 200 },
                "prototypeInteractions": [
                    {
                        "trigger": "ON_CLICK",
                        "actions": [
                            {
                                "type": "NAVIGATE",
                                "destinationId": "2",
                                "transition": { "type": "DISSOLVE", "duration": 0.25 }
                            }
                        ]
                    }
                ]
            },
            {
                "id": "2",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 320, "y": 0, "width": 300, "height": 200 }
            }
        ]
    }"#;

    let imported = import_figma_document(json).expect("import should succeed");
    let node1 = imported.figma_to_scene.get("1").copied().unwrap();
    let node2 = imported.figma_to_scene.get("2").copied().unwrap();

    let mut runtime = PrototypeRuntime::new(&imported).expect("runtime should initialize");
    runtime.set_current_screen(node1);

    let effects = runtime.dispatch(PrototypeRuntimeEvent::Click { node: node1 });
    assert_eq!(runtime.current_screen(), Some(node2));
    assert_eq!(runtime.history(), &[node1]);
    assert!(effects.iter().any(|effect| matches!(
        effect,
        PrototypeRuntimeEffect::Navigate { from, to, .. } if *from == node1 && *to == node2
    )));
}

#[test]
fn prototype_runtime_overlay_and_back_behaves_as_stack() {
    let json = r#"{
        "nodes": [
            {
                "id": "10",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 200 },
                "prototypeInteractions": [
                    {
                        "trigger": "ON_CLICK",
                        "actions": [{ "type": "OPEN_OVERLAY", "destinationId": "11" }]
                    }
                ]
            },
            {
                "id": "11",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 40, "y": 40, "width": 200, "height": 120 },
                "prototypeInteractions": [
                    {
                        "trigger": "ON_CLICK",
                        "actions": [{ "type": "BACK" }]
                    }
                ]
            }
        ]
    }"#;

    let imported = import_figma_document(json).expect("import should succeed");
    let root = imported.figma_to_scene.get("10").copied().unwrap();
    let overlay = imported.figma_to_scene.get("11").copied().unwrap();

    let mut runtime = PrototypeRuntime::new(&imported).expect("runtime should initialize");
    runtime.set_current_screen(root);

    runtime.dispatch(PrototypeRuntimeEvent::Click { node: root });
    assert_eq!(runtime.overlay_stack().len(), 1);
    assert_eq!(runtime.overlay_stack()[0].node, overlay);

    let effects = runtime.dispatch(PrototypeRuntimeEvent::Click { node: overlay });
    assert!(runtime.overlay_stack().is_empty());
    assert_eq!(runtime.current_screen(), Some(root));
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, PrototypeRuntimeEffect::CloseOverlay { .. }))
    );
}

#[test]
fn prototype_runtime_after_timeout_fires_on_tick_budget() {
    let json = r#"{
        "nodes": [
            {
                "id": "20",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 200 },
                "prototypeInteractions": [
                    {
                        "trigger": { "type": "AFTER_TIMEOUT", "timeout": 0.3 },
                        "actions": [{ "type": "NAVIGATE", "destinationId": "21" }]
                    }
                ]
            },
            {
                "id": "21",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 320, "y": 0, "width": 300, "height": 200 }
            }
        ]
    }"#;

    let imported = import_figma_document(json).expect("import should succeed");
    let source = imported.figma_to_scene.get("20").copied().unwrap();
    let dest = imported.figma_to_scene.get("21").copied().unwrap();

    let mut runtime = PrototypeRuntime::new(&imported).expect("runtime should initialize");
    runtime.set_current_screen(source);

    let first = runtime.dispatch(PrototypeRuntimeEvent::Tick { elapsed_ms: 200 });
    assert!(first.is_empty(), "timeout should not fire before threshold");
    assert_eq!(runtime.current_screen(), Some(source));

    let second = runtime.dispatch(PrototypeRuntimeEvent::Tick { elapsed_ms: 120 });
    assert_eq!(runtime.current_screen(), Some(dest));
    assert!(second.iter().any(
        |effect| matches!(effect, PrototypeRuntimeEffect::Navigate { to, .. } if *to == dest)
    ));
}

#[test]
fn prototype_runtime_url_action_emits_effect_without_navigation() {
    let json = r#"{
        "nodes": [
            {
                "id": "30",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 200 },
                "prototypeInteractions": [
                    {
                        "trigger": "ON_CLICK",
                        "actions": [{ "type": "URL", "url": "https://example.com" }]
                    }
                ]
            }
        ]
    }"#;

    let imported = import_figma_document(json).expect("import should succeed");
    let node = imported.figma_to_scene.get("30").copied().unwrap();
    let mut runtime = PrototypeRuntime::new(&imported).expect("runtime should initialize");
    runtime.set_current_screen(node);

    let effects = runtime.dispatch(PrototypeRuntimeEvent::Click { node });
    assert_eq!(runtime.current_screen(), Some(node));
    assert!(
        effects.iter().any(
            |effect| matches!(effect, PrototypeRuntimeEffect::OpenUrl { url } if url == "https://example.com")
        ),
        "url actions should emit side effect"
    );
}
