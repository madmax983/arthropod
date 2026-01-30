use bevy_ecs::prelude::*;
use flux_state::{ReadSignal, WriteSignal};
use glam::Vec4;
use layout_engine::FlexStyle;
use render_engine::{Color, NodeId, Transform2D};
use std::collections::HashMap;
use std::sync::Arc;

/// Reference to a node in the Scene tree
///
/// This component links an ECS entity to a specific node in the custom Scene tree.
/// Systems can query this component to find which scene node to operate on.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneNodeRef(pub NodeId);

/// Thread-safe wrapper for ReadSignal
///
/// Wraps a `ReadSignal` to provide a consistent interface for components.
/// `ReadSignal` is intrinsically thread-safe (Send + Sync) thanks to internal
/// mutex locking in the flux-state runtime.
pub struct MainThreadSignal<T: 'static>(ReadSignal<T>);

impl<T: 'static> MainThreadSignal<T> {
    /// Create a new MainThreadSignal from a ReadSignal
    pub fn new(signal: ReadSignal<T>) -> Self {
        Self(signal)
    }

    /// Get the inner ReadSignal
    pub fn inner(&self) -> &ReadSignal<T> {
        &self.0
    }
}

impl<T: Clone + 'static> Clone for MainThreadSignal<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Reactive color - polls signal to update scene node
///
/// When this component is present, the reactive color system will poll the signal
/// and update the corresponding scene node's color property each frame.
#[derive(Component, Clone)]
pub struct ReactiveColor {
    pub signal: MainThreadSignal<Color>,
}

impl ReactiveColor {
    /// Create a new ReactiveColor from a ReadSignal
    pub fn new(signal: ReadSignal<Color>) -> Self {
        Self {
            signal: MainThreadSignal::new(signal),
        }
    }
}

/// Reactive transform - polls signal to update scene node
///
/// When this component is present, the reactive transform system will poll the signal
/// and update the corresponding scene node's transform each frame.
#[derive(Component, Clone)]
pub struct ReactiveTransform {
    pub signal: MainThreadSignal<Transform2D>,
}

impl ReactiveTransform {
    /// Create a new ReactiveTransform from a ReadSignal
    pub fn new(signal: ReadSignal<Transform2D>) -> Self {
        Self {
            signal: MainThreadSignal::new(signal),
        }
    }
}

/// Reactive opacity - polls signal to update scene node
///
/// When this component is present, the reactive opacity system will poll the signal
/// and update the corresponding scene node's opacity each frame.
#[derive(Component, Clone)]
pub struct ReactiveOpacity {
    pub signal: MainThreadSignal<f32>,
}

impl ReactiveOpacity {
    /// Create a new ReactiveOpacity from a ReadSignal
    pub fn new(signal: ReadSignal<f32>) -> Self {
        Self {
            signal: MainThreadSignal::new(signal),
        }
    }
}

/// Marker for renderable entities (has visual representation)
///
/// Entities with this component will be included in render queries.
/// Typically used in conjunction with SceneNodeRef to mark nodes that should be rendered.
#[derive(Component, Debug)]
pub struct Renderable;

/// Hoverable behavior (future - for events)
///
/// This component will enable hover interaction tracking for entities.
/// Currently a placeholder for future event system integration.
#[derive(Component)]
pub struct Hoverable {
    pub on_hover: Box<dyn Fn() + Send + Sync>,
}

/// Layout style for flexbox layout engine
///
/// Stores the FlexStyle configuration for this node. Layout systems
/// query this component to compute node positions and sizes.
#[derive(Component, Clone, Debug)]
pub struct LayoutStyle(pub FlexStyle);

/// Clickable behavior - callback invoked on click
///
/// Entities with this component can respond to mouse clicks.
/// The event system will invoke the callback when the node is clicked.
#[derive(Component, Clone)]
pub struct Clickable {
    pub callback: Arc<dyn Fn() + Send + Sync>,
}

/// Background color for nodes
///
/// Separate from NodeContent color - allows changing background without
/// rebuilding the entire node content.
#[derive(Component, Clone, Copy, Debug)]
pub struct BackgroundColor(pub Vec4);

/// Text input state component
///
/// Stores the reactive signals and cursor state for text input widgets.
#[derive(Component, Clone)]
pub struct TextInputState {
    pub read_signal: MainThreadSignal<String>,
    pub write_signal_inner: WriteSignal<String>, // WriteSignal is already Send+Sync
    pub cursor_position: usize,
    pub readonly: bool,
    pub max_length: Option<usize>,
}

impl TextInputState {
    pub fn new(
        read_signal: ReadSignal<String>,
        write_signal: WriteSignal<String>,
        cursor_position: usize,
        readonly: bool,
        max_length: Option<usize>,
    ) -> Self {
        Self {
            read_signal: MainThreadSignal::new(read_signal),
            write_signal_inner: write_signal,
            cursor_position,
            readonly,
            max_length,
        }
    }
}

/// Validator component - validates node content
///
/// Stores validation logic and current error state.
pub type ValidatorFn = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

#[derive(Component, Clone)]
pub struct Validator {
    pub validator: ValidatorFn,
    pub error: Option<String>,
}

/// Form state component - tracks form fields and submission
///
/// Stores the mapping of field names to node IDs, validation state,
/// and submission callback.
pub type SubmitCallback = Arc<dyn Fn(HashMap<String, String>) -> Result<(), String> + Send + Sync>;

#[derive(Component, Clone)]
pub struct FormState {
    pub field_mapping: HashMap<String, NodeId>,
    pub is_valid: bool,
    pub on_submit: Option<SubmitCallback>,
    pub submit_error: Option<String>,
}

/// Accessibility node component
///
/// Links an ECS entity to an A11y node, enabling screen reader support and
/// assistive technology integration.
#[derive(Component, Clone, Copy, Debug)]
pub struct AccessibleNode {
    /// Reference to the accessibility tree node
    pub a11y_id: a11y_engine::A11yId,
    /// Role for screen readers
    pub role: a11y_engine::Role,
}

impl AccessibleNode {
    pub fn new(a11y_id: a11y_engine::A11yId, role: a11y_engine::Role) -> Self {
        Self { a11y_id, role }
    }
}

/// Callback type for accessibility actions
pub type A11yActionCallback = Arc<dyn Fn() + Send + Sync>;

/// Click handler component for accessible nodes
///
/// Registers a callback that will be invoked when a screen reader
/// triggers a click action on this element.
#[derive(Component, Clone)]
pub struct OnA11yClick {
    pub callback: A11yActionCallback,
}

impl OnA11yClick {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(callback),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use a11y_engine::{A11yId, Role};

    #[test]
    fn test_accessible_node_component_creation() {
        let a11y_id = A11yId::new();
        let node = AccessibleNode::new(a11y_id, Role::Button);
        assert_eq!(node.role, Role::Button);
        assert_eq!(node.a11y_id, a11y_id);
    }

    #[test]
    fn test_accessible_node_different_roles() {
        let id1 = A11yId::new();
        let id2 = A11yId::new();

        let button = AccessibleNode::new(id1, Role::Button);
        let checkbox = AccessibleNode::new(id2, Role::Checkbox);

        assert_eq!(button.role, Role::Button);
        assert_eq!(checkbox.role, Role::Checkbox);
    }

    #[test]
    fn test_on_a11y_click_component_creation() {
        use std::sync::Mutex;

        let clicked = Arc::new(Mutex::new(false));
        let clicked_clone = clicked.clone();

        let click_handler = OnA11yClick::new(move || {
            *clicked_clone.lock().unwrap() = true;
        });

        // Invoke the callback
        (click_handler.callback)();

        assert!(
            *clicked.lock().unwrap(),
            "Callback should have been invoked"
        );
    }

    #[test]
    fn test_on_a11y_click_can_be_cloned() {
        use std::sync::Mutex;

        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();

        let handler1 = OnA11yClick::new(move || {
            *count_clone.lock().unwrap() += 1;
        });

        // Clone the handler
        let handler2 = handler1.clone();

        // Both should invoke the same callback
        (handler1.callback)();
        (handler2.callback)();

        assert_eq!(*count.lock().unwrap(), 2);
    }
}
