#![allow(missing_docs)]

use hashbrown::HashMap;

use layout_engine::FlexStyle;
use render_engine::{NodeId, Scene};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FigmaImportError {
    #[error("failed to parse figma json: {0}")]
    Parse(#[from] serde_json::Error),
    #[error(
        "invalid figma json shape: expected an array of nodes or an object with a `nodes` array"
    )]
    InvalidDocumentShape,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedComponentPropertyType {
    Variant,
    Boolean,
    Text,
    InstanceSwap,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportedComponentPropertyValue {
    Text(String),
    Bool(bool),
    Number(f64),
    NodeRef(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedComponentPropertyDefinition {
    pub property_type: ImportedComponentPropertyType,
    pub default_value: Option<ImportedComponentPropertyValue>,
    pub preferred_values: Vec<ImportedComponentPropertyValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedComponentPropertyOverride {
    pub property_type: ImportedComponentPropertyType,
    pub value: ImportedComponentPropertyValue,
}

/// The fully parsed and resolved root structure of an imported Figma document.
///
/// This struct holds the converted node hierarchy (as a render engine `Scene`),
/// resolved flexbox layout styles, component property definitions, constraints,
/// and prototype interaction graphs. It serves as the bridge between raw Figma JSON
/// and the `FigmaRuntime` which executes the logic.
pub struct ImportedFigmaDocument {
    pub scene: Scene,
    pub layout_styles: HashMap<NodeId, FlexStyle>,
    pub constraints: HashMap<NodeId, ImportedConstraints>,
    pub prototype_graph: PrototypeGraph,
    pub figma_to_scene: HashMap<String, NodeId>,
    pub components: HashMap<NodeId, ImportedComponentNode>,
    pub instances: HashMap<NodeId, ImportedInstanceNode>,
    pub variant_properties: HashMap<NodeId, HashMap<String, String>>,
    pub component_property_definitions:
        HashMap<NodeId, HashMap<String, ImportedComponentPropertyDefinition>>,
    pub instance_property_overrides:
        HashMap<NodeId, HashMap<String, ImportedComponentPropertyOverride>>,
    pub resolved_instance_properties:
        HashMap<NodeId, HashMap<String, ImportedComponentPropertyValue>>,
}
