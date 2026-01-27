//! Accessibility node types
//!
//! Platform-agnostic representation of accessible UI elements following ARIA 1.2 patterns.

use plat_core::Rect;
use render_engine::NodeId;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for accessibility nodes
///
/// Separate from NodeId (scene graph) and Entity (ECS) to allow independent evolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct A11yId(u64);

static NEXT_A11Y_ID: AtomicU64 = AtomicU64::new(1);

impl A11yId {
    /// Create a new unique A11yId
    pub fn new() -> Self {
        Self(NEXT_A11Y_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value (for platform bridges)
    pub fn raw(&self) -> u64 {
        self.0
    }

    /// Create an A11yId from a raw value
    ///
    /// # Safety
    ///
    /// This should only be used when reconstructing an A11yId from a value
    /// previously obtained via `raw()`. The caller must ensure the ID was
    /// originally created by this system.
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

impl Default for A11yId {
    fn default() -> Self {
        Self::new()
    }
}

/// Platform-agnostic accessibility node
///
/// Contains all information needed for screen readers and assistive technology.
#[derive(Debug, Clone)]
pub struct A11yNode {
    /// ARIA-compatible role (button, checkbox, grid, etc.)
    pub role: Role,

    /// Accessible name (what screen readers announce)
    pub name: AccessibleName,

    /// Optional description (extra context)
    pub description: Option<String>,

    /// Current state (checked, expanded, disabled, etc.)
    pub state: A11yState,

    /// Available actions (click, focus, expand, etc.)
    pub actions: Vec<A11yAction>,

    /// Relationships to other nodes
    pub relations: A11yRelations,

    /// Screen coordinates for spatial navigation
    pub bounds: Rect,

    /// Parent in accessibility tree
    pub parent: Option<A11yId>,

    /// Children in accessibility tree
    pub children: Vec<A11yId>,

    /// Link to Scene node (for synchronization)
    pub scene_node: Option<NodeId>,
}

impl Default for A11yNode {
    fn default() -> Self {
        Self {
            role: Role::Group, // Generic container by default
            name: AccessibleName::ComputedFromChildren,
            description: None,
            state: A11yState::default(),
            actions: Vec::new(),
            relations: A11yRelations::default(),
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            parent: None,
            children: Vec::new(),
            scene_node: None,
        }
    }
}

/// ARIA-compatible roles
///
/// Subset for MVP - full ARIA 1.2 role set to be added incrementally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    // Widgets
    Button,
    Checkbox,
    Radio,
    Textbox,
    Slider,
    ProgressBar,

    // Containers
    Group,
    List,
    ListItem,
    Grid,
    GridCell,

    // Document structure
    Heading { level: u8 }, // 1-6
    Paragraph,
    Region,

    // Landmarks
    Main,
    Navigation,
    Search,
    Form,

    // Special
    Alert,
    Dialog,
    Tooltip,
}

/// Accessible name (what screen readers announce)
#[derive(Debug, Clone, PartialEq)]
pub enum AccessibleName {
    /// Direct text label
    Text(String),

    /// Reference to labelling element
    LabelledBy(A11yId),

    /// Computed from children (for containers)
    ComputedFromChildren,
}

/// State flags (can combine multiple)
///
/// Based on ARIA state attributes (aria-checked, aria-expanded, etc.)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct A11yState {
    pub checked: Option<CheckedState>,
    pub expanded: Option<bool>,
    pub disabled: bool,
    pub focused: bool,
    pub selected: bool,
    pub hidden: bool,
    pub readonly: bool,
    pub required: bool,
    pub invalid: bool,
}

/// Checkbox/radio checked state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckedState {
    Unchecked,
    Checked,
    Mixed, // Indeterminate (some children checked)
}

/// Available actions on node
///
/// Maps to platform-specific actions (UIA patterns, NSAccessibility actions, AT-SPI actions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum A11yAction {
    Click,
    Focus,
    Expand,
    Collapse,
    Check,
    Uncheck,
    Select,
    Increment,
    Decrement,
    ShowContextMenu,
}

/// Relationships between nodes (ARIA relations)
///
/// Based on ARIA relationship attributes (aria-labelledby, aria-describedby, etc.)
#[derive(Debug, Clone, Default, PartialEq)]
pub struct A11yRelations {
    pub labelled_by: Vec<A11yId>,
    pub described_by: Vec<A11yId>,
    pub controls: Vec<A11yId>,
    pub owns: Vec<A11yId>,
    pub flows_to: Option<A11yId>,
}
