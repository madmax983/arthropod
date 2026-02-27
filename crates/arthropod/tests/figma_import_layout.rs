use arthropod::figma::{
    ConstraintAxis, ImportedComponentKind, LayoutPositioning, PrototypeTrigger,
    import_figma_document,
};
use layout_engine::FlexDirection;
use render_engine::NodeContent;
use style_engine::{FontStyle, LineHeight, TextAlign};

#[test]
fn figma_auto_layout_maps_to_flex_style_and_hierarchy() {
    let json = r#"{
        "nodes": [
            {
                "id": "10",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 20, "y": 30, "width": 600, "height": 300 },
                "layoutMode": "HORIZONTAL",
                "primaryAxisSizingMode": "FIXED",
                "counterAxisSizingMode": "FIXED",
                "itemSpacing": 12,
                "paddingLeft": 8,
                "paddingRight": 9,
                "paddingTop": 10,
                "paddingBottom": 11
            },
            {
                "id": "11",
                "parentId": "10",
                "type": "RECTANGLE",
                "absoluteBoundingBox": { "x": 30, "y": 40, "width": 120, "height": 70 },
                "layoutGrow": 1
            }
        ]
    }"#;

    let imported = import_figma_document(json).expect("figma import should succeed");
    let root_scene_id = imported
        .figma_to_scene
        .get("10")
        .copied()
        .expect("root figma id must map to scene node");
    let child_scene_id = imported
        .figma_to_scene
        .get("11")
        .copied()
        .expect("child figma id must map to scene node");

    let root_style = imported
        .layout_styles
        .get(&root_scene_id)
        .expect("root layout style should exist");
    assert_eq!(root_style.direction, FlexDirection::Row);
    assert!((root_style.gap - 12.0).abs() < 1e-6);
    assert!((root_style.padding_left - 8.0).abs() < 1e-6);
    assert!((root_style.padding_right - 9.0).abs() < 1e-6);
    assert!((root_style.padding_top - 10.0).abs() < 1e-6);
    assert!((root_style.padding_bottom - 11.0).abs() < 1e-6);
    assert_eq!(root_style.width, Some(600.0));
    assert_eq!(root_style.height, Some(300.0));

    let child_style = imported
        .layout_styles
        .get(&child_scene_id)
        .expect("child layout style should exist");
    assert!((child_style.flex_grow - 1.0).abs() < 1e-6);

    assert_eq!(
        imported.scene.parent(child_scene_id),
        Some(root_scene_id),
        "child should be attached to mapped parent in imported scene"
    );
}

#[test]
fn figma_constraints_and_positioning_map_to_imported_constraints() {
    let json = r#"{
        "nodes": [
            {
                "id": "20",
                "type": "RECTANGLE",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 100, "height": 50 },
                "constraints": { "horizontal": "RIGHT", "vertical": "SCALE" },
                "layoutPositioning": "ABSOLUTE"
            }
        ]
    }"#;

    let imported = import_figma_document(json).expect("figma import should succeed");
    let node_id = imported
        .figma_to_scene
        .get("20")
        .copied()
        .expect("mapped scene id should exist");
    let constraints = imported
        .constraints
        .get(&node_id)
        .expect("constraints should be captured for imported node");

    assert_eq!(constraints.horizontal, ConstraintAxis::Max);
    assert_eq!(constraints.vertical, ConstraintAxis::Scale);
    assert_eq!(constraints.positioning, LayoutPositioning::Absolute);
}

#[test]
fn figma_text_node_maps_characters_and_type_style() {
    let json = r#"{
        "nodes": [
            {
                "id": "30",
                "type": "TEXT",
                "absoluteBoundingBox": { "x": 10, "y": 20, "width": 180, "height": 40 },
                "characters": "Ship it",
                "style": {
                    "fontFamily": "Inter",
                    "fontSize": 18,
                    "fontWeight": 700,
                    "italic": true,
                    "textAlignHorizontal": "CENTER",
                    "lineHeightPx": 24,
                    "letterSpacing": 1.25
                }
            }
        ]
    }"#;

    let imported = import_figma_document(json).expect("figma import should succeed");
    let node_id = imported
        .figma_to_scene
        .get("30")
        .copied()
        .expect("mapped scene id should exist");
    let node = imported
        .scene
        .get_node(node_id)
        .expect("imported node should exist");

    let NodeContent::Styled { style } = &node.content else {
        panic!("expected styled node for imported text");
    };
    let text = style.text.as_ref().expect("text style should be present");
    assert_eq!(text.text, "Ship it");
    assert!((text.font_size - 18.0).abs() < 1e-6);
    assert_eq!(text.font_weight, 700);
    assert_eq!(text.font_style, FontStyle::Italic);
    assert_eq!(text.align, TextAlign::Center);
    assert_eq!(text.line_height, LineHeight::Fixed(24.0));
    assert_eq!(text.font_family.as_deref(), Some("Inter"));
    assert!((text.letter_spacing - 1.25).abs() < 1e-6);
}

#[test]
fn figma_prototype_interactions_map_to_graph_edges() {
    let json = r#"{
        "nodes": [
            {
                "id": "100",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 200 },
                "prototypeInteractions": [
                    { "trigger": "ON_CLICK", "destinationId": "200" }
                ]
            },
            {
                "id": "200",
                "type": "FRAME",
                "absoluteBoundingBox": { "x": 20, "y": 20, "width": 300, "height": 200 }
            }
        ]
    }"#;

    let imported = import_figma_document(json).expect("figma import should succeed");
    let source = imported
        .figma_to_scene
        .get("100")
        .copied()
        .expect("source node should be mapped");
    let edge = imported
        .prototype_graph
        .edges
        .iter()
        .find(|edge| edge.from == source)
        .expect("expected prototype edge from source node");

    assert_eq!(edge.to_figma_id, "200");
    assert_eq!(edge.trigger, PrototypeTrigger::OnClick);
}

#[test]
fn figma_component_and_instance_nodes_map_to_metadata() {
    let json = r#"{
        "nodes": [
            {
                "id": "500",
                "type": "COMPONENT_SET",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 300, "height": 150 },
                "key": "set-key-abc"
            },
            {
                "id": "510",
                "parentId": "500",
                "type": "COMPONENT",
                "componentSetId": "500",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 140, "height": 64 },
                "key": "component-key-def",
                "variantProperties": {
                    "State": "Default",
                    "Size": "M"
                }
            },
            {
                "id": "700",
                "type": "INSTANCE",
                "componentId": "510",
                "mainComponent": { "id": "510" },
                "absoluteBoundingBox": { "x": 20, "y": 20, "width": 140, "height": 64 },
                "variantProperties": {
                    "State": "Hover"
                }
            }
        ]
    }"#;

    let imported = import_figma_document(json).expect("figma import should succeed");
    let set_id = imported
        .figma_to_scene
        .get("500")
        .copied()
        .expect("component set should map");
    let component_id = imported
        .figma_to_scene
        .get("510")
        .copied()
        .expect("component should map");
    let instance_id = imported
        .figma_to_scene
        .get("700")
        .copied()
        .expect("instance should map");

    let set_meta = imported
        .components
        .get(&set_id)
        .expect("component set metadata should be present");
    assert_eq!(set_meta.kind, ImportedComponentKind::ComponentSet);
    assert_eq!(set_meta.key.as_deref(), Some("set-key-abc"));

    let component_meta = imported
        .components
        .get(&component_id)
        .expect("component metadata should be present");
    assert_eq!(component_meta.kind, ImportedComponentKind::Component);
    assert_eq!(component_meta.key.as_deref(), Some("component-key-def"));
    assert_eq!(component_meta.component_set_id.as_deref(), Some("500"));

    let instance_meta = imported
        .instances
        .get(&instance_id)
        .expect("instance metadata should be present");
    assert_eq!(instance_meta.component_id.as_deref(), Some("510"));
    assert_eq!(instance_meta.main_component_id.as_deref(), Some("510"));

    let component_variants = imported
        .variant_properties
        .get(&component_id)
        .expect("component variants should exist");
    assert_eq!(
        component_variants.get("State").map(String::as_str),
        Some("Default")
    );
    assert_eq!(
        component_variants.get("Size").map(String::as_str),
        Some("M")
    );

    let instance_variants = imported
        .variant_properties
        .get(&instance_id)
        .expect("instance variants should exist");
    assert_eq!(
        instance_variants.get("State").map(String::as_str),
        Some("Hover")
    );
}

#[test]
fn figma_variant_properties_extract_from_component_properties_variant_type() {
    let json = r#"{
        "nodes": [
            {
                "id": "800",
                "type": "INSTANCE",
                "absoluteBoundingBox": { "x": 0, "y": 0, "width": 120, "height": 40 },
                "componentProperties": {
                    "State#12:0": { "type": "VARIANT", "value": "Pressed" },
                    "Density#12:1": { "type": "VARIANT", "value": "Compact" },
                    "Disabled#12:2": { "type": "BOOLEAN", "value": true }
                }
            }
        ]
    }"#;

    let imported = import_figma_document(json).expect("figma import should succeed");
    let node_id = imported
        .figma_to_scene
        .get("800")
        .copied()
        .expect("instance should map");
    let variants = imported
        .variant_properties
        .get(&node_id)
        .expect("variant properties should exist");

    assert_eq!(variants.get("State").map(String::as_str), Some("Pressed"));
    assert_eq!(variants.get("Density").map(String::as_str), Some("Compact"));
    assert!(
        !variants.contains_key("Disabled"),
        "non-variant component properties should not be projected into variant map"
    );
}
