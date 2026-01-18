use bevy_ecs::prelude::*;
use flux_state::ReadSignal;
use render_engine::{Color, NodeId, Transform2D};

/// Reference to a node in the Scene tree
///
/// This component links an ECS entity to a specific node in the custom Scene tree.
/// Systems can query this component to find which scene node to operate on.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneNodeRef(pub NodeId);

/// Thread-safe wrapper for ReadSignal
///
/// # Safety
///
/// This type unsafely implements Send + Sync to satisfy bevy_ecs's requirements.
/// This is safe in the context of Arthropod because:
/// 1. GUI applications run entirely on the main thread
/// 2. The FrameworkContext and all ECS systems run synchronously on the main thread
/// 3. No parallel system execution is used
///
/// Users must ensure they never move FrameworkContext or components containing
/// MainThreadSignal to other threads.
pub struct MainThreadSignal<T: 'static>(ReadSignal<T>);

impl<T: 'static> MainThreadSignal<T> {
    /// Create a new MainThreadSignal from a ReadSignal
    ///
    /// # Safety
    ///
    /// The signal must only be used from the main thread. The caller must ensure
    /// this wrapper is never sent to or accessed from other threads.
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

// SAFETY: See MainThreadSignal documentation. This is safe because Arthropod
// runs all GUI operations on the main thread.
unsafe impl<T: 'static> Send for MainThreadSignal<T> {}
unsafe impl<T: 'static> Sync for MainThreadSignal<T> {}

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
