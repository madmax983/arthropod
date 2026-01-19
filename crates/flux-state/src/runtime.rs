//! Reactive runtime with dependency tracking.

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// Unique identifier for reactive nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// The reactive runtime - manages the dependency graph.
///
/// Stored as `Arc<Runtime>` and shared between signals and effects.
/// Thread-safe using Mutex for interior mutability.
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
    effects: HashMap<NodeId, Box<dyn Fn() + Send>>,

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
    compute: Box<dyn Fn() -> Box<dyn Any + Send> + Send>,
    value: Option<Box<dyn Any + Send>>,
}

impl Runtime {
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
        compute: Box<dyn Fn() -> Box<dyn Any + Send> + Send>,
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

    pub(crate) fn create_effect(&self, effect_fn: Box<dyn Fn() + Send>) -> NodeId {
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
        let subs_to_mark: Vec<NodeId> = {
            let inner = self.inner.lock().unwrap();
            inner
                .subscribers
                .get(&source)
                .map(|subs| subs.iter().copied().collect())
                .unwrap_or_default()
        };

        // Mark each subscriber as stale
        for sub in subs_to_mark.iter() {
            let should_recurse = {
                let mut inner = self.inner.lock().unwrap();

                // Only process if not already stale (avoid infinite loops)
                if inner.stale.contains(sub) {
                    false
                } else {
                    inner.stale.insert(*sub);

                    // If it's an effect, schedule it
                    if inner.effects.contains_key(sub) && !inner.pending_effects.contains(sub) {
                        inner.pending_effects.push(*sub);
                    }

                    // We should recurse for computed values
                    inner.computeds.contains_key(sub)
                }
            };

            // Recursively mark dependents of this computed
            if should_recurse {
                self.mark_stale_recursive(*sub);
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
            if let Some(deps) = inner.dependencies.remove(&id) {
                for dep in deps {
                    if let Some(subs) = inner.subscribers.get_mut(&dep) {
                        subs.remove(&id);
                    }
                }
            }
            inner.stale.remove(&id);
            inner.tracking_context = Some(id);
        }

        // Run effect (will re-establish dependencies)
        let effect_fn = {
            let inner = self.inner.lock().unwrap();
            inner.effects.get(&id).map(|f| f as *const dyn Fn())
        };

        if let Some(f) = effect_fn {
            unsafe { (*f)() };
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

            // Clear old dependencies
            if let Some(deps) = inner.dependencies.remove(&id) {
                for dep in deps {
                    if let Some(subs) = inner.subscribers.get_mut(&dep) {
                        subs.remove(&id);
                    }
                }
            }

            inner.stale.remove(&id);
            inner.tracking_context = Some(id);
        }
        // Drop borrow before running user code!

        // Get compute function pointer and run it (without holding any borrows)
        let new_value = {
            let compute_fn = {
                let inner = self.inner.lock().unwrap();
                let computed = inner.computeds.get(&id).expect("Computed not found");
                &computed.compute as *const dyn Fn() -> Box<dyn std::any::Any + Send>
            };
            // Call without holding borrow
            unsafe { (*compute_fn)() }
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
        if let Some(deps) = inner.dependencies.remove(&id) {
            for dep in deps {
                if let Some(subs) = inner.subscribers.get_mut(&dep) {
                    subs.remove(&id);
                }
            }
        }
    }
}
