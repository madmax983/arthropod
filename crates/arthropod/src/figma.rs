use std::collections::HashMap;

use layout_engine::{FlexDirection, FlexStyle};
use plat_core::Rect;
use render_engine::{NodeContent, NodeId, Scene, SceneNode};
use serde::Deserialize;
use style_engine::{FontStyle, LineHeight, TextAlign, TextContent, TextDecoration, VisualStyle};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FigmaImportError {
    #[error("failed to parse figma json: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintAxis {
    Min,
    Center,
    Max,
    Stretch,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutPositioning {
    Auto,
    Absolute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeTrigger {
    OnClick,
    OnHover,
    OnDrag,
    AfterTimeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportedConstraints {
    pub horizontal: ConstraintAxis,
    pub vertical: ConstraintAxis,
    pub positioning: LayoutPositioning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrototypeEdge {
    pub from: NodeId,
    pub to_figma_id: String,
    pub trigger: PrototypeTrigger,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrototypeGraph {
    pub edges: Vec<PrototypeEdge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedComponentKind {
    Component,
    ComponentSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedComponentNode {
    pub kind: ImportedComponentKind,
    pub key: Option<String>,
    pub component_set_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedInstanceNode {
    pub component_id: Option<String>,
    pub main_component_id: Option<String>,
}

pub struct ImportedFigmaDocument {
    pub scene: Scene,
    pub layout_styles: HashMap<NodeId, FlexStyle>,
    pub constraints: HashMap<NodeId, ImportedConstraints>,
    pub prototype_graph: PrototypeGraph,
    pub figma_to_scene: HashMap<String, NodeId>,
    pub components: HashMap<NodeId, ImportedComponentNode>,
    pub instances: HashMap<NodeId, ImportedInstanceNode>,
    pub variant_properties: HashMap<NodeId, HashMap<String, String>>,
}

pub fn import_figma_document(json: &str) -> Result<ImportedFigmaDocument, FigmaImportError> {
    let document: FigmaDocument = serde_json::from_str(json)?;

    let mut scene = Scene::new();
    let root = scene.root();
    let mut figma_to_scene = HashMap::new();
    let mut layout_styles = HashMap::new();
    let mut constraints_map = HashMap::new();
    let mut components = HashMap::new();
    let mut instances = HashMap::new();
    let mut variant_properties = HashMap::new();
    let mut pending: Vec<usize> = (0..document.nodes.len()).collect();

    while !pending.is_empty() {
        let mut progressed = false;
        let mut unresolved = Vec::new();

        for index in pending {
            let node = &document.nodes[index];
            let parent = match &node.parent_id {
                Some(parent_id) => figma_to_scene.get(&parent_id.as_key()).copied(),
                None => Some(root),
            };

            if let Some(parent_id) = parent {
                let bounds = node.resolved_bounds();
                let mut scene_node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(node.to_visual_style()),
                });
                scene_node.bounds = bounds;
                let scene_id = scene.add_node(parent_id, scene_node);

                figma_to_scene.insert(node.id.as_key(), scene_id);
                layout_styles.insert(scene_id, node.to_flex_style(bounds));
                constraints_map.insert(scene_id, node.to_constraints());
                if let Some(component) = node.to_component_node() {
                    components.insert(scene_id, component);
                }
                if let Some(instance) = node.to_instance_node() {
                    instances.insert(scene_id, instance);
                }
                let variants = node.to_variant_properties();
                if !variants.is_empty() {
                    variant_properties.insert(scene_id, variants);
                }
                progressed = true;
            } else {
                unresolved.push(index);
            }
        }

        if !progressed {
            for index in unresolved {
                let node = &document.nodes[index];
                let bounds = node.resolved_bounds();
                let mut scene_node = SceneNode::new(NodeContent::Styled {
                    style: Box::new(node.to_visual_style()),
                });
                scene_node.bounds = bounds;
                let scene_id = scene.add_node(root, scene_node);

                figma_to_scene.insert(node.id.as_key(), scene_id);
                layout_styles.insert(scene_id, node.to_flex_style(bounds));
                constraints_map.insert(scene_id, node.to_constraints());
                if let Some(component) = node.to_component_node() {
                    components.insert(scene_id, component);
                }
                if let Some(instance) = node.to_instance_node() {
                    instances.insert(scene_id, instance);
                }
                let variants = node.to_variant_properties();
                if !variants.is_empty() {
                    variant_properties.insert(scene_id, variants);
                }
            }
            break;
        }

        pending = unresolved;
    }

    let mut prototype_edges = Vec::new();
    for node in &document.nodes {
        let Some(from) = figma_to_scene.get(&node.id.as_key()).copied() else {
            continue;
        };
        for interaction in &node.prototype_interactions {
            let Some(trigger) = interaction
                .trigger
                .and_then(FigmaPrototypeTrigger::to_public)
            else {
                continue;
            };
            let Some(destination_id) = interaction.destination_id.as_ref() else {
                continue;
            };
            prototype_edges.push(PrototypeEdge {
                from,
                to_figma_id: destination_id.as_key(),
                trigger,
            });
        }
    }

    Ok(ImportedFigmaDocument {
        scene,
        layout_styles,
        constraints: constraints_map,
        prototype_graph: PrototypeGraph {
            edges: prototype_edges,
        },
        figma_to_scene,
        components,
        instances,
        variant_properties,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaDocument {
    #[serde(default)]
    nodes: Vec<FigmaNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaNode {
    id: FigmaNodeKey,
    #[serde(default, alias = "parentId")]
    parent_id: Option<FigmaNodeKey>,
    #[serde(default, rename = "type")]
    node_type: Option<FigmaNodeType>,
    #[serde(default)]
    key: Option<String>,
    #[serde(default, alias = "componentSetId")]
    component_set_id: Option<FigmaNodeKey>,
    #[serde(default, alias = "componentId")]
    component_id: Option<FigmaNodeKey>,
    #[serde(default, alias = "mainComponent")]
    main_component: Option<FigmaMainComponent>,
    #[serde(default, alias = "mainComponentId")]
    main_component_id: Option<FigmaNodeKey>,
    #[serde(default)]
    variant_properties: HashMap<String, FigmaScalarValue>,
    #[serde(default)]
    component_properties: HashMap<String, FigmaComponentProperty>,
    #[serde(default)]
    bounds: Option<[f32; 4]>,
    #[serde(default, alias = "absoluteBoundingBox")]
    absolute_bounding_box: Option<FigmaRect>,
    #[serde(default)]
    layout_mode: Option<FigmaLayoutMode>,
    #[serde(default)]
    primary_axis_sizing_mode: Option<FigmaAxisSizingMode>,
    #[serde(default)]
    counter_axis_sizing_mode: Option<FigmaAxisSizingMode>,
    #[serde(default)]
    item_spacing: Option<f32>,
    #[serde(default)]
    padding_left: Option<f32>,
    #[serde(default)]
    padding_right: Option<f32>,
    #[serde(default)]
    padding_top: Option<f32>,
    #[serde(default)]
    padding_bottom: Option<f32>,
    #[serde(default)]
    layout_grow: Option<f32>,
    #[serde(default)]
    constraints: Option<FigmaConstraints>,
    #[serde(default)]
    layout_positioning: Option<FigmaLayoutPositioning>,
    #[serde(default)]
    characters: Option<String>,
    #[serde(default)]
    style: Option<FigmaTypeStyle>,
    #[serde(default, alias = "interactions", alias = "prototypeInteractions")]
    prototype_interactions: Vec<FigmaPrototypeInteraction>,
}

impl FigmaNode {
    fn resolved_bounds(&self) -> Rect {
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

    fn to_flex_style(&self, bounds: Rect) -> FlexStyle {
        let layout_mode = self.layout_mode.unwrap_or(FigmaLayoutMode::None);
        let mut style = FlexStyle {
            direction: match layout_mode {
                FigmaLayoutMode::Horizontal => FlexDirection::Row,
                FigmaLayoutMode::Vertical => FlexDirection::Column,
                FigmaLayoutMode::None | FigmaLayoutMode::Unknown => FlexDirection::Column,
            },
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

        style
    }

    fn to_constraints(&self) -> ImportedConstraints {
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

    fn to_visual_style(&self) -> VisualStyle {
        let mut style = VisualStyle::new();

        if matches!(self.node_type, Some(FigmaNodeType::Text))
            && let Some(text) = self.to_text_content()
        {
            style = style.text(text);
        }

        style
    }

    fn to_text_content(&self) -> Option<TextContent> {
        let characters = self.characters.as_ref()?.clone();
        let style = self.style.as_ref();

        let font_size = style
            .and_then(|s| s.font_size)
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(16.0);
        let mut text = TextContent::new(characters, font_size);

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

        Some(text)
    }

    fn to_component_node(&self) -> Option<ImportedComponentNode> {
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

    fn to_instance_node(&self) -> Option<ImportedInstanceNode> {
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

    fn to_variant_properties(&self) -> HashMap<String, String> {
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
}

fn map_font_style(style: Option<&FigmaTypeStyle>) -> FontStyle {
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

fn map_line_height(style: Option<&FigmaTypeStyle>) -> LineHeight {
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

fn figma_constraint_axis(axis: &str) -> ConstraintAxis {
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

fn canonical_property_name(name: &str) -> String {
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

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum FigmaNodeKey {
    Text(String),
    Numeric(u64),
}

impl FigmaNodeKey {
    fn as_key(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Numeric(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaNodeType {
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
enum FigmaLayoutMode {
    #[serde(rename = "NONE")]
    None,
    Horizontal,
    Vertical,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaAxisSizingMode {
    Fixed,
    Auto,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaLayoutPositioning {
    Auto,
    Absolute,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaConstraints {
    #[serde(default)]
    horizontal: Option<String>,
    #[serde(default)]
    vertical: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaMainComponent {
    id: FigmaNodeKey,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaComponentProperty {
    #[serde(default, rename = "type")]
    property_type: Option<FigmaComponentPropertyType>,
    #[serde(default)]
    value: Option<FigmaScalarValue>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaComponentPropertyType {
    Variant,
    Boolean,
    Text,
    InstanceSwap,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum FigmaScalarValue {
    Text(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
}

impl FigmaScalarValue {
    fn as_string(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Bool(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaTypeStyle {
    #[serde(default)]
    font_family: Option<String>,
    #[serde(default)]
    font_style: Option<String>,
    #[serde(default)]
    font_weight: Option<u16>,
    #[serde(default)]
    font_size: Option<f32>,
    #[serde(default)]
    italic: Option<bool>,
    #[serde(default)]
    text_align_horizontal: Option<FigmaTextAlignHorizontal>,
    #[serde(default)]
    line_height_px: Option<f32>,
    #[serde(default)]
    line_height_percent_font_size: Option<f32>,
    #[serde(default)]
    letter_spacing: Option<f32>,
    #[serde(default)]
    text_decoration: Option<FigmaTextDecoration>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaTextAlignHorizontal {
    Left,
    Center,
    Right,
    Justified,
    #[serde(other)]
    Unknown,
}

impl FigmaTextAlignHorizontal {
    fn to_text_align(self) -> TextAlign {
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
enum FigmaTextDecoration {
    None,
    Underline,
    Strikethrough,
    #[serde(other)]
    Unknown,
}

impl FigmaTextDecoration {
    fn to_text_decoration(self) -> TextDecoration {
        match self {
            Self::None => TextDecoration::None,
            Self::Underline => TextDecoration::Underline,
            Self::Strikethrough => TextDecoration::LineThrough,
            Self::Unknown => TextDecoration::None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaPrototypeInteraction {
    #[serde(default)]
    trigger: Option<FigmaPrototypeTrigger>,
    #[serde(default, alias = "destinationId", alias = "targetId")]
    destination_id: Option<FigmaNodeKey>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaPrototypeTrigger {
    OnClick,
    OnHover,
    OnDrag,
    AfterTimeout,
    #[serde(other)]
    Unknown,
}

impl FigmaPrototypeTrigger {
    fn to_public(self) -> Option<PrototypeTrigger> {
        match self {
            Self::OnClick => Some(PrototypeTrigger::OnClick),
            Self::OnHover => Some(PrototypeTrigger::OnHover),
            Self::OnDrag => Some(PrototypeTrigger::OnDrag),
            Self::AfterTimeout => Some(PrototypeTrigger::AfterTimeout),
            Self::Unknown => None,
        }
    }
}
