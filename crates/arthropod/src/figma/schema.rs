use std::collections::HashMap;
use std::fmt::Write as _;

use layout_engine::{
    FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle, FlexWrap, ItemAlignSelf,
};
use plat_core::Rect;
use render_engine::{NodeId, Transform2D, Vec2, Vec4};
use serde::Deserialize;
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use style_engine::{
    BackgroundBlur, BlendMode, ColorFilter, ColorStop, CornerRadii, DropShadow, Effect, FontStyle,
    ImageFill, ImageId, ImageScaleMode, InnerShadow, LayerBlur, LineHeight, LinearGradient,
    MaskType, Paint, RadialGradient, SideWeights, StrokeAlign, StrokeCap, StrokeJoin, StrokeStyle,
    TextAlign, TextAlignVertical, TextAutoResize, TextCase, TextContent, TextDecoration,
    TextOverflow, VectorPath, VisualStyle, WindingRule,
};

use super::*;

pub(crate) fn parse_figma_document(json: &str) -> Result<FigmaDocument, FigmaImportError> {
    let mut raw: JsonValue = serde_json::from_str(json)?;
    normalize_enum_wrappers(&mut raw);
    let nodes = match raw {
        JsonValue::Array(nodes) => nodes,
        JsonValue::Object(mut object) => match object.remove("nodes") {
            Some(JsonValue::Array(nodes)) => nodes,
            _ => return Err(FigmaImportError::InvalidDocumentShape),
        },
        _ => return Err(FigmaImportError::InvalidDocumentShape),
    };

    let mut flattened = Vec::new();
    let mut path = Vec::new();
    flatten_document_nodes(&nodes, None, &mut path, &mut flattened);

    let mut normalized = JsonMap::new();
    normalized.insert("nodes".to_string(), JsonValue::Array(flattened));
    Ok(serde_json::from_value(JsonValue::Object(normalized))?)
}

pub(crate) fn normalize_enum_wrappers(value: &mut JsonValue) {
    match value {
        JsonValue::Array(items) => {
            for item in items {
                normalize_enum_wrappers(item);
            }
        }
        JsonValue::Object(map) => {
            if map.len() == 2 && map.contains_key("__enum__") && map.contains_key("value") {
                if let Some(mut enum_value) = map.remove("value") {
                    normalize_enum_wrappers(&mut enum_value);
                    *value = enum_value;
                }
                return;
            }
            for child in map.values_mut() {
                normalize_enum_wrappers(child);
            }
        }
        _ => {}
    }
}

pub(crate) fn flatten_document_nodes(
    nodes: &[JsonValue],
    parent_id: Option<&str>,
    path: &mut Vec<usize>,
    flattened: &mut Vec<JsonValue>,
) {
    for (index, raw_node) in nodes.iter().enumerate() {
        let JsonValue::Object(mut object) = raw_node.clone() else {
            continue;
        };
        path.push(index);

        let node_id = extract_node_id(&object, path);
        object.insert("id".to_string(), JsonValue::String(node_id.clone()));
        if let Some(parent_id) = parent_id {
            object
                .entry("parentId".to_string())
                .or_insert_with(|| JsonValue::String(parent_id.to_string()));
        }

        ensure_bounds_from_xywh(&mut object);
        normalize_text_style_fields(&mut object);
        normalize_paint_fields(&mut object);

        let children = object
            .remove("children")
            .and_then(|value| match value {
                JsonValue::Array(children) => Some(children),
                _ => None,
            })
            .unwrap_or_default();

        flattened.push(JsonValue::Object(object));
        flatten_document_nodes(&children, Some(&node_id), path, flattened);
        path.pop();
    }
}

pub(crate) fn extract_node_id(object: &JsonMap<String, JsonValue>, path: &[usize]) -> String {
    if let Some(value) = object.get("id") {
        if let Some(text) = value.as_str() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Some(number) = value.as_u64() {
            return number.to_string();
        }
        if let Some(number) = value.as_i64() {
            return number.to_string();
        }
    }

    let path_text = path
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".");
    format!("generated:{path_text}")
}

pub(crate) fn ensure_bounds_from_xywh(object: &mut JsonMap<String, JsonValue>) {
    if object.contains_key("absoluteBoundingBox") || object.contains_key("bounds") {
        return;
    }

    let Some([x, y, width, height]) =
        bounds_from_xywh(object).or_else(|| bounds_from_size_and_transform(object))
    else {
        return;
    };

    object.insert(
        "bounds".to_string(),
        JsonValue::Array(vec![
            JsonValue::Number(x),
            JsonValue::Number(y),
            JsonValue::Number(width),
            JsonValue::Number(height),
        ]),
    );
}

pub(crate) fn bounds_from_xywh(object: &JsonMap<String, JsonValue>) -> Option<[JsonNumber; 4]> {
    let x = json_number(object.get("x"))?;
    let y = json_number(object.get("y"))?;
    let width = json_number(object.get("width"))?;
    let height = json_number(object.get("height"))?;
    Some([x, y, width, height])
}

pub(crate) fn bounds_from_size_and_transform(
    object: &JsonMap<String, JsonValue>,
) -> Option<[JsonNumber; 4]> {
    let size = object.get("size")?.as_object()?;
    let width = json_number(size.get("x").or_else(|| size.get("width")))?;
    let height = json_number(size.get("y").or_else(|| size.get("height")))?;
    let x = json_number(object.get("x"))
        .or_else(|| transform_translation_component(object.get("transform"), true))
        .or_else(|| JsonNumber::from_f64(0.0))?;
    let y = json_number(object.get("y"))
        .or_else(|| transform_translation_component(object.get("transform"), false))
        .or_else(|| JsonNumber::from_f64(0.0))?;
    Some([x, y, width, height])
}

pub(crate) fn transform_translation_component(
    transform: Option<&JsonValue>,
    horizontal: bool,
) -> Option<JsonNumber> {
    match transform? {
        JsonValue::Object(map) => {
            let key = if horizontal { "m02" } else { "m12" };
            json_number(map.get(key))
        }
        JsonValue::Array(values) => {
            // Flat affine matrix form: [m00, m01, m02, m10, m11, m12]
            if values.len() >= 6 {
                let index = if horizontal { 2 } else { 5 };
                return json_number(values.get(index));
            }
            // Row matrix form: [[m00,m01,m02],[m10,m11,m12],...]
            if values.len() >= 2
                && let (Some(JsonValue::Array(row0)), Some(JsonValue::Array(row1))) =
                    (values.first(), values.get(1))
            {
                return if horizontal {
                    json_number(row0.get(2))
                } else {
                    json_number(row1.get(2))
                };
            }
            None
        }
        _ => None,
    }
}

pub(crate) fn json_number(value: Option<&JsonValue>) -> Option<JsonNumber> {
    let value = value.and_then(JsonValue::as_f64)?;
    JsonNumber::from_f64(value)
}

pub(crate) fn normalize_text_style_fields(object: &mut JsonMap<String, JsonValue>) {
    let is_text = object
        .get("type")
        .and_then(JsonValue::as_str)
        .is_some_and(|kind| kind.eq_ignore_ascii_case("TEXT"));
    if !is_text {
        return;
    }

    let had_style = object.contains_key("style");
    let mut style = match object.remove("style") {
        Some(JsonValue::Object(style)) => style,
        Some(other) => {
            object.insert("style".to_string(), other);
            return;
        }
        None => JsonMap::new(),
    };

    insert_style_alias(&mut style, object, "fontSize", "fontSize");
    insert_style_alias(&mut style, object, "fontWeight", "fontWeight");
    insert_style_alias(
        &mut style,
        object,
        "textAlignHorizontal",
        "textAlignHorizontal",
    );
    insert_style_alias(&mut style, object, "textAlignVertical", "textAlignVertical");
    insert_style_alias(&mut style, object, "letterSpacing", "letterSpacing");
    insert_style_alias(&mut style, object, "textDecoration", "textDecoration");
    insert_style_alias(&mut style, object, "textCase", "textCase");
    insert_style_alias(&mut style, object, "paragraphSpacing", "paragraphSpacing");
    insert_style_alias(&mut style, object, "paragraphIndent", "paragraphIndent");
    insert_style_alias(&mut style, object, "textTruncation", "textTruncation");

    if !style.contains_key("fontFamily")
        && let Some(family) = object
            .get("fontName")
            .and_then(JsonValue::as_object)
            .and_then(|font_name| font_name.get("family"))
            .cloned()
    {
        style.insert("fontFamily".to_string(), family);
    }
    if !style.contains_key("fontStyle")
        && let Some(font_style) = object
            .get("fontName")
            .and_then(JsonValue::as_object)
            .and_then(|font_name| font_name.get("style"))
            .cloned()
    {
        style.insert("fontStyle".to_string(), font_style);
    }

    if !style.contains_key("lineHeightPx") && !style.contains_key("lineHeightPercentFontSize") {
        normalize_line_height_alias(&mut style, object.get("lineHeight"));
    }

    if had_style || !style.is_empty() {
        object.insert("style".to_string(), JsonValue::Object(style));
    }
}

pub(crate) fn normalize_paint_fields(object: &mut JsonMap<String, JsonValue>) {
    if !object.contains_key("fills") {
        if let Some(paints) = object.get("fillPaints").cloned() {
            object.insert("fills".to_string(), paints);
        } else if let Some(color) = object
            .get("backgroundColor")
            .cloned()
            .filter(|_| object.get("backgroundEnabled").and_then(JsonValue::as_bool) != Some(false))
        {
            let opacity = object
                .get("backgroundOpacity")
                .and_then(JsonValue::as_f64)
                .and_then(JsonNumber::from_f64)
                .unwrap_or_else(|| JsonNumber::from(1));
            let paint = serde_json::json!({
                "type": "SOLID",
                "color": color,
                "opacity": opacity,
                "visible": true
            });
            object.insert("fills".to_string(), JsonValue::Array(vec![paint]));
        }
    }

    if !object.contains_key("strokes")
        && let Some(paints) = object.get("strokePaints").cloned()
    {
        object.insert("strokes".to_string(), paints);
    }
}

pub(crate) fn insert_style_alias(
    style: &mut JsonMap<String, JsonValue>,
    object: &JsonMap<String, JsonValue>,
    style_key: &str,
    source_key: &str,
) {
    if style.contains_key(style_key) {
        return;
    }
    if let Some(value) = object.get(source_key).cloned() {
        style.insert(style_key.to_string(), value);
    }
}

pub(crate) fn normalize_line_height_alias(
    style: &mut JsonMap<String, JsonValue>,
    line_height: Option<&JsonValue>,
) {
    let Some(line_height) = line_height else {
        return;
    };

    if let Some(value) = line_height.as_f64().and_then(JsonNumber::from_f64) {
        style.insert("lineHeightPx".to_string(), JsonValue::Number(value));
        return;
    }

    let Some(object) = line_height.as_object() else {
        return;
    };
    let Some(value) = object
        .get("value")
        .and_then(JsonValue::as_f64)
        .and_then(JsonNumber::from_f64)
    else {
        return;
    };
    let unit = object
        .get("unit")
        .and_then(JsonValue::as_str)
        .map(str::to_ascii_uppercase);

    match unit.as_deref() {
        Some("PERCENT") | Some("PERCENT_FONT_SIZE") => {
            style.insert(
                "lineHeightPercentFontSize".to_string(),
                JsonValue::Number(value),
            );
        }
        _ => {
            style.insert("lineHeightPx".to_string(), JsonValue::Number(value));
        }
    }
}

pub(crate) fn map_color_stops(stops: &[FigmaColorStop]) -> Vec<ColorStop> {
    stops
        .iter()
        .filter_map(|stop| {
            if !stop.position.is_finite() {
                return None;
            }
            stop.color
                .to_vec4()
                .map(|color| ColorStop::new(stop.position, color))
        })
        .collect()
}

pub(crate) fn normalize_color_component(value: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    if value > 1.0 {
        (value / 255.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

pub(crate) fn normalize_alpha_component(value: f32) -> f32 {
    if !value.is_finite() {
        return 1.0;
    }
    if value > 1.0 {
        (value / 255.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

pub(crate) fn parse_hex_color(hex: &str) -> Option<Vec4> {
    let text = hex.trim();
    let bytes = text.strip_prefix('#').unwrap_or(text);
    match bytes.len() {
        3 => {
            let r = parse_hex_nibble(bytes.as_bytes()[0])? * 17;
            let g = parse_hex_nibble(bytes.as_bytes()[1])? * 17;
            let b = parse_hex_nibble(bytes.as_bytes()[2])? * 17;
            Some(Vec4::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                1.0,
            ))
        }
        4 => {
            let r = parse_hex_nibble(bytes.as_bytes()[0])? * 17;
            let g = parse_hex_nibble(bytes.as_bytes()[1])? * 17;
            let b = parse_hex_nibble(bytes.as_bytes()[2])? * 17;
            let a = parse_hex_nibble(bytes.as_bytes()[3])? * 17;
            Some(Vec4::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            ))
        }
        6 => {
            let bytes = bytes.as_bytes();
            Some(Vec4::new(
                parse_hex_byte(bytes[0], bytes[1])? as f32 / 255.0,
                parse_hex_byte(bytes[2], bytes[3])? as f32 / 255.0,
                parse_hex_byte(bytes[4], bytes[5])? as f32 / 255.0,
                1.0,
            ))
        }
        8 => {
            let bytes = bytes.as_bytes();
            Some(Vec4::new(
                parse_hex_byte(bytes[0], bytes[1])? as f32 / 255.0,
                parse_hex_byte(bytes[2], bytes[3])? as f32 / 255.0,
                parse_hex_byte(bytes[4], bytes[5])? as f32 / 255.0,
                parse_hex_byte(bytes[6], bytes[7])? as f32 / 255.0,
            ))
        }
        _ => None,
    }
}

pub(crate) fn parse_hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn parse_hex_byte(high: u8, low: u8) -> Option<u8> {
    let high = parse_hex_nibble(high)?;
    let low = parse_hex_nibble(low)?;
    Some((high << 4) | low)
}

pub(crate) fn figma_image_reference_to_id(reference: &str) -> u64 {
    if let Ok(parsed) = reference.parse::<u64>() {
        return parsed;
    }
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in reference.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash | (1_u64 << 63)
}

pub(crate) fn figma_image_transform(
    scale_mode: FigmaImageScaleMode,
    transform: Option<&FigmaImageTransform>,
    scaling_factor: Option<f32>,
) -> Option<[f32; 9]> {
    let transform = transform.map(FigmaImageTransform::to_matrix3x3);
    if !matches!(scale_mode, FigmaImageScaleMode::Tile) {
        return transform;
    }
    let Some(scaling_factor) = scaling_factor.filter(|value| value.is_finite() && *value > 0.0)
    else {
        return transform;
    };
    if transform.is_some() || (scaling_factor - 1.0).abs() <= f32::EPSILON {
        return transform;
    }
    let inverse = 1.0 / scaling_factor;
    Some([inverse, 0.0, 0.0, 0.0, inverse, 0.0, 0.0, 0.0, 1.0])
}

pub(crate) fn figma_stroke_miter_limit_from_angle(angle_degrees: f32) -> Option<f32> {
    if !angle_degrees.is_finite() || angle_degrees <= 0.0 {
        return None;
    }
    let half_radians = 0.5 * angle_degrees.to_radians();
    let sin = half_radians.sin().abs();
    if sin <= f32::EPSILON {
        return None;
    }
    Some(1.0 / sin)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaDocument {
    #[serde(default)]
    pub(crate) nodes: Vec<FigmaNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaNode {
    pub(crate) id: FigmaNodeKey,
    #[serde(default, alias = "parentId")]
    pub(crate) parent_id: Option<FigmaNodeKey>,
    #[serde(default, rename = "type")]
    pub(crate) node_type: Option<FigmaNodeType>,
    #[serde(default)]
    pub(crate) key: Option<String>,
    #[serde(default, alias = "componentSetId")]
    pub(crate) component_set_id: Option<FigmaNodeKey>,
    #[serde(default, alias = "componentId")]
    pub(crate) component_id: Option<FigmaNodeKey>,
    #[serde(default, alias = "mainComponent")]
    pub(crate) main_component: Option<FigmaMainComponent>,
    #[serde(default, alias = "mainComponentId")]
    pub(crate) main_component_id: Option<FigmaNodeKey>,
    #[serde(default)]
    pub(crate) variant_properties: HashMap<String, FigmaPropertyValue>,
    #[serde(default)]
    pub(crate) component_properties: HashMap<String, FigmaComponentProperty>,
    #[serde(default)]
    pub(crate) component_property_definitions: HashMap<String, FigmaComponentPropertyDefinition>,
    #[serde(default)]
    pub(crate) fills: Vec<FigmaPaint>,
    #[serde(default)]
    pub(crate) strokes: Vec<FigmaPaint>,
    #[serde(default)]
    pub(crate) stroke_weight: Option<f32>,
    #[serde(default)]
    pub(crate) stroke_align: Option<FigmaStrokeAlign>,
    #[serde(default)]
    pub(crate) stroke_cap: Option<FigmaStrokeCap>,
    #[serde(default)]
    pub(crate) stroke_join: Option<FigmaStrokeJoin>,
    #[serde(default)]
    pub(crate) stroke_miter_angle: Option<f32>,
    #[serde(default)]
    pub(crate) stroke_miter_limit: Option<f32>,
    #[serde(default)]
    pub(crate) stroke_dashes: Option<Vec<f32>>,
    #[serde(default, alias = "strokeDashOffset")]
    pub(crate) dash_offset: Option<f32>,
    #[serde(default, alias = "individualStrokeWeights")]
    pub(crate) individual_stroke_weights: Option<FigmaSideWeights>,
    #[serde(default)]
    pub(crate) effects: Vec<FigmaEffect>,
    #[serde(default)]
    pub(crate) fill_geometry: Option<Vec<FigmaPathGeometry>>,
    #[serde(default)]
    pub(crate) stroke_geometry: Option<Vec<FigmaPathGeometry>>,
    #[serde(default)]
    pub(crate) corner_radius: Option<f32>,
    #[serde(default)]
    pub(crate) rectangle_corner_radii: Option<[f32; 4]>,
    #[serde(default)]
    pub(crate) corner_smoothing: Option<f32>,
    #[serde(default)]
    pub(crate) opacity: Option<f32>,
    #[serde(default)]
    pub(crate) rotation: Option<f32>,
    #[serde(default)]
    pub(crate) blend_mode: Option<FigmaBlendMode>,
    #[serde(default)]
    pub(crate) clips_content: Option<bool>,
    #[serde(default, alias = "isMask")]
    pub(crate) is_mask: Option<bool>,
    #[serde(default, alias = "maskType")]
    pub(crate) mask_type: Option<FigmaMaskType>,
    #[serde(default)]
    pub(crate) bounds: Option<[f32; 4]>,
    #[serde(default, alias = "absoluteBoundingBox")]
    pub(crate) absolute_bounding_box: Option<FigmaRect>,
    #[serde(default)]
    pub(crate) layout_mode: Option<FigmaLayoutMode>,
    #[serde(default)]
    pub(crate) primary_axis_align_items: Option<FigmaPrimaryAxisAlignItems>,
    #[serde(default)]
    pub(crate) counter_axis_align_items: Option<FigmaCounterAxisAlignItems>,
    #[serde(default)]
    pub(crate) layout_wrap: Option<FigmaLayoutWrap>,
    #[serde(default)]
    pub(crate) primary_axis_sizing_mode: Option<FigmaAxisSizingMode>,
    #[serde(default)]
    pub(crate) counter_axis_sizing_mode: Option<FigmaAxisSizingMode>,
    #[serde(default)]
    pub(crate) layout_align: Option<FigmaLayoutAlign>,
    #[serde(default)]
    pub(crate) layout_sizing_horizontal: Option<FigmaLayoutSizingMode>,
    #[serde(default)]
    pub(crate) layout_sizing_vertical: Option<FigmaLayoutSizingMode>,
    #[serde(default)]
    pub(crate) item_spacing: Option<f32>,
    #[serde(default)]
    pub(crate) padding_left: Option<f32>,
    #[serde(default)]
    pub(crate) padding_right: Option<f32>,
    #[serde(default)]
    pub(crate) padding_top: Option<f32>,
    #[serde(default)]
    pub(crate) padding_bottom: Option<f32>,
    #[serde(default)]
    pub(crate) layout_grow: Option<f32>,
    #[serde(default)]
    pub(crate) min_width: Option<f32>,
    #[serde(default)]
    pub(crate) max_width: Option<f32>,
    #[serde(default)]
    pub(crate) min_height: Option<f32>,
    #[serde(default)]
    pub(crate) max_height: Option<f32>,
    #[serde(default)]
    pub(crate) constraints: Option<FigmaConstraints>,
    #[serde(default)]
    pub(crate) layout_positioning: Option<FigmaLayoutPositioning>,
    #[serde(default)]
    pub(crate) characters: Option<String>,
    #[serde(default)]
    pub(crate) text_auto_resize: Option<FigmaTextAutoResize>,
    #[serde(default)]
    pub(crate) max_lines: Option<u32>,
    #[serde(default)]
    pub(crate) text_truncation: Option<FigmaTextTruncation>,
    #[serde(default)]
    pub(crate) style: Option<FigmaTypeStyle>,
    #[serde(default, alias = "interactions", alias = "prototypeInteractions")]
    pub(crate) prototype_interactions: Vec<FigmaPrototypeInteraction>,
}

impl FigmaNode {
    pub(crate) fn to_node_opacity(&self) -> f32 {
        self.opacity
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(0.0, 1.0))
            .unwrap_or(1.0)
    }

    pub(crate) fn to_node_transform(&self, bounds: Rect) -> Option<Transform2D> {
        let rotation_radians = self
            .rotation
            .filter(|value| value.is_finite())
            .map(f32::to_radians)
            .filter(|value| value.abs() > f32::EPSILON)?;

        let center_x = bounds.x + bounds.width * 0.5;
        let center_y = bounds.y + bounds.height * 0.5;
        let to_center = Transform2D::translate(center_x, center_y);
        let rotate = Transform2D::rotate_radians(rotation_radians);
        let from_center = Transform2D::translate(-center_x, -center_y);
        Some(to_center.compose(&rotate).compose(&from_center))
    }

    pub(crate) fn resolved_bounds(&self) -> Rect {
        if let Some(bounds) = self.absolute_bounding_box {
            return Rect::new(
                bounds.x,
                bounds.y,
                bounds.width.max(0.0),
                bounds.height.max(0.0),
            );
        }
        if let Some([x, y, width, height]) = self.bounds {
            return Rect::new(x, y, width.max(0.0), height.max(0.0));
        }
        Rect::new(0.0, 0.0, 0.0, 0.0)
    }

    pub(crate) fn to_flex_style(&self, bounds: Rect) -> FlexStyle {
        let layout_mode = self.layout_mode.unwrap_or(FigmaLayoutMode::None);
        let mut style = FlexStyle {
            direction: match layout_mode {
                FigmaLayoutMode::Horizontal => FlexDirection::Row,
                FigmaLayoutMode::Vertical => FlexDirection::Column,
                FigmaLayoutMode::None | FigmaLayoutMode::Unknown => FlexDirection::Column,
            },
            justify_content: self
                .primary_axis_align_items
                .map(FigmaPrimaryAxisAlignItems::to_justify_content)
                .unwrap_or(FlexJustifyContent::Start),
            align_items: self
                .counter_axis_align_items
                .map(FigmaCounterAxisAlignItems::to_align)
                .unwrap_or(FlexAlign::Stretch),
            wrap: self
                .layout_wrap
                .map(FigmaLayoutWrap::to_wrap)
                .unwrap_or(FlexWrap::NoWrap),
            align_self: self.layout_align.and_then(FigmaLayoutAlign::to_align_self),
            flex_grow: self
                .layout_grow
                .filter(|value| value.is_finite() && *value >= 0.0)
                .unwrap_or(0.0),
            gap: self
                .item_spacing
                .filter(|value| value.is_finite() && *value >= 0.0)
                .unwrap_or(0.0),
            padding_left: self
                .padding_left
                .filter(|value| value.is_finite())
                .unwrap_or(0.0),
            padding_right: self
                .padding_right
                .filter(|value| value.is_finite())
                .unwrap_or(0.0),
            padding_top: self
                .padding_top
                .filter(|value| value.is_finite())
                .unwrap_or(0.0),
            padding_bottom: self
                .padding_bottom
                .filter(|value| value.is_finite())
                .unwrap_or(0.0),
            min_width: self
                .min_width
                .filter(|value| value.is_finite() && *value >= 0.0),
            max_width: self
                .max_width
                .filter(|value| value.is_finite() && *value >= 0.0),
            min_height: self
                .min_height
                .filter(|value| value.is_finite() && *value >= 0.0),
            max_height: self
                .max_height
                .filter(|value| value.is_finite() && *value >= 0.0),
            ..FlexStyle::default()
        };

        let primary_fixed = self.primary_axis_sizing_mode != Some(FigmaAxisSizingMode::Auto);
        let counter_fixed = self.counter_axis_sizing_mode != Some(FigmaAxisSizingMode::Auto);

        match layout_mode {
            FigmaLayoutMode::Horizontal => {
                style.width = primary_fixed.then_some(bounds.width);
                style.height = counter_fixed.then_some(bounds.height);
            }
            FigmaLayoutMode::Vertical => {
                style.width = counter_fixed.then_some(bounds.width);
                style.height = primary_fixed.then_some(bounds.height);
            }
            FigmaLayoutMode::None | FigmaLayoutMode::Unknown => {
                style.width = Some(bounds.width);
                style.height = Some(bounds.height);
            }
        }

        if let Some(sizing) = self.layout_sizing_horizontal {
            match sizing {
                FigmaLayoutSizingMode::Fixed => {
                    style.width = Some(bounds.width);
                }
                FigmaLayoutSizingMode::Hug | FigmaLayoutSizingMode::Auto => {
                    style.width = None;
                }
                FigmaLayoutSizingMode::Fill => {
                    style.width = None;
                    style.flex_grow = style.flex_grow.max(1.0);
                    style.align_self.get_or_insert(ItemAlignSelf::Stretch);
                }
                FigmaLayoutSizingMode::Unknown => {}
            }
        }
        if let Some(sizing) = self.layout_sizing_vertical {
            match sizing {
                FigmaLayoutSizingMode::Fixed => {
                    style.height = Some(bounds.height);
                }
                FigmaLayoutSizingMode::Hug | FigmaLayoutSizingMode::Auto => {
                    style.height = None;
                }
                FigmaLayoutSizingMode::Fill => {
                    style.height = None;
                    style.flex_grow = style.flex_grow.max(1.0);
                    style.align_self.get_or_insert(ItemAlignSelf::Stretch);
                }
                FigmaLayoutSizingMode::Unknown => {}
            }
        }

        style
    }

    pub(crate) fn to_constraints(&self) -> ImportedConstraints {
        let positioning = match self
            .layout_positioning
            .unwrap_or(FigmaLayoutPositioning::Auto)
        {
            FigmaLayoutPositioning::Absolute => LayoutPositioning::Absolute,
            FigmaLayoutPositioning::Auto | FigmaLayoutPositioning::Unknown => {
                LayoutPositioning::Auto
            }
        };
        let horizontal = self
            .constraints
            .as_ref()
            .and_then(|constraints| constraints.horizontal.as_deref())
            .map(figma_constraint_axis)
            .unwrap_or(ConstraintAxis::Min);
        let vertical = self
            .constraints
            .as_ref()
            .and_then(|constraints| constraints.vertical.as_deref())
            .map(figma_constraint_axis)
            .unwrap_or(ConstraintAxis::Min);

        ImportedConstraints {
            horizontal,
            vertical,
            positioning,
        }
    }

    pub(crate) fn to_visual_style(&self) -> VisualStyle {
        let mut style = VisualStyle::new();

        if matches!(self.node_type, Some(FigmaNodeType::Text))
            && let Some(text) = self.to_text_content()
        {
            style = style.text(text);
        }

        for fill in self.fills.iter().filter_map(FigmaPaint::to_paint) {
            style = style.fill(fill);
        }
        for filter_effect in self
            .fills
            .iter()
            .filter_map(FigmaPaint::to_color_filter_effect)
        {
            style = style.effect(filter_effect);
        }
        if let Some(fill_geometry) = &self.fill_geometry {
            let paths: Vec<_> = fill_geometry
                .iter()
                .filter_map(FigmaPathGeometry::to_vector_path)
                .collect();
            if !paths.is_empty() {
                style = style.fill_geometry(paths);
            }
        }

        if let Some(corner_radius) = self.corner_radius.filter(|value| value.is_finite()) {
            style = style.corner_radius(corner_radius.max(0.0));
        }
        if let Some([tl, tr, br, bl]) = self.rectangle_corner_radii {
            style = style.corner_radii(CornerRadii::new(
                tl.max(0.0),
                tr.max(0.0),
                br.max(0.0),
                bl.max(0.0),
            ));
        }
        if let Some(corner_smoothing) = self.corner_smoothing.filter(|value| value.is_finite()) {
            style = style.corner_smoothing(corner_smoothing);
        }

        for effect in self.effects.iter().filter_map(FigmaEffect::to_effect) {
            style = style.effect(effect);
        }

        if let Some(stroke) = self.to_stroke_style() {
            style = style.stroke(stroke);
        }
        if let Some(stroke_geometry) = &self.stroke_geometry {
            let paths: Vec<_> = stroke_geometry
                .iter()
                .filter_map(FigmaPathGeometry::to_vector_path)
                .collect();
            if !paths.is_empty() {
                style = style.stroke_geometry(paths);
            }
        }

        if let Some(blend_mode) = self.blend_mode {
            style = style.blend_mode(blend_mode.to_blend_mode());
        }
        if let Some(clips_content) = self.clips_content {
            style = style.clips_content(clips_content);
        }
        if let Some(is_mask) = self.is_mask {
            style = style.is_mask(is_mask);
        }
        if let Some(mask_type) = self.mask_type {
            style = style.mask_type(mask_type.to_mask_type());
        }

        style
    }

    pub(crate) fn to_stroke_style(&self) -> Option<StrokeStyle> {
        if self.strokes.is_empty() && self.stroke_weight.unwrap_or_default() <= 0.0 {
            return None;
        }

        let mut stroke = StrokeStyle {
            paints: self
                .strokes
                .iter()
                .filter_map(FigmaPaint::to_paint)
                .collect(),
            weight: self
                .stroke_weight
                .filter(|value| value.is_finite() && *value > 0.0)
                .unwrap_or(1.0),
            align: self
                .stroke_align
                .map(FigmaStrokeAlign::to_stroke_align)
                .unwrap_or(StrokeAlign::Center),
            cap: self
                .stroke_cap
                .map(FigmaStrokeCap::to_stroke_cap)
                .unwrap_or(StrokeCap::Butt),
            join: self
                .stroke_join
                .map(FigmaStrokeJoin::to_stroke_join)
                .unwrap_or(StrokeJoin::Miter),
            miter_limit: self
                .stroke_miter_limit
                .filter(|value| value.is_finite() && *value > 0.0)
                .or_else(|| {
                    self.stroke_miter_angle
                        .and_then(figma_stroke_miter_limit_from_angle)
                })
                .unwrap_or(StrokeStyle::default().miter_limit),
            dash_pattern: self.stroke_dashes.clone().unwrap_or_default(),
            dash_offset: self
                .dash_offset
                .filter(|value| value.is_finite())
                .unwrap_or(StrokeStyle::default().dash_offset),
            ..StrokeStyle::default()
        };
        if stroke.paints.is_empty() {
            stroke.paints.push(Paint::solid(Vec4::ONE));
        }
        if let Some(side_weights) = self.individual_stroke_weights {
            stroke.side_weights = Some(side_weights.to_side_weights());
        }
        Some(stroke)
    }

    pub(crate) fn to_text_content(&self) -> Option<TextContent> {
        let characters = self.characters.as_ref()?.clone();
        let style = self.style.as_ref();
        let is_icon_ligature_font = style
            .and_then(|s| s.font_family.as_deref())
            .is_some_and(is_ligature_icon_font_family);
        let source_text = if is_icon_ligature_font {
            material_icon_ligature_to_codepoint(&characters).unwrap_or_else(|| characters.clone())
        } else {
            characters.clone()
        };
        let text_case = style
            .and_then(|s| (!is_icon_ligature_font).then_some(s))
            .and_then(|s| s.text_case)
            .map(FigmaTextCase::to_text_case)
            .unwrap_or(TextCase::Original);
        let transformed = apply_text_case(&source_text, text_case);

        let font_size = style
            .and_then(|s| s.font_size)
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(16.0);
        let mut text = TextContent::new(transformed, font_size);

        text.font_weight = style
            .and_then(|s| s.font_weight)
            .map(|value| value.clamp(100, 900))
            .unwrap_or(400);
        text.font_style = map_font_style(style);
        text.align = style
            .and_then(|s| s.text_align_horizontal)
            .map(FigmaTextAlignHorizontal::to_text_align)
            .unwrap_or(TextAlign::Left);
        text.line_height = map_line_height(style);
        text.font_family = style.and_then(|s| s.font_family.clone());
        text.letter_spacing = style
            .and_then(|s| s.letter_spacing)
            .filter(|value| value.is_finite())
            .unwrap_or(0.0);
        text.decoration = style
            .and_then(|s| s.text_decoration)
            .map(FigmaTextDecoration::to_text_decoration)
            .unwrap_or(TextDecoration::None);
        text.text_case = text_case;
        text.align_vertical = style
            .and_then(|s| s.text_align_vertical)
            .map(FigmaTextAlignVertical::to_text_align_vertical)
            .unwrap_or(TextAlignVertical::Top);
        text.auto_resize = self
            .text_auto_resize
            .map(FigmaTextAutoResize::to_text_auto_resize)
            .unwrap_or(TextAutoResize::None);
        text.max_lines = self.max_lines.filter(|value| *value > 0);
        text.overflow = self
            .text_truncation
            .or_else(|| style.and_then(|s| s.text_truncation))
            .map(FigmaTextTruncation::to_text_overflow)
            .unwrap_or(TextOverflow::Clip);
        text.paragraph_spacing = style
            .and_then(|s| s.paragraph_spacing)
            .filter(|value| value.is_finite())
            .unwrap_or(0.0);
        text.paragraph_indent = style
            .and_then(|s| s.paragraph_indent)
            .filter(|value| value.is_finite())
            .unwrap_or(0.0);

        Some(text)
    }

    pub(crate) fn to_component_node(&self) -> Option<ImportedComponentNode> {
        match self.node_type.unwrap_or(FigmaNodeType::Unknown) {
            FigmaNodeType::Component => Some(ImportedComponentNode {
                kind: ImportedComponentKind::Component,
                key: self.key.clone(),
                component_set_id: self.component_set_id.as_ref().map(FigmaNodeKey::as_key),
            }),
            FigmaNodeType::ComponentSet => Some(ImportedComponentNode {
                kind: ImportedComponentKind::ComponentSet,
                key: self.key.clone(),
                component_set_id: None,
            }),
            _ => None,
        }
    }

    pub(crate) fn to_instance_node(&self) -> Option<ImportedInstanceNode> {
        if self.node_type != Some(FigmaNodeType::Instance) {
            return None;
        }

        let main_component_id = self
            .main_component_id
            .as_ref()
            .map(FigmaNodeKey::as_key)
            .or_else(|| {
                self.main_component
                    .as_ref()
                    .map(|component| component.id.as_key())
            });
        let component_id = self
            .component_id
            .as_ref()
            .map(FigmaNodeKey::as_key)
            .or_else(|| main_component_id.clone());

        Some(ImportedInstanceNode {
            component_id,
            main_component_id,
        })
    }

    pub(crate) fn to_variant_properties(&self) -> HashMap<String, String> {
        let mut variants = HashMap::new();

        for (name, value) in &self.variant_properties {
            let canonical = canonical_property_name(name);
            if !canonical.is_empty() {
                variants.insert(canonical, value.as_string());
            }
        }

        for (name, property) in &self.component_properties {
            if property.property_type != Some(FigmaComponentPropertyType::Variant) {
                continue;
            }
            let Some(value) = property.value.as_ref() else {
                continue;
            };
            let canonical = canonical_property_name(name);
            if !canonical.is_empty() {
                variants.insert(canonical, value.as_string());
            }
        }

        variants
    }

    pub(crate) fn to_component_property_definitions(
        &self,
    ) -> HashMap<String, ImportedComponentPropertyDefinition> {
        let mut definitions = HashMap::new();
        for (name, definition) in &self.component_property_definitions {
            let canonical = canonical_property_name(name);
            if canonical.is_empty() {
                continue;
            }
            definitions.insert(
                canonical,
                ImportedComponentPropertyDefinition {
                    property_type: definition
                        .property_type
                        .map(FigmaComponentPropertyType::to_public)
                        .unwrap_or(ImportedComponentPropertyType::Unknown),
                    default_value: definition
                        .default_value
                        .as_ref()
                        .map(FigmaPropertyValue::to_imported_value),
                    preferred_values: definition
                        .preferred_values
                        .iter()
                        .map(FigmaPropertyValue::to_imported_value)
                        .collect(),
                },
            );
        }
        definitions
    }

    pub(crate) fn to_instance_property_overrides(
        &self,
    ) -> HashMap<String, ImportedComponentPropertyOverride> {
        let mut overrides = HashMap::new();
        for (name, property) in &self.component_properties {
            let Some(value) = property.value.as_ref() else {
                continue;
            };
            let canonical = canonical_property_name(name);
            if canonical.is_empty() {
                continue;
            }
            overrides.insert(
                canonical,
                ImportedComponentPropertyOverride {
                    property_type: property
                        .property_type
                        .map(FigmaComponentPropertyType::to_public)
                        .unwrap_or(ImportedComponentPropertyType::Unknown),
                    value: value.to_imported_value(),
                },
            );
        }
        overrides
    }
}

pub(crate) fn map_font_style(style: Option<&FigmaTypeStyle>) -> FontStyle {
    let Some(style) = style else {
        return FontStyle::Normal;
    };
    if style.italic.unwrap_or(false) {
        return FontStyle::Italic;
    }
    if style
        .font_style
        .as_deref()
        .is_some_and(|value| value.to_ascii_lowercase().contains("italic"))
    {
        return FontStyle::Italic;
    }
    FontStyle::Normal
}

pub(crate) fn map_line_height(style: Option<&FigmaTypeStyle>) -> LineHeight {
    let Some(style) = style else {
        return LineHeight::Auto;
    };
    if let Some(px) = style
        .line_height_px
        .filter(|value| value.is_finite() && *value > 0.0)
    {
        return LineHeight::Fixed(px);
    }
    if let Some(percent) = style
        .line_height_percent_font_size
        .filter(|value| value.is_finite() && *value > 0.0)
    {
        return LineHeight::Relative(percent / 100.0);
    }
    LineHeight::Auto
}

pub(crate) fn figma_constraint_axis(axis: &str) -> ConstraintAxis {
    let normalized = axis.trim().to_ascii_uppercase();
    match normalized.as_str() {
        "MIN" | "LEFT" | "TOP" => ConstraintAxis::Min,
        "CENTER" => ConstraintAxis::Center,
        "MAX" | "RIGHT" | "BOTTOM" => ConstraintAxis::Max,
        "STRETCH" | "LEFT_RIGHT" | "TOP_BOTTOM" => ConstraintAxis::Stretch,
        "SCALE" => ConstraintAxis::Scale,
        _ => ConstraintAxis::Min,
    }
}

pub(crate) fn canonical_property_name(name: &str) -> String {
    let trimmed = name.trim();
    let canonical = trimmed
        .split_once('#')
        .map_or(trimmed, |(base, _)| base)
        .trim();
    if canonical.is_empty() {
        return trimmed.to_string();
    }
    canonical.to_string()
}

pub(crate) fn duration_to_ms(value: f64) -> Option<u32> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let ms = if value <= 10.0 { value * 1000.0 } else { value };
    let rounded = ms.round();
    if rounded > u32::MAX as f64 {
        return None;
    }
    Some(rounded as u32)
}

pub(crate) fn apply_text_case(text: &str, text_case: TextCase) -> String {
    match text_case {
        TextCase::Original => text.to_string(),
        TextCase::Upper | TextCase::SmallCaps | TextCase::SmallCapsForced => text.to_uppercase(),
        TextCase::Lower => text.to_lowercase(),
        TextCase::Title => text
            .split_whitespace()
            .map(title_case_word)
            .collect::<Vec<_>>()
            .join(" "),
    }
}

pub(crate) fn title_case_word(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum FigmaNodeKey {
    Text(String),
    Numeric(u64),
}

impl FigmaNodeKey {
    pub(crate) fn as_key(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Numeric(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaNodeType {
    Frame,
    Group,
    Rectangle,
    Text,
    Component,
    ComponentSet,
    Instance,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaLayoutMode {
    #[serde(rename = "NONE")]
    None,
    Horizontal,
    Vertical,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaPrimaryAxisAlignItems {
    Min,
    Center,
    Max,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    #[serde(other)]
    Unknown,
}

impl FigmaPrimaryAxisAlignItems {
    pub(crate) fn to_justify_content(self) -> FlexJustifyContent {
        match self {
            Self::Min => FlexJustifyContent::Start,
            Self::Center => FlexJustifyContent::Center,
            Self::Max => FlexJustifyContent::End,
            Self::SpaceBetween => FlexJustifyContent::SpaceBetween,
            Self::SpaceAround => FlexJustifyContent::SpaceAround,
            Self::SpaceEvenly => FlexJustifyContent::SpaceEvenly,
            Self::Unknown => FlexJustifyContent::Start,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaCounterAxisAlignItems {
    Min,
    Center,
    Max,
    Baseline,
    #[serde(other)]
    Unknown,
}

impl FigmaCounterAxisAlignItems {
    pub(crate) fn to_align(self) -> FlexAlign {
        match self {
            Self::Min => FlexAlign::Start,
            Self::Center => FlexAlign::Center,
            Self::Max => FlexAlign::End,
            Self::Baseline => FlexAlign::Start,
            Self::Unknown => FlexAlign::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaLayoutWrap {
    NoWrap,
    Wrap,
    #[serde(other)]
    Unknown,
}

impl FigmaLayoutWrap {
    pub(crate) fn to_wrap(self) -> FlexWrap {
        match self {
            Self::NoWrap => FlexWrap::NoWrap,
            Self::Wrap => FlexWrap::Wrap,
            Self::Unknown => FlexWrap::NoWrap,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaLayoutAlign {
    Inherit,
    Stretch,
    Min,
    Center,
    Max,
    #[serde(rename = "AUTO")]
    Auto,
    #[serde(other)]
    Unknown,
}

impl FigmaLayoutAlign {
    pub(crate) fn to_align_self(self) -> Option<ItemAlignSelf> {
        match self {
            Self::Inherit | Self::Auto | Self::Unknown => None,
            Self::Stretch => Some(ItemAlignSelf::Stretch),
            Self::Min => Some(ItemAlignSelf::Start),
            Self::Center => Some(ItemAlignSelf::Center),
            Self::Max => Some(ItemAlignSelf::End),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaLayoutSizingMode {
    Fill,
    Hug,
    Fixed,
    Auto,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaAxisSizingMode {
    Fixed,
    Auto,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaLayoutPositioning {
    Auto,
    Absolute,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaConstraints {
    #[serde(default)]
    pub(crate) horizontal: Option<String>,
    #[serde(default)]
    pub(crate) vertical: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaMainComponent {
    pub(crate) id: FigmaNodeKey,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaComponentProperty {
    #[serde(default, rename = "type")]
    pub(crate) property_type: Option<FigmaComponentPropertyType>,
    #[serde(default)]
    pub(crate) value: Option<FigmaPropertyValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaComponentPropertyDefinition {
    #[serde(default, rename = "type")]
    pub(crate) property_type: Option<FigmaComponentPropertyType>,
    #[serde(default, alias = "defaultValue")]
    pub(crate) default_value: Option<FigmaPropertyValue>,
    #[serde(default, alias = "preferredValues")]
    pub(crate) preferred_values: Vec<FigmaPropertyValue>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaComponentPropertyType {
    Variant,
    Boolean,
    Text,
    InstanceSwap,
    #[serde(other)]
    Unknown,
}

impl FigmaComponentPropertyType {
    pub(crate) fn to_public(self) -> ImportedComponentPropertyType {
        match self {
            Self::Variant => ImportedComponentPropertyType::Variant,
            Self::Boolean => ImportedComponentPropertyType::Boolean,
            Self::Text => ImportedComponentPropertyType::Text,
            Self::InstanceSwap => ImportedComponentPropertyType::InstanceSwap,
            Self::Unknown => ImportedComponentPropertyType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum FigmaPropertyValue {
    Text(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
    Node(FigmaPropertyNodeRef),
}

impl FigmaPropertyValue {
    pub(crate) fn as_string(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Bool(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::Node(node) => node
                .id
                .as_ref()
                .map(FigmaNodeKey::as_key)
                .or_else(|| node.node_id.as_ref().map(FigmaNodeKey::as_key))
                .unwrap_or_default(),
        }
    }

    pub(crate) fn to_imported_value(&self) -> ImportedComponentPropertyValue {
        match self {
            Self::Text(value) => ImportedComponentPropertyValue::Text(value.clone()),
            Self::Bool(value) => ImportedComponentPropertyValue::Bool(*value),
            Self::Integer(value) => ImportedComponentPropertyValue::Number(*value as f64),
            Self::Float(value) => ImportedComponentPropertyValue::Number(*value),
            Self::Node(node) => ImportedComponentPropertyValue::NodeRef(
                node.id
                    .as_ref()
                    .map(FigmaNodeKey::as_key)
                    .or_else(|| node.node_id.as_ref().map(FigmaNodeKey::as_key))
                    .unwrap_or_default(),
            ),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaPropertyNodeRef {
    #[serde(default)]
    pub(crate) id: Option<FigmaNodeKey>,
    #[serde(default, alias = "nodeId")]
    pub(crate) node_id: Option<FigmaNodeKey>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum FigmaPathGeometry {
    SvgPathData(String),
    PathObject(FigmaPathGeometryObject),
    Unsupported(JsonValue),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaPathGeometryObject {
    #[serde(alias = "pathData")]
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) winding_rule: Option<FigmaWindingRule>,
}

impl FigmaPathGeometry {
    pub(crate) fn to_vector_path(&self) -> Option<VectorPath> {
        let (path_data, winding_rule) = match self {
            Self::SvgPathData(path) => (path.as_str(), None),
            Self::PathObject(object) => (
                object.path.as_str(),
                object.winding_rule.map(FigmaWindingRule::to_winding_rule),
            ),
            Self::Unsupported(value) => {
                let _ = value;
                return None;
            }
        };
        let mut path = VectorPath::from_svg_path_data(path_data).ok()?;
        if let Some(winding_rule) = winding_rule {
            path.winding_rule = winding_rule;
        }
        Some(path)
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) enum FigmaWindingRule {
    #[serde(rename = "NONZERO")]
    NonZero,
    #[serde(rename = "EVENODD", alias = "EVEN_ODD")]
    EvenOdd,
    #[serde(rename = "NONE")]
    None,
}

impl FigmaWindingRule {
    pub(crate) fn to_winding_rule(self) -> WindingRule {
        match self {
            Self::NonZero => WindingRule::NonZero,
            Self::EvenOdd => WindingRule::EvenOdd,
            Self::None => WindingRule::NonZero,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaBlendMode {
    Normal,
    Darken,
    Multiply,
    ColorBurn,
    Lighten,
    Screen,
    ColorDodge,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    LinearBurn,
    LinearDodge,
    PassThrough,
    #[serde(other)]
    Unknown,
}

impl FigmaBlendMode {
    pub(crate) fn to_blend_mode(self) -> BlendMode {
        match self {
            Self::Normal => BlendMode::Normal,
            Self::Darken => BlendMode::Darken,
            Self::Multiply => BlendMode::Multiply,
            Self::ColorBurn => BlendMode::ColorBurn,
            Self::Lighten => BlendMode::Lighten,
            Self::Screen => BlendMode::Screen,
            Self::ColorDodge => BlendMode::ColorDodge,
            Self::Overlay => BlendMode::Overlay,
            Self::SoftLight => BlendMode::SoftLight,
            Self::HardLight => BlendMode::HardLight,
            Self::Difference => BlendMode::Difference,
            Self::Exclusion => BlendMode::Exclusion,
            Self::Hue => BlendMode::Hue,
            Self::Saturation => BlendMode::Saturation,
            Self::Color => BlendMode::Color,
            Self::Luminosity => BlendMode::Luminosity,
            Self::LinearBurn => BlendMode::LinearBurn,
            Self::LinearDodge => BlendMode::LinearDodge,
            Self::PassThrough => BlendMode::PassThrough,
            Self::Unknown => BlendMode::Normal,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum FigmaColorValue {
    Vec4([f32; 4]),
    Vec3([f32; 3]),
    Object(FigmaColorObject),
    Hex(String),
}

impl FigmaColorValue {
    pub(crate) fn to_vec4(&self) -> Option<Vec4> {
        match self {
            Self::Vec4([r, g, b, a]) => Some(Vec4::new(
                normalize_color_component(*r),
                normalize_color_component(*g),
                normalize_color_component(*b),
                normalize_alpha_component(*a),
            )),
            Self::Vec3([r, g, b]) => Some(Vec4::new(
                normalize_color_component(*r),
                normalize_color_component(*g),
                normalize_color_component(*b),
                1.0,
            )),
            Self::Object(color) => Some(Vec4::new(
                normalize_color_component(color.r),
                normalize_color_component(color.g),
                normalize_color_component(color.b),
                color
                    .a
                    .or(color.alpha)
                    .map(normalize_alpha_component)
                    .unwrap_or(1.0),
            )),
            Self::Hex(hex) => parse_hex_color(hex),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaColorObject {
    pub(crate) r: f32,
    pub(crate) g: f32,
    pub(crate) b: f32,
    #[serde(default)]
    pub(crate) a: Option<f32>,
    #[serde(default)]
    pub(crate) alpha: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FigmaColorStop {
    pub(crate) position: f32,
    pub(crate) color: FigmaColorValue,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaImageScaleMode {
    Fill,
    Fit,
    Crop,
    Tile,
    Stretch,
}

impl FigmaImageScaleMode {
    pub(crate) fn to_scale_mode(self) -> ImageScaleMode {
        match self {
            Self::Fill => ImageScaleMode::Fill,
            Self::Fit => ImageScaleMode::Fit,
            Self::Crop => ImageScaleMode::Crop,
            Self::Tile => ImageScaleMode::Tile,
            Self::Stretch => ImageScaleMode::Stretch,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum FigmaImageIdValue {
    Numeric(u64),
    Text(String),
    HashBytes(Vec<u8>),
}

impl FigmaImageIdValue {
    pub(crate) fn to_image_id(&self) -> ImageId {
        match self {
            Self::Numeric(id) => ImageId(*id),
            Self::Text(reference) => ImageId(figma_image_reference_to_id(reference)),
            Self::HashBytes(bytes) => {
                let mut hash = String::with_capacity(bytes.len() * 2);
                for byte in bytes {
                    let _ = write!(&mut hash, "{byte:02x}");
                }
                ImageId(figma_image_reference_to_id(&hash))
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaImageDescriptor {
    #[serde(default)]
    pub(crate) hash: Option<Vec<u8>>,
    #[serde(default, alias = "filename", alias = "name")]
    pub(crate) reference: Option<String>,
}

impl FigmaImageDescriptor {
    pub(crate) fn to_image_id_value(&self) -> Option<FigmaImageIdValue> {
        if let Some(hash) = self.hash.as_ref().filter(|hash| !hash.is_empty()) {
            return Some(FigmaImageIdValue::HashBytes(hash.clone()));
        }
        self.reference
            .as_ref()
            .map(|reference| FigmaImageIdValue::Text(reference.clone()))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum FigmaImageTransform {
    Object(FigmaImageTransformObject),
    Matrix3x3([f32; 9]),
    Rows2x3([[f32; 3]; 2]),
    Rows3x3([[f32; 3]; 3]),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaImageTransformObject {
    pub(crate) m00: f32,
    pub(crate) m01: f32,
    pub(crate) m02: f32,
    pub(crate) m10: f32,
    pub(crate) m11: f32,
    pub(crate) m12: f32,
}

impl FigmaImageTransform {
    pub(crate) fn to_matrix3x3(&self) -> [f32; 9] {
        match self {
            Self::Object(object) => [
                object.m00, object.m01, object.m02, object.m10, object.m11, object.m12, 0.0, 0.0,
                1.0,
            ],
            Self::Matrix3x3(matrix) => *matrix,
            Self::Rows2x3([[a, b, tx], [c, d, ty]]) => [*a, *b, *tx, *c, *d, *ty, 0.0, 0.0, 1.0],
            Self::Rows3x3([row0, row1, row2]) => [
                row0[0], row0[1], row0[2], row1[0], row1[1], row1[2], row2[0], row2[1], row2[2],
            ],
        }
    }
}

pub(crate) fn is_ligature_icon_font_family(font_family: &str) -> bool {
    let normalized = font_family.trim().to_ascii_lowercase();
    normalized.contains("material symbols") || normalized.contains("material icons")
}

pub(crate) fn material_icon_ligature_to_codepoint(text: &str) -> Option<String> {
    let normalized = text.trim();
    let codepoint = match normalized {
        // Riot Waves / Stitch icon set used in current parity targets.
        "skull" => 0xF89A,
        "graphic_eq" => 0xE1B8,
        "inventory_2" => 0xE1A1,
        "cable" => 0xEFE6,
        "deck" => 0xEA42,
        "search" => 0xE8B6,
        "pause" => 0xE034,
        "shuffle" => 0xE043,
        _ => return None,
    };
    char::from_u32(codepoint).map(|glyph| glyph.to_string())
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaImageFilter {
    #[serde(default)]
    pub(crate) grayscale: Option<f32>,
    #[serde(default)]
    pub(crate) contrast: Option<f32>,
    #[serde(default)]
    pub(crate) invert: Option<f32>,
}

impl FigmaImageFilter {
    pub(crate) fn to_color_filter(self) -> Option<ColorFilter> {
        let grayscale = self
            .grayscale
            .filter(|value| value.is_finite())
            .map_or(0.0, |value| value.clamp(0.0, 1.0));
        let contrast = self
            .contrast
            .filter(|value| value.is_finite())
            .map_or(1.0, |value| value.max(0.0));
        let invert = self
            .invert
            .filter(|value| value.is_finite())
            .map_or(0.0, |value| value.clamp(0.0, 1.0));

        let is_identity = grayscale <= f32::EPSILON
            && (contrast - 1.0).abs() <= f32::EPSILON
            && invert <= f32::EPSILON;
        if is_identity {
            return None;
        }

        Some(ColorFilter {
            grayscale,
            contrast,
            invert,
            visible: true,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaPaint {
    Solid {
        color: FigmaColorValue,
        #[serde(default)]
        opacity: Option<f32>,
        #[serde(default = "default_visible")]
        visible: bool,
    },
    GradientLinear {
        #[serde(default)]
        start: Option<[f32; 2]>,
        #[serde(default)]
        end: Option<[f32; 2]>,
        #[serde(default)]
        stops: Vec<FigmaColorStop>,
        #[serde(default, alias = "gradientHandlePositions")]
        gradient_handle_positions: Option<Vec<[f32; 2]>>,
        #[serde(default = "default_visible")]
        visible: bool,
    },
    GradientRadial {
        #[serde(default)]
        center: Option<[f32; 2]>,
        #[serde(default)]
        radius: Option<f32>,
        #[serde(default)]
        stops: Vec<FigmaColorStop>,
        #[serde(default, alias = "gradientHandlePositions")]
        gradient_handle_positions: Option<Vec<[f32; 2]>>,
        #[serde(default = "default_visible")]
        visible: bool,
    },
    Image {
        #[serde(default, alias = "imageId", alias = "imageRef", alias = "imageHash")]
        image_id: Option<FigmaImageIdValue>,
        #[serde(default)]
        image: Option<FigmaImageDescriptor>,
        #[serde(default, alias = "scaleMode", alias = "imageScaleMode")]
        scale_mode: Option<FigmaImageScaleMode>,
        #[serde(default, alias = "scalingFactor")]
        scaling_factor: Option<f32>,
        #[serde(default, alias = "imageTransform")]
        transform: Option<FigmaImageTransform>,
        #[serde(default, alias = "imageFilter")]
        image_filter: Option<FigmaImageFilter>,
        #[serde(default = "default_visible")]
        visible: bool,
    },
    #[serde(other)]
    Unsupported,
}

impl FigmaPaint {
    pub(crate) fn to_paint(&self) -> Option<Paint> {
        match self {
            Self::Solid {
                color,
                opacity,
                visible,
            } => {
                if !*visible {
                    return None;
                }
                let mut c = color.to_vec4()?;
                if let Some(opacity) = opacity {
                    c.w *= normalize_alpha_component(*opacity);
                }
                Some(Paint::solid(c))
            }
            Self::GradientLinear {
                start,
                end,
                stops,
                gradient_handle_positions,
                visible,
            } => {
                if !*visible {
                    return None;
                }
                let (start, end) = if let (Some(start), Some(end)) = (start, end) {
                    (*start, *end)
                } else if let Some(handles) = gradient_handle_positions {
                    match handles.as_slice() {
                        [start, end, ..] => (*start, *end),
                        _ => return None,
                    }
                } else {
                    return None;
                };
                let stops = map_color_stops(stops);
                if stops.is_empty() {
                    return None;
                }
                Some(Paint::Linear(LinearGradient {
                    start: Vec2::new(start[0], start[1]),
                    end: Vec2::new(end[0], end[1]),
                    stops,
                }))
            }
            Self::GradientRadial {
                center,
                radius,
                stops,
                gradient_handle_positions,
                visible,
            } => {
                if !*visible {
                    return None;
                }
                let center = if let Some(center) = center {
                    *center
                } else if let Some(handles) = gradient_handle_positions {
                    match handles.as_slice() {
                        [center, ..] => *center,
                        _ => return None,
                    }
                } else {
                    return None;
                };
                let radius = radius
                    .filter(|value| value.is_finite() && *value >= 0.0)
                    .or_else(|| {
                        gradient_handle_positions.as_ref().and_then(|handles| {
                            if handles.len() >= 2 {
                                let dx = handles[1][0] - center[0];
                                let dy = handles[1][1] - center[1];
                                Some((dx * dx + dy * dy).sqrt())
                            } else {
                                None
                            }
                        })
                    })?;
                let stops = map_color_stops(stops);
                if stops.is_empty() {
                    return None;
                }
                Some(Paint::Radial(RadialGradient {
                    center: Vec2::new(center[0], center[1]),
                    radius,
                    stops,
                }))
            }
            Self::Image {
                image_id,
                image,
                scale_mode,
                scaling_factor,
                transform,
                image_filter: _,
                visible,
            } => {
                if !*visible {
                    return None;
                }
                let image_id = image_id.as_ref().cloned().or_else(|| {
                    image
                        .as_ref()
                        .and_then(FigmaImageDescriptor::to_image_id_value)
                })?;
                let scale_mode = scale_mode.unwrap_or(FigmaImageScaleMode::Fill);
                Some(Paint::Image(ImageFill {
                    image_id: image_id.to_image_id(),
                    scale_mode: scale_mode.to_scale_mode(),
                    transform: figma_image_transform(
                        scale_mode,
                        transform.as_ref(),
                        *scaling_factor,
                    ),
                }))
            }
            Self::Unsupported => None,
        }
    }

    pub(crate) fn to_color_filter_effect(&self) -> Option<Effect> {
        let Self::Image {
            image_filter,
            visible,
            ..
        } = self
        else {
            return None;
        };
        if !*visible {
            return None;
        }
        image_filter
            .and_then(|filter| filter.to_color_filter())
            .map(Effect::ColorFilter)
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaStrokeAlign {
    Inside,
    Center,
    Outside,
}

impl FigmaStrokeAlign {
    pub(crate) fn to_stroke_align(self) -> StrokeAlign {
        match self {
            Self::Inside => StrokeAlign::Inside,
            Self::Center => StrokeAlign::Center,
            Self::Outside => StrokeAlign::Outside,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaStrokeCap {
    None,
    Round,
    Square,
    LineArrow,
    TriangleArrow,
    DiamondFilled,
    CircleFilled,
}

impl FigmaStrokeCap {
    pub(crate) fn to_stroke_cap(self) -> StrokeCap {
        match self {
            Self::None => StrokeCap::Butt,
            Self::Round | Self::CircleFilled => StrokeCap::Round,
            Self::Square => StrokeCap::Square,
            Self::LineArrow | Self::TriangleArrow | Self::DiamondFilled => StrokeCap::Butt,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaStrokeJoin {
    Miter,
    Round,
    Bevel,
}

impl FigmaStrokeJoin {
    pub(crate) fn to_stroke_join(self) -> StrokeJoin {
        match self {
            Self::Miter => StrokeJoin::Miter,
            Self::Round => StrokeJoin::Round,
            Self::Bevel => StrokeJoin::Bevel,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaMaskType {
    Alpha,
    Vector,
    Luminance,
}

impl FigmaMaskType {
    pub(crate) fn to_mask_type(self) -> MaskType {
        match self {
            Self::Alpha => MaskType::Alpha,
            Self::Vector => MaskType::Vector,
            Self::Luminance => MaskType::Luminance,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaSideWeights {
    pub(crate) top: f32,
    pub(crate) right: f32,
    pub(crate) bottom: f32,
    pub(crate) left: f32,
}

impl FigmaSideWeights {
    pub(crate) fn to_side_weights(self) -> SideWeights {
        SideWeights {
            top: self.top.max(0.0),
            right: self.right.max(0.0),
            bottom: self.bottom.max(0.0),
            left: self.left.max(0.0),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaEffect {
    DropShadow {
        offset: [f32; 2],
        radius: f32,
        color: FigmaColorValue,
        #[serde(default = "default_visible")]
        visible: bool,
    },
    InnerShadow {
        offset: [f32; 2],
        radius: f32,
        color: FigmaColorValue,
        #[serde(default = "default_visible")]
        visible: bool,
    },
    LayerBlur {
        radius: f32,
        #[serde(default = "default_visible")]
        visible: bool,
    },
    BackgroundBlur {
        radius: f32,
        #[serde(default = "default_visible")]
        visible: bool,
    },
    #[serde(other)]
    Unsupported,
}

impl FigmaEffect {
    pub(crate) fn to_effect(&self) -> Option<Effect> {
        match self {
            Self::DropShadow {
                offset,
                radius,
                color,
                visible,
            } => (*visible).then(|| {
                Effect::DropShadow(DropShadow {
                    offset: Vec2::new(offset[0], offset[1]),
                    blur: *radius,
                    color: color.to_vec4().unwrap_or(Vec4::ZERO),
                    visible: true,
                })
            }),
            Self::InnerShadow {
                offset,
                radius,
                color,
                visible,
            } => (*visible).then(|| {
                Effect::InnerShadow(InnerShadow {
                    offset: Vec2::new(offset[0], offset[1]),
                    blur: *radius,
                    color: color.to_vec4().unwrap_or(Vec4::ZERO),
                    visible: true,
                })
            }),
            Self::LayerBlur { radius, visible } => {
                (*visible).then_some(Effect::LayerBlur(LayerBlur {
                    radius: *radius,
                    visible: true,
                }))
            }
            Self::BackgroundBlur { radius, visible } => {
                (*visible).then_some(Effect::BackgroundBlur(BackgroundBlur {
                    radius: *radius,
                    visible: true,
                }))
            }
            Self::Unsupported => None,
        }
    }
}

const fn default_visible() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaTypeStyle {
    #[serde(default)]
    pub(crate) font_family: Option<String>,
    #[serde(default)]
    pub(crate) font_style: Option<String>,
    #[serde(default)]
    pub(crate) font_weight: Option<u16>,
    #[serde(default)]
    pub(crate) font_size: Option<f32>,
    #[serde(default)]
    pub(crate) italic: Option<bool>,
    #[serde(default)]
    pub(crate) text_align_horizontal: Option<FigmaTextAlignHorizontal>,
    #[serde(default)]
    pub(crate) line_height_px: Option<f32>,
    #[serde(default)]
    pub(crate) line_height_percent_font_size: Option<f32>,
    #[serde(default)]
    pub(crate) letter_spacing: Option<f32>,
    #[serde(default)]
    pub(crate) text_decoration: Option<FigmaTextDecoration>,
    #[serde(default)]
    pub(crate) text_case: Option<FigmaTextCase>,
    #[serde(default)]
    pub(crate) text_align_vertical: Option<FigmaTextAlignVertical>,
    #[serde(default)]
    pub(crate) paragraph_spacing: Option<f32>,
    #[serde(default)]
    pub(crate) paragraph_indent: Option<f32>,
    #[serde(default)]
    pub(crate) text_truncation: Option<FigmaTextTruncation>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaTextAlignHorizontal {
    Left,
    Center,
    Right,
    Justified,
    #[serde(other)]
    Unknown,
}

impl FigmaTextAlignHorizontal {
    pub(crate) fn to_text_align(self) -> TextAlign {
        match self {
            Self::Left => TextAlign::Left,
            Self::Center => TextAlign::Center,
            Self::Right => TextAlign::Right,
            Self::Justified => TextAlign::Justified,
            Self::Unknown => TextAlign::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaTextDecoration {
    None,
    Underline,
    Strikethrough,
    #[serde(other)]
    Unknown,
}

impl FigmaTextDecoration {
    pub(crate) fn to_text_decoration(self) -> TextDecoration {
        match self {
            Self::None => TextDecoration::None,
            Self::Underline => TextDecoration::Underline,
            Self::Strikethrough => TextDecoration::LineThrough,
            Self::Unknown => TextDecoration::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaTextCase {
    Original,
    Upper,
    Lower,
    Title,
    SmallCaps,
    SmallCapsForced,
    #[serde(other)]
    Unknown,
}

impl FigmaTextCase {
    pub(crate) fn to_text_case(self) -> TextCase {
        match self {
            Self::Original => TextCase::Original,
            Self::Upper => TextCase::Upper,
            Self::Lower => TextCase::Lower,
            Self::Title => TextCase::Title,
            Self::SmallCaps => TextCase::SmallCaps,
            Self::SmallCapsForced => TextCase::SmallCapsForced,
            Self::Unknown => TextCase::Original,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaTextAlignVertical {
    Top,
    Center,
    Bottom,
    #[serde(other)]
    Unknown,
}

impl FigmaTextAlignVertical {
    pub(crate) fn to_text_align_vertical(self) -> TextAlignVertical {
        match self {
            Self::Top => TextAlignVertical::Top,
            Self::Center => TextAlignVertical::Center,
            Self::Bottom => TextAlignVertical::Bottom,
            Self::Unknown => TextAlignVertical::Top,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaTextAutoResize {
    None,
    WidthAndHeight,
    Height,
    Width,
    Truncate,
    #[serde(other)]
    Unknown,
}

impl FigmaTextAutoResize {
    pub(crate) fn to_text_auto_resize(self) -> TextAutoResize {
        match self {
            Self::None => TextAutoResize::None,
            Self::WidthAndHeight => TextAutoResize::WidthAndHeight,
            Self::Height => TextAutoResize::Height,
            Self::Width => TextAutoResize::Width,
            Self::Truncate => TextAutoResize::Truncate,
            Self::Unknown => TextAutoResize::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaTextTruncation {
    None,
    Disabled,
    Ending,
    #[serde(other)]
    Unknown,
}

impl FigmaTextTruncation {
    pub(crate) fn to_text_overflow(self) -> TextOverflow {
        match self {
            Self::Ending => TextOverflow::Ellipsis,
            Self::None | Self::Disabled | Self::Unknown => TextOverflow::Clip,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaPrototypeInteraction {
    #[serde(default)]
    pub(crate) trigger: Option<FigmaPrototypeTriggerInput>,
    #[serde(
        default,
        alias = "destinationId",
        alias = "targetId",
        alias = "transitionNodeID",
        alias = "transitionNodeId"
    )]
    pub(crate) destination_id: Option<FigmaNodeKey>,
    #[serde(default)]
    pub(crate) actions: Vec<FigmaPrototypeAction>,
    #[serde(default, rename = "action")]
    pub(crate) action_type: Option<FigmaPrototypeActionType>,
    #[serde(default)]
    pub(crate) navigation: Option<FigmaPrototypeActionType>,
    #[serde(default)]
    pub(crate) transition: Option<FigmaPrototypeTransition>,
    #[serde(default)]
    pub(crate) preserve_scroll_position: Option<bool>,
    #[serde(default)]
    pub(crate) url: Option<String>,
}

impl FigmaPrototypeInteraction {
    pub(crate) fn trigger_spec(&self) -> Option<(PrototypeTrigger, Option<u32>)> {
        self.trigger
            .as_ref()
            .and_then(FigmaPrototypeTriggerInput::to_public)
    }

    pub(crate) fn transition_details(&self) -> Option<PrototypeTransition> {
        self.transition
            .as_ref()
            .and_then(FigmaPrototypeTransition::to_public)
    }

    pub(crate) fn to_legacy_edge(
        &self,
        from: NodeId,
        trigger: PrototypeTrigger,
        trigger_timeout_ms: Option<u32>,
        inherited_transition: Option<PrototypeTransition>,
        inherited_preserve_scroll: bool,
    ) -> Option<PrototypeEdge> {
        let action = self.legacy_action_kind();
        let to_figma_id = match action {
            PrototypeActionKind::Back | PrototypeActionKind::CloseOverlay => None,
            _ => self.destination_id.as_ref().map(FigmaNodeKey::as_key),
        };
        let url = self.url.clone();

        if action == PrototypeActionKind::Unknown && to_figma_id.is_none() && url.is_none() {
            return None;
        }

        Some(PrototypeEdge {
            from,
            to_figma_id,
            trigger,
            trigger_timeout_ms,
            action,
            preserve_scroll_position: inherited_preserve_scroll,
            transition: inherited_transition,
            overlay: None,
            url,
        })
    }

    pub(crate) fn legacy_action_kind(&self) -> PrototypeActionKind {
        if let Some(action) = self
            .action_type
            .or(self.navigation)
            .map(FigmaPrototypeActionType::to_public)
        {
            return action;
        }
        if self.url.is_some() {
            return PrototypeActionKind::Url;
        }
        if self.destination_id.is_some() {
            return PrototypeActionKind::Navigate;
        }
        PrototypeActionKind::Unknown
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaPrototypeAction {
    #[serde(default, rename = "type")]
    pub(crate) action_type: Option<FigmaPrototypeActionType>,
    #[serde(
        default,
        alias = "destinationId",
        alias = "targetId",
        alias = "nodeId",
        alias = "transitionNodeID",
        alias = "transitionNodeId"
    )]
    pub(crate) destination_id: Option<FigmaNodeKey>,
    #[serde(default)]
    pub(crate) transition: Option<FigmaPrototypeTransition>,
    #[serde(default)]
    pub(crate) preserve_scroll_position: Option<bool>,
    #[serde(default)]
    pub(crate) overlay_position_type: Option<FigmaOverlayPositionType>,
    #[serde(default)]
    pub(crate) overlay_background_interaction: Option<FigmaOverlayBackgroundInteraction>,
    #[serde(default)]
    pub(crate) overlay_relative_position: Option<FigmaVector2>,
    #[serde(default)]
    pub(crate) url: Option<String>,
}

impl FigmaPrototypeAction {
    pub(crate) fn to_edge(
        &self,
        from: NodeId,
        trigger: PrototypeTrigger,
        trigger_timeout_ms: Option<u32>,
        inherited_transition: Option<PrototypeTransition>,
        inherited_preserve_scroll: bool,
    ) -> Option<PrototypeEdge> {
        let action = self
            .action_type
            .map(FigmaPrototypeActionType::to_public)
            .unwrap_or_else(|| {
                if self.url.is_some() {
                    PrototypeActionKind::Url
                } else if self.destination_id.is_some() {
                    PrototypeActionKind::Navigate
                } else {
                    PrototypeActionKind::Unknown
                }
            });

        let to_figma_id = match action {
            PrototypeActionKind::Back
            | PrototypeActionKind::CloseOverlay
            | PrototypeActionKind::Url => None,
            _ => self.destination_id.as_ref().map(FigmaNodeKey::as_key),
        };
        let url = self.url.clone();
        if action == PrototypeActionKind::Unknown && to_figma_id.is_none() && url.is_none() {
            return None;
        }

        let transition = self
            .transition
            .as_ref()
            .and_then(FigmaPrototypeTransition::to_public)
            .or(inherited_transition);

        Some(PrototypeEdge {
            from,
            to_figma_id,
            trigger,
            trigger_timeout_ms,
            action,
            preserve_scroll_position: self
                .preserve_scroll_position
                .unwrap_or(inherited_preserve_scroll),
            transition,
            overlay: self.overlay_config(),
            url,
        })
    }

    pub(crate) fn overlay_config(&self) -> Option<PrototypeOverlayConfig> {
        let position = self
            .overlay_position_type
            .and_then(FigmaOverlayPositionType::to_public);
        let background_interaction = self
            .overlay_background_interaction
            .map(FigmaOverlayBackgroundInteraction::to_public);
        let relative_position = self.overlay_relative_position.map(FigmaVector2::to_tuple);
        if position.is_none() && background_interaction.is_none() && relative_position.is_none() {
            return None;
        }
        Some(PrototypeOverlayConfig {
            position,
            background_interaction,
            relative_position,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum FigmaPrototypeTriggerInput {
    Simple(FigmaPrototypeTrigger),
    Detailed(FigmaPrototypeTriggerDetails),
}

impl FigmaPrototypeTriggerInput {
    pub(crate) fn to_public(&self) -> Option<(PrototypeTrigger, Option<u32>)> {
        match self {
            Self::Simple(trigger) => trigger.to_public().map(|value| (value, None)),
            Self::Detailed(details) => details.to_public(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaPrototypeTriggerDetails {
    #[serde(default, rename = "type")]
    pub(crate) trigger_type: Option<FigmaPrototypeTrigger>,
    #[serde(default)]
    pub(crate) timeout: Option<f64>,
    #[serde(default)]
    pub(crate) delay: Option<f64>,
}

impl FigmaPrototypeTriggerDetails {
    pub(crate) fn to_public(&self) -> Option<(PrototypeTrigger, Option<u32>)> {
        let trigger = self
            .trigger_type
            .and_then(FigmaPrototypeTrigger::to_public)?;
        let timeout_ms = self.timeout.or(self.delay).and_then(duration_to_ms);
        Some((trigger, timeout_ms))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaPrototypeTransition {
    #[serde(default, rename = "type")]
    pub(crate) transition_type: Option<FigmaPrototypeTransitionType>,
    #[serde(default, alias = "transitionDuration")]
    pub(crate) duration: Option<f64>,
    #[serde(default, alias = "transitionEasing")]
    pub(crate) easing: Option<FigmaPrototypeEasingInput>,
    #[serde(default, alias = "transitionDirection")]
    pub(crate) direction: Option<FigmaPrototypeDirection>,
    #[serde(default)]
    pub(crate) match_layers: Option<bool>,
}

impl FigmaPrototypeTransition {
    pub(crate) fn to_public(&self) -> Option<PrototypeTransition> {
        let kind = self
            .transition_type
            .unwrap_or(FigmaPrototypeTransitionType::Unknown)
            .to_public();
        let duration_ms = self.duration.and_then(duration_to_ms);
        let easing = self
            .easing
            .as_ref()
            .and_then(FigmaPrototypeEasingInput::to_public);
        let direction = self.direction.and_then(FigmaPrototypeDirection::to_public);
        let match_layers = self.match_layers;
        if kind == PrototypeTransitionKind::Unknown
            && duration_ms.is_none()
            && easing.is_none()
            && direction.is_none()
            && match_layers.is_none()
        {
            return None;
        }
        Some(PrototypeTransition {
            kind,
            duration_ms,
            easing,
            direction,
            match_layers,
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaPrototypeActionType {
    #[serde(alias = "NODE")]
    Navigate,
    OpenOverlay,
    SwapOverlay,
    #[serde(alias = "CLOSE")]
    CloseOverlay,
    Back,
    #[serde(alias = "OPEN_URL")]
    Url,
    ScrollTo,
    #[serde(other)]
    Unknown,
}

impl FigmaPrototypeActionType {
    pub(crate) fn to_public(self) -> PrototypeActionKind {
        match self {
            Self::Navigate => PrototypeActionKind::Navigate,
            Self::OpenOverlay => PrototypeActionKind::OpenOverlay,
            Self::SwapOverlay => PrototypeActionKind::SwapOverlay,
            Self::CloseOverlay => PrototypeActionKind::CloseOverlay,
            Self::Back => PrototypeActionKind::Back,
            Self::Url => PrototypeActionKind::Url,
            Self::ScrollTo => PrototypeActionKind::ScrollTo,
            Self::Unknown => PrototypeActionKind::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaPrototypeTransitionType {
    Instant,
    Dissolve,
    MoveIn,
    MoveOut,
    Push,
    SlideIn,
    SlideOut,
    #[serde(alias = "MAGIC_MOVE")]
    SmartAnimate,
    ScrollAnimate,
    #[serde(other)]
    Unknown,
}

impl FigmaPrototypeTransitionType {
    pub(crate) fn to_public(self) -> PrototypeTransitionKind {
        match self {
            Self::Instant => PrototypeTransitionKind::Instant,
            Self::Dissolve => PrototypeTransitionKind::Dissolve,
            Self::MoveIn => PrototypeTransitionKind::MoveIn,
            Self::MoveOut => PrototypeTransitionKind::MoveOut,
            Self::Push => PrototypeTransitionKind::Push,
            Self::SlideIn => PrototypeTransitionKind::SlideIn,
            Self::SlideOut => PrototypeTransitionKind::SlideOut,
            Self::SmartAnimate => PrototypeTransitionKind::SmartAnimate,
            Self::ScrollAnimate => PrototypeTransitionKind::ScrollAnimate,
            Self::Unknown => PrototypeTransitionKind::Unknown,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum FigmaPrototypeEasingInput {
    Simple(FigmaPrototypeEasing),
    Detailed(FigmaPrototypeEasingDetails),
}

impl FigmaPrototypeEasingInput {
    pub(crate) fn to_public(&self) -> Option<PrototypeEasing> {
        match self {
            Self::Simple(easing) => Some(easing.to_public()),
            Self::Detailed(details) => details.easing_type.map(FigmaPrototypeEasing::to_public),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaPrototypeEasingDetails {
    #[serde(default, rename = "type")]
    pub(crate) easing_type: Option<FigmaPrototypeEasing>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaPrototypeEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInAndOut,
    Gentle,
    #[serde(other)]
    Unknown,
}

impl FigmaPrototypeEasing {
    pub(crate) fn to_public(self) -> PrototypeEasing {
        match self {
            Self::Linear => PrototypeEasing::Linear,
            Self::EaseIn => PrototypeEasing::EaseIn,
            Self::EaseOut => PrototypeEasing::EaseOut,
            Self::EaseInAndOut => PrototypeEasing::EaseInAndOut,
            Self::Gentle => PrototypeEasing::Gentle,
            Self::Unknown => PrototypeEasing::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaPrototypeDirection {
    Left,
    Right,
    Top,
    Bottom,
    #[serde(other)]
    Unknown,
}

impl FigmaPrototypeDirection {
    pub(crate) fn to_public(self) -> Option<PrototypeDirection> {
        match self {
            Self::Left => Some(PrototypeDirection::Left),
            Self::Right => Some(PrototypeDirection::Right),
            Self::Top => Some(PrototypeDirection::Top),
            Self::Bottom => Some(PrototypeDirection::Bottom),
            Self::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaOverlayPositionType {
    Center,
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Manual,
    #[serde(other)]
    Unknown,
}

impl FigmaOverlayPositionType {
    pub(crate) fn to_public(self) -> Option<PrototypeOverlayPosition> {
        match self {
            Self::Center => Some(PrototypeOverlayPosition::Center),
            Self::TopLeft => Some(PrototypeOverlayPosition::TopLeft),
            Self::TopCenter => Some(PrototypeOverlayPosition::TopCenter),
            Self::TopRight => Some(PrototypeOverlayPosition::TopRight),
            Self::BottomLeft => Some(PrototypeOverlayPosition::BottomLeft),
            Self::BottomCenter => Some(PrototypeOverlayPosition::BottomCenter),
            Self::BottomRight => Some(PrototypeOverlayPosition::BottomRight),
            Self::Manual => Some(PrototypeOverlayPosition::Manual),
            Self::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaOverlayBackgroundInteraction {
    None,
    CloseOnClickOutside,
    #[serde(alias = "PASSTHROUGH", alias = "PASS_THROUGH")]
    PassThrough,
    #[serde(other)]
    Unknown,
}

impl FigmaOverlayBackgroundInteraction {
    pub(crate) fn to_public(self) -> PrototypeOverlayBackgroundInteraction {
        match self {
            Self::None => PrototypeOverlayBackgroundInteraction::None,
            Self::CloseOnClickOutside => PrototypeOverlayBackgroundInteraction::CloseOnClickOutside,
            Self::PassThrough => PrototypeOverlayBackgroundInteraction::PassThrough,
            Self::Unknown => PrototypeOverlayBackgroundInteraction::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigmaVector2 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl FigmaVector2 {
    pub(crate) fn to_tuple(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FigmaPrototypeTrigger {
    OnClick,
    OnHover,
    OnDrag,
    AfterTimeout,
    OnPress,
    OnKeyDown,
    #[serde(other)]
    Unknown,
}

impl FigmaPrototypeTrigger {
    pub(crate) fn to_public(self) -> Option<PrototypeTrigger> {
        match self {
            Self::OnClick => Some(PrototypeTrigger::OnClick),
            Self::OnHover => Some(PrototypeTrigger::OnHover),
            Self::OnDrag => Some(PrototypeTrigger::OnDrag),
            Self::AfterTimeout => Some(PrototypeTrigger::AfterTimeout),
            Self::OnPress => Some(PrototypeTrigger::OnPress),
            Self::OnKeyDown => Some(PrototypeTrigger::OnKeyDown),
            Self::Unknown => None,
        }
    }
}
