use std::collections::HashMap;

use layout_engine::{
    FlexAlign, FlexDirection, FlexJustifyContent, FlexStyle, FlexWrap, ItemAlignSelf,
};
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
    OnPress,
    OnKeyDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeActionKind {
    Navigate,
    OpenOverlay,
    SwapOverlay,
    CloseOverlay,
    Back,
    Url,
    ScrollTo,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeTransitionKind {
    Instant,
    Dissolve,
    MoveIn,
    MoveOut,
    Push,
    SlideIn,
    SlideOut,
    SmartAnimate,
    ScrollAnimate,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInAndOut,
    Gentle,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeDirection {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrototypeTransition {
    pub kind: PrototypeTransitionKind,
    pub duration_ms: Option<u32>,
    pub easing: Option<PrototypeEasing>,
    pub direction: Option<PrototypeDirection>,
    pub match_layers: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeOverlayPosition {
    Center,
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeOverlayBackgroundInteraction {
    None,
    CloseOnClickOutside,
    PassThrough,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrototypeOverlayConfig {
    pub position: Option<PrototypeOverlayPosition>,
    pub background_interaction: Option<PrototypeOverlayBackgroundInteraction>,
    pub relative_position: Option<(f32, f32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedConstraints {
    pub horizontal: ConstraintAxis,
    pub vertical: ConstraintAxis,
    pub positioning: LayoutPositioning,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrototypeEdge {
    pub from: NodeId,
    pub to_figma_id: Option<String>,
    pub trigger: PrototypeTrigger,
    pub trigger_timeout_ms: Option<u32>,
    pub action: PrototypeActionKind,
    pub preserve_scroll_position: bool,
    pub transition: Option<PrototypeTransition>,
    pub overlay: Option<PrototypeOverlayConfig>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
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
            let Some((trigger, trigger_timeout_ms)) = interaction.trigger_spec() else {
                continue;
            };

            let inherited_transition = interaction.transition_details();
            let inherited_preserve_scroll = interaction.preserve_scroll_position.unwrap_or(false);

            if interaction.actions.is_empty() {
                if let Some(edge) = interaction.to_legacy_edge(
                    from,
                    trigger,
                    trigger_timeout_ms,
                    inherited_transition,
                    inherited_preserve_scroll,
                ) {
                    prototype_edges.push(edge);
                }
                continue;
            }

            for action in &interaction.actions {
                if let Some(edge) = action.to_edge(
                    from,
                    trigger,
                    trigger_timeout_ms,
                    inherited_transition.clone(),
                    inherited_preserve_scroll,
                ) {
                    prototype_edges.push(edge);
                }
            }
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
    primary_axis_align_items: Option<FigmaPrimaryAxisAlignItems>,
    #[serde(default)]
    counter_axis_align_items: Option<FigmaCounterAxisAlignItems>,
    #[serde(default)]
    layout_wrap: Option<FigmaLayoutWrap>,
    #[serde(default)]
    primary_axis_sizing_mode: Option<FigmaAxisSizingMode>,
    #[serde(default)]
    counter_axis_sizing_mode: Option<FigmaAxisSizingMode>,
    #[serde(default)]
    layout_align: Option<FigmaLayoutAlign>,
    #[serde(default)]
    layout_sizing_horizontal: Option<FigmaLayoutSizingMode>,
    #[serde(default)]
    layout_sizing_vertical: Option<FigmaLayoutSizingMode>,
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
    min_width: Option<f32>,
    #[serde(default)]
    max_width: Option<f32>,
    #[serde(default)]
    min_height: Option<f32>,
    #[serde(default)]
    max_height: Option<f32>,
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

fn duration_to_ms(value: f64) -> Option<u32> {
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
enum FigmaPrimaryAxisAlignItems {
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
    fn to_justify_content(self) -> FlexJustifyContent {
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
enum FigmaCounterAxisAlignItems {
    Min,
    Center,
    Max,
    Baseline,
    #[serde(other)]
    Unknown,
}

impl FigmaCounterAxisAlignItems {
    fn to_align(self) -> FlexAlign {
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
enum FigmaLayoutWrap {
    NoWrap,
    Wrap,
    #[serde(other)]
    Unknown,
}

impl FigmaLayoutWrap {
    fn to_wrap(self) -> FlexWrap {
        match self {
            Self::NoWrap => FlexWrap::NoWrap,
            Self::Wrap => FlexWrap::Wrap,
            Self::Unknown => FlexWrap::NoWrap,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaLayoutAlign {
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
    fn to_align_self(self) -> Option<ItemAlignSelf> {
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
enum FigmaLayoutSizingMode {
    Fill,
    Hug,
    Fixed,
    Auto,
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
    trigger: Option<FigmaPrototypeTriggerInput>,
    #[serde(
        default,
        alias = "destinationId",
        alias = "targetId",
        alias = "transitionNodeID",
        alias = "transitionNodeId"
    )]
    destination_id: Option<FigmaNodeKey>,
    #[serde(default)]
    actions: Vec<FigmaPrototypeAction>,
    #[serde(default, rename = "action")]
    action_type: Option<FigmaPrototypeActionType>,
    #[serde(default)]
    navigation: Option<FigmaPrototypeActionType>,
    #[serde(default)]
    transition: Option<FigmaPrototypeTransition>,
    #[serde(default)]
    preserve_scroll_position: Option<bool>,
    #[serde(default)]
    url: Option<String>,
}

impl FigmaPrototypeInteraction {
    fn trigger_spec(&self) -> Option<(PrototypeTrigger, Option<u32>)> {
        self.trigger
            .as_ref()
            .and_then(FigmaPrototypeTriggerInput::to_public)
    }

    fn transition_details(&self) -> Option<PrototypeTransition> {
        self.transition
            .as_ref()
            .and_then(FigmaPrototypeTransition::to_public)
    }

    fn to_legacy_edge(
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

    fn legacy_action_kind(&self) -> PrototypeActionKind {
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
struct FigmaPrototypeAction {
    #[serde(default, rename = "type")]
    action_type: Option<FigmaPrototypeActionType>,
    #[serde(
        default,
        alias = "destinationId",
        alias = "targetId",
        alias = "nodeId",
        alias = "transitionNodeID",
        alias = "transitionNodeId"
    )]
    destination_id: Option<FigmaNodeKey>,
    #[serde(default)]
    transition: Option<FigmaPrototypeTransition>,
    #[serde(default)]
    preserve_scroll_position: Option<bool>,
    #[serde(default)]
    overlay_position_type: Option<FigmaOverlayPositionType>,
    #[serde(default)]
    overlay_background_interaction: Option<FigmaOverlayBackgroundInteraction>,
    #[serde(default)]
    overlay_relative_position: Option<FigmaVector2>,
    #[serde(default)]
    url: Option<String>,
}

impl FigmaPrototypeAction {
    fn to_edge(
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

    fn overlay_config(&self) -> Option<PrototypeOverlayConfig> {
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
enum FigmaPrototypeTriggerInput {
    Simple(FigmaPrototypeTrigger),
    Detailed(FigmaPrototypeTriggerDetails),
}

impl FigmaPrototypeTriggerInput {
    fn to_public(&self) -> Option<(PrototypeTrigger, Option<u32>)> {
        match self {
            Self::Simple(trigger) => trigger.to_public().map(|value| (value, None)),
            Self::Detailed(details) => details.to_public(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaPrototypeTriggerDetails {
    #[serde(default, rename = "type")]
    trigger_type: Option<FigmaPrototypeTrigger>,
    #[serde(default)]
    timeout: Option<f64>,
    #[serde(default)]
    delay: Option<f64>,
}

impl FigmaPrototypeTriggerDetails {
    fn to_public(&self) -> Option<(PrototypeTrigger, Option<u32>)> {
        let trigger = self
            .trigger_type
            .and_then(FigmaPrototypeTrigger::to_public)?;
        let timeout_ms = self.timeout.or(self.delay).and_then(duration_to_ms);
        Some((trigger, timeout_ms))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaPrototypeTransition {
    #[serde(default, rename = "type")]
    transition_type: Option<FigmaPrototypeTransitionType>,
    #[serde(default, alias = "transitionDuration")]
    duration: Option<f64>,
    #[serde(default, alias = "transitionEasing")]
    easing: Option<FigmaPrototypeEasingInput>,
    #[serde(default, alias = "transitionDirection")]
    direction: Option<FigmaPrototypeDirection>,
    #[serde(default)]
    match_layers: Option<bool>,
}

impl FigmaPrototypeTransition {
    fn to_public(&self) -> Option<PrototypeTransition> {
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
enum FigmaPrototypeActionType {
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
    fn to_public(self) -> PrototypeActionKind {
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
enum FigmaPrototypeTransitionType {
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
    fn to_public(self) -> PrototypeTransitionKind {
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
enum FigmaPrototypeEasingInput {
    Simple(FigmaPrototypeEasing),
    Detailed(FigmaPrototypeEasingDetails),
}

impl FigmaPrototypeEasingInput {
    fn to_public(&self) -> Option<PrototypeEasing> {
        match self {
            Self::Simple(easing) => Some(easing.to_public()),
            Self::Detailed(details) => details.easing_type.map(FigmaPrototypeEasing::to_public),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigmaPrototypeEasingDetails {
    #[serde(default, rename = "type")]
    easing_type: Option<FigmaPrototypeEasing>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaPrototypeEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInAndOut,
    Gentle,
    #[serde(other)]
    Unknown,
}

impl FigmaPrototypeEasing {
    fn to_public(self) -> PrototypeEasing {
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
enum FigmaPrototypeDirection {
    Left,
    Right,
    Top,
    Bottom,
    #[serde(other)]
    Unknown,
}

impl FigmaPrototypeDirection {
    fn to_public(self) -> Option<PrototypeDirection> {
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
enum FigmaOverlayPositionType {
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
    fn to_public(self) -> Option<PrototypeOverlayPosition> {
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
enum FigmaOverlayBackgroundInteraction {
    None,
    CloseOnClickOutside,
    #[serde(alias = "PASSTHROUGH", alias = "PASS_THROUGH")]
    PassThrough,
    #[serde(other)]
    Unknown,
}

impl FigmaOverlayBackgroundInteraction {
    fn to_public(self) -> PrototypeOverlayBackgroundInteraction {
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
struct FigmaVector2 {
    x: f32,
    y: f32,
}

impl FigmaVector2 {
    fn to_tuple(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FigmaPrototypeTrigger {
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
    fn to_public(self) -> Option<PrototypeTrigger> {
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
