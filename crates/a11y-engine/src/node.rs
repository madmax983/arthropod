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

/// ARIA-compatible roles defining the semantic meaning of a UI element.
///
/// Roles are the primary way assistive technologies (like screen readers) understand
/// what a specific visual element on the screen actually *does*. This enum represents
/// a subset of the full ARIA 1.2 specification, currently focused on the MVP widget set.
///
/// ## Examples
///
/// ```
/// use a11y_engine::node::Role;
///
/// // A clickable interface element.
/// let my_role = Role::Button;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    // Widgets
    /// A standard push button that triggers an action when clicked.
    Button,
    /// A toggle switch with a binary on/off (or mixed) state.
    Checkbox,
    /// A mutually exclusive selection option within a group.
    Radio,
    /// An input field allowing the user to enter freeform text.
    Textbox,
    /// An input control that allows the user to select a value from within a given range.
    Slider,
    /// An indicator showing the completion status of a task.
    ProgressBar,

    // Containers
    /// A generic container used to group related elements together structurally.
    Group,
    /// A container specifically for a sequence of list items.
    List,
    /// An individual item contained within a `List` or `Group`.
    ListItem,
    /// A complex tabular container with rows and columns, allowing for spatial navigation.
    Grid,
    /// An individual, focusable cell within a `Grid`.
    GridCell,

    // Document structure
    /// A structural heading that defines the start of a new section.
    Heading {
        /// The hierarchical level of the heading, typically from 1 (most important) to 6 (least important).
        level: u8,
    },
    /// A standard block of flowing text.
    Paragraph,
    /// A generic perceivable section containing content that is relevant to a specific purpose.
    Region,

    // Landmarks
    /// The primary content of a document or application.
    Main,
    /// A collection of links suitable for use when navigating the document or related documents.
    Navigation,
    /// A region that contains a collection of items and objects that, as a whole, combine to create a search facility.
    Search,
    /// A landmark region that contains a collection of items and objects that, as a whole, combine to create a form.
    Form,

    // Special
    /// An important, time-sensitive message that requires the user's immediate attention.
    Alert,
    /// A transient, modal window that interrupts the user's workflow to ask a question or present critical info.
    Dialog,
    /// A contextual popup that displays a description for an element when it receives focus or hover.
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

/// A bitfield-like structure representing the current semantic state of an accessible node.
///
/// While `Role` defines what an element *is*, `A11yState` defines what it is currently *doing*.
/// It maps directly to ARIA state attributes (e.g., `aria-checked`, `aria-expanded`, `aria-disabled`).
///
/// ## Examples
///
/// ```
/// use a11y_engine::node::{A11yState, CheckedState};
///
/// let mut state = A11yState::default();
/// state.disabled = true;
/// state.checked = Some(CheckedState::Checked);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct A11yState {
    /// Indicates whether the element is toggled on, off, or in an indeterminate state. Commonly used for Checkboxes and Radios.
    pub checked: Option<CheckedState>,
    /// Indicates whether a collapsible container is currently showing (`true`) or hiding (`false`) its contents.
    pub expanded: Option<bool>,
    /// If `true`, the element exists but cannot currently be interacted with by the user.
    pub disabled: bool,
    /// If `true`, this element currently holds the application's keyboard focus.
    pub focused: bool,
    /// If `true`, this element is currently selected within a larger selectable group (like a List or Grid).
    pub selected: bool,
    /// If `true`, the element is completely ignored by assistive technologies, effectively removing it from the accessibility tree.
    pub hidden: bool,
    /// If `true`, the element's value can be read but cannot be modified by the user (common in Textboxes).
    pub readonly: bool,
    /// If `true`, user input is mandatory on this element before the enclosing form can be successfully submitted.
    pub required: bool,
    /// If `true`, the user has provided input that fails the element's validation constraints.
    pub invalid: bool,
}

/// Represents the specific checked status of a togglable UI element.
///
/// ## Examples
///
/// ```
/// use a11y_engine::node::CheckedState;
///
/// let completely_selected = CheckedState::Checked;
/// let partially_selected = CheckedState::Mixed;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckedState {
    /// The element is entirely untoggled.
    Unchecked,
    /// The element is entirely toggled on.
    Checked,
    /// The element is in an indeterminate state (e.g. a parent checkbox where only some children are checked).
    Mixed,
}

/// Represents a specific, semantic interaction that an assistive technology can request an element to perform.
///
/// These actions bridge the gap between physical hardware events (like a mouse click) and semantic
/// intent. For example, a screen reader user might trigger a "Click" action via a keyboard shortcut,
/// which the application must then route to the underlying widget.
///
/// ## Examples
///
/// ```
/// use a11y_engine::node::A11yAction;
///
/// let default_interaction = A11yAction::Click;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum A11yAction {
    /// Request the default behavior of the element (e.g., submitting a form, following a link, or pressing a button).
    Click,
    /// Request that the application route keyboard input to this specific element.
    Focus,
    /// Request that a collapsed container reveal its contents.
    Expand,
    /// Request that an expanded container hide its contents.
    Collapse,
    /// Request that a togglable element transition to the `Checked` state.
    Check,
    /// Request that a togglable element transition to the `Unchecked` state.
    Uncheck,
    /// Request that this item become the active choice within its selection group.
    Select,
    /// Request to increase the value of a range control (like a Slider).
    Increment,
    /// Request to decrease the value of a range control.
    Decrement,
    /// Request to display the context menu associated with this element.
    ShowContextMenu,
}

/// Defines the complex, non-hierarchical semantic relationships between different nodes in the accessibility tree.
///
/// While the standard parent/child tree defines layout, relations define semantic links,
/// such as a text label explicitly describing a separate input field, or a custom reading order.
///
/// ## Examples
///
/// ```
/// use a11y_engine::node::{A11yRelations, A11yId};
///
/// let label_id = A11yId::new();
/// let input_relations = A11yRelations {
///     labelled_by: vec![label_id],
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct A11yRelations {
    /// A list of node IDs that provide the primary accessible name (label) for this element. Maps to `aria-labelledby`.
    pub labelled_by: Vec<A11yId>,
    /// A list of node IDs that provide extended, secondary descriptive text for this element. Maps to `aria-describedby`.
    pub described_by: Vec<A11yId>,
    /// A list of node IDs whose state or visibility is directly managed by this element (e.g., a button controlling a dropdown menu). Maps to `aria-controls`.
    pub controls: Vec<A11yId>,
    /// A list of node IDs that should be treated as semantic children of this element, even if they exist elsewhere in the visual DOM tree. Maps to `aria-owns`.
    pub owns: Vec<A11yId>,
    /// An optional node ID indicating the next element in a custom, logical reading order that diverges from the standard visual layout. Maps to `aria-flowto`.
    pub flows_to: Option<A11yId>,
}
