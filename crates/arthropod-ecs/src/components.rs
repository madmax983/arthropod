use bevy_ecs::prelude::*;
use flux_state::{Computed, ReadSignal};
use glam::Vec4;
use layout_engine::{FlexStyle, LayoutConstraints};
use render_engine::{Color, NodeId, Transform2D};
use std::sync::Arc;

/// Reference to a node in the Scene tree
///
/// This component links an ECS entity to a specific node in the custom Scene tree.
/// Systems can query this component to find which scene node to operate on.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneNodeRef(pub NodeId);

/// Reactive color - polls signal to update scene node
///
/// When this component is present, the reactive color system will poll the signal
/// and update the corresponding scene node's color property each frame.
#[derive(Component, Clone)]
pub struct ReactiveColor {
    pub signal: ReadSignal<Color>,
}

impl ReactiveColor {
    /// Create a new ReactiveColor from a ReadSignal
    pub fn new(signal: ReadSignal<Color>) -> Self {
        Self { signal }
    }
}

/// Reactive transform - polls signal to update scene node
///
/// When this component is present, the reactive transform system will poll the signal
/// and update the corresponding scene node's transform each frame.
#[derive(Component, Clone)]
pub struct ReactiveTransform {
    pub signal: ReadSignal<Transform2D>,
}

impl ReactiveTransform {
    /// Create a new ReactiveTransform from a ReadSignal
    pub fn new(signal: ReadSignal<Transform2D>) -> Self {
        Self { signal }
    }
}

/// Reactive opacity - polls signal to update scene node
///
/// When this component is present, the reactive opacity system will poll the signal
/// and update the corresponding scene node's opacity each frame.
#[derive(Component, Clone)]
pub struct ReactiveOpacity {
    pub signal: ReadSignal<f32>,
}

impl ReactiveOpacity {
    /// Create a new ReactiveOpacity from a ReadSignal
    pub fn new(signal: ReadSignal<f32>) -> Self {
        Self { signal }
    }
}

/// Reactive text - polls signal to update scene node text content
///
/// When this component is present, the reactive text system will poll the signal
/// and update the corresponding scene node's text content each frame.
#[derive(Component, Clone)]
pub struct ReactiveText {
    pub signal: ReadSignal<String>,
}

impl ReactiveText {
    /// Create a new ReactiveText from a ReadSignal
    pub fn new(signal: ReadSignal<String>) -> Self {
        Self { signal }
    }
}

/// Reactive computed text - polls computed value to update scene node text content
///
/// When this component is present, the reactive computed text system will poll the computed
/// and update the corresponding scene node's text content each frame.
/// Computed values automatically update when their dependencies change.
#[derive(Component, Clone)]
pub struct ReactiveComputedText {
    pub computed: Computed<String>,
}

impl ReactiveComputedText {
    /// Create a new ReactiveComputedText from a Computed
    pub fn new(computed: Computed<String>) -> Self {
        Self { computed }
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

/// Global resource for layout constraints (e.g., window size)
///
/// This resource is used by the layout system to constrain the root node.
#[derive(Resource, Default, Clone, Debug)]
pub struct LayoutConstraintsResource(pub LayoutConstraints);

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
