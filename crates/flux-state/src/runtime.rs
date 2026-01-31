//! Reactive runtime with dependency tracking.

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// Unique identifier for reactive nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// The reactive runtime - manages the dependency graph.
///
/// The `Runtime` is the heart of the reactivity system. It maintains:
/// - Storage for signals, computeds, and effects
/// - The dependency graph (who depends on whom)
/// - The current execution context (which effect/computed is running)
///
/// It is typically shared via `Arc<Runtime>` between all reactive primitives.
///
/// # Thread Safety
///
/// `Runtime` is thread-safe (`Send + Sync`) and uses internal `Mutex` locking
/// to allow signals to be read/written from any thread.
///
/// # Example
///
/// ```
/// use flux_state::Runtime;
/// use std::sync::Arc;
///
/// let runtime = Runtime::new();
/// // Pass runtime.clone() to signals/effects
/// ```
pub struct Runtime {
    inner: Mutex<RuntimeInner>,
}

struct RuntimeInner {
    next_id: u64,

    // Signal storage
    signals: HashMap<NodeId, Box<dyn Any + Send>>,

    // Computed storage
    computeds: HashMap<NodeId, ComputedNode>,

    // Effect storage
    effects: HashMap<NodeId, std::sync::Arc<dyn Fn() + Send + Sync>>,

    // Dependency graph
    dependencies: HashMap<NodeId, HashSet<NodeId>>, // node -> its dependencies
    subscribers: HashMap<NodeId, HashSet<NodeId>>,  // node -> nodes that depend on it

    // Current tracking context
    tracking_context: Option<NodeId>,

    // Stale tracking
    stale: HashSet<NodeId>,

    // Pending effects to run
    pending_effects: Vec<NodeId>,
}

struct ComputedNode {
    compute: std::sync::Arc<dyn Fn() -> Box<dyn Any + Send> + Send + Sync>,
    value: Option<Box<dyn Any + Send>>,
}

impl RuntimeInner {
    fn cleanup_dependencies(&mut self, id: NodeId) {
        if let Some(deps) = self.dependencies.remove(&id) {
            for dep in deps {
                if let Some(subs) = self.subscribers.get_mut(&dep) {
                    subs.remove(&id);
                }
            }
        }
    }

    fn mark_stale(&mut self, id: NodeId) -> bool {
        // Only process if not already stale (avoid infinite loops)
        if self.stale.contains(&id) {
            return false;
        }

        self.stale.insert(id);

        // If it's an effect, schedule it
        if self.effects.contains_key(&id) && !self.pending_effects.contains(&id) {
            self.pending_effects.push(id);
        }

        // We should recurse for computed values
        self.computeds.contains_key(&id)
    }

    fn get_subscribers(&self, source: NodeId) -> Vec<NodeId> {
        self.subscribers
            .get(&source)
            .map(|subs| subs.iter().copied().collect())
            .unwrap_or_default()
    }
}

impl Runtime {
    /// Create a new reactive runtime.
    ///
    /// Returns an `Arc<Runtime>` because the runtime must be shared between
    /// all signals, effects, and computed values it manages.
    pub fn new() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            inner: Mutex::new(RuntimeInner {
                next_id: 0,
                signals: HashMap::new(),
                computeds: HashMap::new(),
                effects: HashMap::new(),
                dependencies: HashMap::new(),
                subscribers: HashMap::new(),
                tracking_context: None,
                stale: HashSet::new(),
                pending_effects: Vec::new(),
            }),
        })
    }

    pub(crate) fn create_signal(&self, value: Box<dyn Any + Send>) -> NodeId {
        let mut inner = self.inner.lock().unwrap();
        let id = NodeId(inner.next_id);
        inner.next_id += 1;
        inner.signals.insert(id, value);
        id
    }

    pub(crate) fn create_computed(
        &self,
        compute: std::sync::Arc<dyn Fn() -> Box<dyn Any + Send> + Send + Sync>,
    ) -> NodeId {
        let mut inner = self.inner.lock().unwrap();
        let id = NodeId(inner.next_id);
        inner.next_id += 1;
        inner.computeds.insert(
            id,
            ComputedNode {
                compute,
                value: None,
            },
        );
        id
    }

    pub(crate) fn create_effect(
        &self,
        effect_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
    ) -> NodeId {
        let mut inner = self.inner.lock().unwrap();
        let id = NodeId(inner.next_id);
        inner.next_id += 1;
        inner.effects.insert(id, effect_fn);
        id
    }

    /// Track a dependency (called during signal/computed reads).
    pub(crate) fn track(&self, source: NodeId) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(observer) = inner.tracking_context {
            inner
                .dependencies
                .entry(observer)
                .or_default()
                .insert(source);
            inner
                .subscribers
                .entry(source)
                .or_default()
                .insert(observer);
        }
    }

    /// Notify subscribers that a source changed.
    pub(crate) fn notify(&self, source: NodeId) {
        // Mark all transitive subscribers as stale
        self.mark_stale_recursive(source);

        // Flush pending effects (synchronous for now)
        self.flush_effects();
    }

    fn mark_stale_recursive(&self, source: NodeId) {
        // Collect immediate subscribers
        let subs_to_mark = self.inner.lock().unwrap().get_subscribers(source);

        // Mark each subscriber as stale
        for sub in subs_to_mark {
            let should_recurse = self.inner.lock().unwrap().mark_stale(sub);

            // Recursively mark dependents of this computed
            if should_recurse {
                self.mark_stale_recursive(sub);
            }
        }
    }

    fn flush_effects(&self) {
        loop {
            let effect_id = {
                let mut inner = self.inner.lock().unwrap();
                inner.pending_effects.pop()
            };

            match effect_id {
                Some(id) => self.run_effect(id),
                None => break,
            }
        }
    }

    pub(crate) fn run_effect(&self, id: NodeId) {
        // Clear old dependencies
        {
            let mut inner = self.inner.lock().unwrap();
            inner.cleanup_dependencies(id);
            inner.stale.remove(&id);
            inner.tracking_context = Some(id);
        }

        // Run effect (will re-establish dependencies)
        let effect_fn = {
            let inner = self.inner.lock().unwrap();
            inner.effects.get(&id).cloned()
        };

        if let Some(f) = effect_fn {
            f();
        }

        // Clear tracking context
        self.inner.lock().unwrap().tracking_context = None;
    }

    pub(crate) fn with_signal_value<R, F>(&self, id: NodeId, f: F) -> R
    where
        F: FnOnce(&dyn Any) -> R,
    {
        let inner = self.inner.lock().unwrap();
        let value = inner.signals.get(&id).expect("Signal not found");
        f(value.as_ref())
    }

    pub(crate) fn is_stale(&self, id: NodeId) -> bool {
        self.inner.lock().unwrap().stale.contains(&id)
    }

    pub(crate) fn recompute(&self, id: NodeId) {
        // Clear old dependencies and set tracking context
        {
            let mut inner = self.inner.lock().unwrap();
            inner.cleanup_dependencies(id);
            inner.stale.remove(&id);
            inner.tracking_context = Some(id);
        }
        // Drop borrow before running user code!

        // Get compute function and run it (without holding any borrows)
        let new_value = {
            let compute_fn = {
                let inner = self.inner.lock().unwrap();
                let computed = inner.computeds.get(&id).expect("Computed not found");
                computed.compute.clone()
            };
            // Call without holding borrow
            compute_fn()
        };

        // Store new value and clear tracking context
        {
            let mut inner = self.inner.lock().unwrap();
            if let Some(computed) = inner.computeds.get_mut(&id) {
                computed.value = Some(new_value);
            }
            inner.tracking_context = None;
        }
    }

    pub(crate) fn with_computed_value<R, F>(&self, id: NodeId, f: F) -> R
    where
        F: FnOnce(&dyn Any) -> R,
    {
        let inner = self.inner.lock().unwrap();
        let computed = inner.computeds.get(&id).expect("Computed not found");
        let value = computed
            .value
            .as_ref()
            .expect("Computed value not initialized");
        f(value.as_ref())
    }

    pub(crate) fn dispose_effect(&self, id: NodeId) {
        let mut inner = self.inner.lock().unwrap();

        // Remove from effects
        inner.effects.remove(&id);

        // Clean up dependencies
        inner.cleanup_dependencies(id);
    }
}
