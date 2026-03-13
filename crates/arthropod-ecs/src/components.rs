use bevy_ecs::prelude::*;
use flux_state::{Computed, ReadSignal};
use glam::Vec4;
use layout_engine::{FlexStyle, LayoutConstraints};
use render_engine::{Color, NodeId, Transform2D};
use std::sync::Arc;

/// Reference to a node in the Scene tree
///
/// This component is the **bridge** between the ECS world and the Scene Graph.
/// Arthropod uses a "Hybrid Architecture":
/// - **Scene Graph (`render-engine`)**: Specialized tree structure for layout and parent-child transforms (O(1) access, cache-friendly).
/// - **ECS (`arthropod-ecs`)**: Flat arrays of components for cross-cutting systems like animation, physics, and reactivity.
///
/// `SceneNodeRef` allows ECS systems to "reach into" the scene graph and modify node properties.
///
/// # Example: A System that updates Scene Nodes
///
/// ```
/// use bevy_ecs::prelude::*;
/// use arthropod_ecs::SceneNodeRef;
/// use render_engine::{Scene, NodeId, Transform2D};
///
/// // A simple component
/// #[derive(Component)]
/// struct Velocity(f32);
///
/// // A system that reads Velocity and updates the Scene Node position
/// fn movement_system(
///     mut scene: ResMut<Scene>,
///     query: Query<(&SceneNodeRef, &Velocity)>
/// ) {
///     for (node_ref, velocity) in query.iter() {
///         // 1. Get the raw NodeId from the component
///         let node_id = node_ref.0;
///
///         // 2. Use it to access the Scene Graph
///         if let Some(node) = scene.get_node_mut(node_id) {
///             // 3. Modify the node directly
///             // We compose the current transform with a translation
///             let translation = Transform2D::translate(velocity.0, 0.0);
///             node.transform = node.transform.compose(&translation);
///         }
///     }
/// }
/// ```
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneNodeRef(pub NodeId);

/// Reactive color - polls signal to update scene node
///
/// When this component is present, the reactive color system will poll the signal
/// and update the corresponding scene node's color property each frame.
///
/// # Example
///
/// ```
/// # use arthropod_ecs::ReactiveColor;
/// # use flux_state::{Runtime, Signal};
/// # use render_engine::Color;
/// # let runtime = Runtime::new();
/// let color = Signal::new(runtime, Color::RED);
/// let (read, _) = color.split();
///
/// let reactive = ReactiveColor::new(read);
/// ```
#[derive(Component, Clone)]
pub struct ReactiveColor {
    pub signal: ReadSignal<Color>,
    pub last_value: Color,
}

impl ReactiveColor {
    /// Create a new ReactiveColor from a ReadSignal
    pub fn new(signal: ReadSignal<Color>) -> Self {
        let last_value = signal.get_untracked();
        Self { signal, last_value }
    }
}

/// Reactive transform - polls signal to update scene node
///
/// When this component is present, the reactive transform system will poll the signal
/// and update the corresponding scene node's transform each frame.
///
/// # Example
///
/// ```
/// # use arthropod_ecs::ReactiveTransform;
/// # use flux_state::{Runtime, Signal};
/// # use render_engine::Transform2D;
/// # let runtime = Runtime::new();
/// let transform = Signal::new(runtime, Transform2D::identity());
/// let (read, _) = transform.split();
///
/// let reactive = ReactiveTransform::new(read);
/// ```
#[derive(Component, Clone)]
pub struct ReactiveTransform {
    pub signal: ReadSignal<Transform2D>,
    pub last_value: Transform2D,
}

impl ReactiveTransform {
    /// Create a new ReactiveTransform from a ReadSignal
    pub fn new(signal: ReadSignal<Transform2D>) -> Self {
        let last_value = signal.get_untracked();
        Self { signal, last_value }
    }
}

/// Reactive opacity - polls signal to update scene node
///
/// When this component is present, the reactive opacity system will poll the signal
/// and update the corresponding scene node's opacity each frame.
///
/// # Example
///
/// ```
/// # use arthropod_ecs::ReactiveOpacity;
/// # use flux_state::{Runtime, Signal};
/// # let runtime = Runtime::new();
/// // Note: ReactiveOpacity requires an f32 signal
/// let opacity = Signal::new(runtime, 1.0f32);
/// let (read, _) = opacity.split();
///
/// let reactive = ReactiveOpacity::new(read);
/// ```
#[derive(Component, Clone)]
pub struct ReactiveOpacity {
    pub signal: ReadSignal<f32>,
    pub last_value: f32,
}

impl ReactiveOpacity {
    /// Create a new ReactiveOpacity from a ReadSignal
    pub fn new(signal: ReadSignal<f32>) -> Self {
        let last_value = signal.get_untracked();
        Self { signal, last_value }
    }
}

/// Reactive text - polls signal to update scene node text content
///
/// When this component is present, the reactive text system will poll the signal
/// and update the corresponding scene node's text content each frame.
///
/// # Example
///
/// ```
/// # use arthropod_ecs::ReactiveText;
/// # use flux_state::{Runtime, Signal};
/// # let runtime = Runtime::new();
/// let text = Signal::new(runtime, "Hello".to_string());
/// let (read, _) = text.split();
///
/// let reactive = ReactiveText::new(read);
/// ```
#[derive(Component, Clone)]
pub struct ReactiveText {
    pub signal: ReadSignal<String>,
    pub last_value: String,
}

impl ReactiveText {
    /// Create a new ReactiveText from a ReadSignal
    pub fn new(signal: ReadSignal<String>) -> Self {
        let last_value = signal.get_untracked();
        Self { signal, last_value }
    }
}

/// Reactive computed text - polls computed value to update scene node text content
///
/// When this component is present, the reactive computed text system will poll the computed
/// and update the corresponding scene node's text content each frame.
/// Computed values automatically update when their dependencies change.
///
/// # Example
///
/// ```
/// # use arthropod_ecs::ReactiveComputedText;
/// # use flux_state::{Runtime, Signal, Computed};
/// # let runtime = Runtime::new();
/// let count = Signal::new(runtime.clone(), 0);
/// let (read, _) = count.split();
///
/// let computed = Computed::new(runtime, move || format!("Count: {}", read.get()));
///
/// let reactive = ReactiveComputedText::new(computed);
/// ```
#[derive(Component, Clone)]
pub struct ReactiveComputedText {
    pub computed: Computed<String>,
    pub last_value: String,
}

impl ReactiveComputedText {
    /// Create a new ReactiveComputedText from a Computed
    pub fn new(computed: Computed<String>) -> Self {
        let last_value = computed.get();
        Self {
            computed,
            last_value,
        }
    }
}

/// Current interaction state of a widget
///
/// Tracks hover, focus, and active (pressed) states.
/// These states are used by the style system to resolve pseudo-states (:hover, etc.)
/// and by the interaction system to trigger callbacks.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InteractionState {
    pub hovered: bool,
    pub focused: bool,
    pub active: bool, // Pressed
    pub disabled: bool,
}

/// Unified style component - resolves pseudo-states into visual/layout properties
///
/// Stores a high-level `theme_engine::Style` which contains both layout and
/// visual properties, including pseudo-state overrides.
#[derive(Component, Clone, Debug, Default)]
pub struct WidgetStyle(pub theme_engine::Style);

/// Reactive layout width - polls signal to update LayoutStyle width
///
/// When this component is present, the reactive layout system will poll the signal
/// and update the corresponding LayoutStyle component's width each frame.
#[derive(Component, Clone)]
pub struct ReactiveLayoutWidth {
    pub signal: ReadSignal<f32>,
    pub last_value: f32,
}

impl ReactiveLayoutWidth {
    /// Create a new ReactiveLayoutWidth from a ReadSignal
    pub fn new(signal: ReadSignal<f32>) -> Self {
        let last_value = signal.get_untracked();
        Self { signal, last_value }
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

/// Global resource for mouse position
///
/// Used by the interaction system to perform hit testing and update hover states.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct MousePosition(pub glam::Vec2);

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
