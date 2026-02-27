use arthropod::figma::{
    ConstraintAxis, LayoutPositioning, PrototypeTrigger, import_figma_document,
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
